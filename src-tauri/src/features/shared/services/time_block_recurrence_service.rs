/// 时间块循环实例化服务
///
/// 核心逻辑：根据 RRULE 标准规则在某一天自动创建时间块实例
use chrono::{Local, NaiveDate, NaiveTime, TimeZone};
use rrule::RRuleSet;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    entities::{TimeBlock, TimeBlockRecurrence, TimeBlockRecurrenceLink, TimeBlockTemplate},
    features::shared::{
        repositories::{
            TimeBlockRecurrenceLinkRepository, TimeBlockRecurrenceRepository, TimeBlockRepository,
            TimeBlockTemplateRepository,
        },
        TransactionHelper,
    },
    infra::{
        core::{utils::time_utils, AppError, AppResult},
        Clock, IdGenerator,
    },
};

pub struct TimeBlockRecurrenceInstantiationService;

impl TimeBlockRecurrenceInstantiationService {
    /// 为某一天实例化所有生效的循环时间块
    ///
    /// 返回该天成功创建的所有循环时间块实例ID
    pub async fn instantiate_for_date(
        pool: &SqlitePool,
        id_generator: &dyn IdGenerator,
        clock: &dyn Clock,
        target_date: &NaiveDate,
    ) -> AppResult<Vec<Uuid>> {
        let date_str = time_utils::format_date_yyyy_mm_dd(target_date);

        tracing::info!(
            "🔄 [TB_RECURRENCE] Starting instantiation for date: {}",
            date_str
        );

        // 1. 查询在该日期生效的所有时间块循环规则
        let recurrences =
            TimeBlockRecurrenceRepository::find_effective_for_date(pool, &date_str).await?;

        tracing::info!(
            "🔄 [TB_RECURRENCE] Found {} active recurrence rules for date {}",
            recurrences.len(),
            date_str
        );

        if recurrences.is_empty() {
            return Ok(Vec::new());
        }

        let mut time_block_ids = Vec::new();

        // 2. 对每个循环规则，检查是否需要实例化
        for recurrence in recurrences {
            tracing::info!(
                "🔄 [TB_RECURRENCE] Processing recurrence: {}",
                recurrence.id
            );

            // 2.1 解析 RRULE，判断是否匹配今天
            let matches = Self::date_matches_rrule(target_date, &recurrence)?;

            if !matches {
                tracing::debug!(
                    "🔄 [TB_RECURRENCE] Recurrence {} does not match date {}",
                    recurrence.id,
                    date_str
                );
                continue;
            }

            tracing::info!(
                "🔄 [TB_RECURRENCE] ✅ Recurrence {} matches date {}",
                recurrence.id,
                date_str
            );

            // 2.2 检查链接表是否已有今天的实例
            match TimeBlockRecurrenceLinkRepository::find_link(pool, recurrence.id, &date_str)
                .await?
            {
                Some(link) => {
                    // 2.3 已有链接，验证时间块是否仍有效
                    if Self::validate_time_block_instance(pool, link.time_block_id).await? {
                        time_block_ids.push(link.time_block_id);
                        tracing::info!(
                            "🔄 [TB_RECURRENCE] ✅ Existing valid time block {} for recurrence {}",
                            link.time_block_id,
                            recurrence.id
                        );
                    } else {
                        tracing::warn!(
                            "🔄 [TB_RECURRENCE] ⚠️ Time block {} is no longer valid",
                            link.time_block_id
                        );
                    }
                }
                None => {
                    // 2.4 没有链接，创建新时间块实例
                    match Self::create_time_block_instance(
                        pool,
                        id_generator,
                        clock,
                        &recurrence,
                        target_date,
                    )
                    .await
                    {
                        Ok(Some(time_block_id)) => {
                            time_block_ids.push(time_block_id);
                            tracing::info!(
                                "🔄 [TB_RECURRENCE] ✅ Created new time block {} for recurrence {}",
                                time_block_id,
                                recurrence.id
                            );
                        }
                        Ok(None) => {
                            // 跳过了（冲突）
                            tracing::info!(
                                "🔄 [TB_RECURRENCE] ⏭️ Skipped recurrence {} due to conflict",
                                recurrence.id
                            );
                        }
                        Err(e) => {
                            tracing::error!(
                                "🔄 [TB_RECURRENCE] ❌ Failed to create time block for recurrence {}: {:?}",
                                recurrence.id,
                                e
                            );
                        }
                    }
                }
            }
        }

        Ok(time_block_ids)
    }

    /// 判断日期是否匹配 RRULE
    fn date_matches_rrule(date: &NaiveDate, recurrence: &TimeBlockRecurrence) -> AppResult<bool> {
        // 确定 DTSTART：优先使用 start_date，否则使用 created_at 的日期部分
        let dtstart_date = if let Some(ref start_date) = recurrence.start_date {
            start_date.clone()
        } else {
            let created_date = recurrence.created_at.date_naive();
            time_utils::format_date_yyyy_mm_dd(&created_date)
        };

        // 构建完整的 RRULE 字符串（包含 DTSTART）
        let start_date_rrule = dtstart_date.replace("-", "");
        let full_rrule = format!("DTSTART:{}\nRRULE:{}", start_date_rrule, recurrence.rule);

        let rrule_set: RRuleSet = full_rrule.parse().map_err(|e| {
            AppError::ValidationFailed(vec![crate::infra::core::ValidationError::new(
                "rule".to_string(),
                format!("Invalid RRULE: {:?}", e),
                "INVALID_RRULE".to_string(),
            )])
        })?;

        // 检查该日期是否在 RRULE 生成的日期集合中
        let occurrences = rrule_set.into_iter();
        let mut count = 0;

        for occurrence in occurrences {
            count += 1;
            let occ_date = occurrence.date_naive();

            if occ_date == *date {
                return Ok(true);
            }
            // 如果已经超过目标日期，停止检查
            if occ_date > *date {
                break;
            }
            // 限制检查次数
            if count > 1000 {
                break;
            }
        }

        Ok(false)
    }

    /// 创建时间块实例
    async fn create_time_block_instance(
        pool: &SqlitePool,
        id_generator: &dyn IdGenerator,
        clock: &dyn Clock,
        recurrence: &TimeBlockRecurrence,
        target_date: &NaiveDate,
    ) -> AppResult<Option<Uuid>> {
        let date_str = time_utils::format_date_yyyy_mm_dd(target_date);

        // 1. 查询模板
        let template = Self::find_template(pool, recurrence.template_id).await?;

        // 2. 计算该日期的具体时间
        let (start_time, end_time, start_time_local, end_time_local) =
            Self::calculate_instance_times(target_date, &template)?;

        // 3. 生成ID和时间
        let time_block_id = id_generator.new_uuid();
        let now = clock.now_utc();

        // 4. 开启事务
        let mut tx = TransactionHelper::begin(pool).await?;

        // 5. 创建时间块实例
        let source_info_json = serde_json::json!({
            "source_type": "native::from_time_block_recurrence",
            "recurrence_id": recurrence.id.to_string(),
            "template_id": template.id.to_string(),
            "instance_date": date_str,
        });

        // 注意：recurrence_parent_id 有外键约束指向 time_blocks(id)，不能设置为循环规则ID
        // 对于自动生成的循环实例，recurrence_parent_id 应为 None
        let time_block = TimeBlock {
            id: time_block_id,
            title: template.title.clone(),
            glance_note: template.glance_note_template.clone(),
            detail_note: template.detail_note_template.clone(),
            start_time,
            end_time,
            start_time_local: Some(start_time_local),
            end_time_local: Some(end_time_local),
            time_type: template.time_type,
            creation_timezone: None,
            is_all_day: template.is_all_day,
            area_id: template.area_id,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            source_info: serde_json::from_value(source_info_json).ok(),
            external_source_id: None,
            external_source_provider: None,
            external_source_metadata: None,
            recurrence_rule: Some(recurrence.rule.clone()),
            recurrence_parent_id: None, // 外键约束指向 time_blocks(id)，循环实例无父时间块
            recurrence_original_date: Some(date_str.clone()),
        };

        TimeBlockRepository::insert_in_tx(&mut tx, &time_block).await?;

        // 7. 创建循环链接
        let link =
            TimeBlockRecurrenceLink::new(recurrence.id, date_str.clone(), time_block_id, now);
        TimeBlockRecurrenceLinkRepository::insert_in_tx(&mut tx, &link).await?;

        // 8. 提交事务
        TransactionHelper::commit(tx).await?;

        Ok(Some(time_block_id))
    }

    /// 计算实例的具体时间
    fn calculate_instance_times(
        target_date: &NaiveDate,
        template: &TimeBlockTemplate,
    ) -> AppResult<(
        chrono::DateTime<chrono::Utc>,
        chrono::DateTime<chrono::Utc>,
        String,
        String,
    )> {
        // 解析 start_time_local (HH:MM:SS)
        let start_time_local = NaiveTime::parse_from_str(&template.start_time_local, "%H:%M:%S")
            .map_err(|e| {
                AppError::ValidationFailed(vec![crate::infra::core::ValidationError::new(
                    "start_time_local".to_string(),
                    format!("Invalid time format: {:?}", e),
                    "INVALID_TIME_FORMAT".to_string(),
                )])
            })?;

        // 计算结束时间
        let duration = chrono::Duration::minutes(template.duration_minutes as i64);
        let end_time_local = start_time_local + duration;

        // 构建本地日期时间
        let local_start = target_date.and_time(start_time_local);
        let local_end = target_date.and_time(end_time_local);

        // 转换为 UTC（假设本地时区）
        let local_tz = Local;
        let start_utc = local_tz
            .from_local_datetime(&local_start)
            .single()
            .ok_or_else(|| {
                AppError::ValidationFailed(vec![crate::infra::core::ValidationError::new(
                    "start_time".to_string(),
                    "Ambiguous or invalid local time".to_string(),
                    "INVALID_LOCAL_TIME".to_string(),
                )])
            })?
            .with_timezone(&chrono::Utc);

        let end_utc = local_tz
            .from_local_datetime(&local_end)
            .single()
            .ok_or_else(|| {
                AppError::ValidationFailed(vec![crate::infra::core::ValidationError::new(
                    "end_time".to_string(),
                    "Ambiguous or invalid local time".to_string(),
                    "INVALID_LOCAL_TIME".to_string(),
                )])
            })?
            .with_timezone(&chrono::Utc);

        // 格式化本地时间字符串
        let start_local_str = start_time_local.format("%H:%M:%S").to_string();
        let end_local_str = end_time_local.format("%H:%M:%S").to_string();

        Ok((start_utc, end_utc, start_local_str, end_local_str))
    }

    /// 验证时间块实例是否仍有效
    async fn validate_time_block_instance(
        pool: &SqlitePool,
        time_block_id: Uuid,
    ) -> AppResult<bool> {
        // find_by_id 在找不到时返回错误，所以用 match 处理
        match TimeBlockRepository::find_by_id(pool, time_block_id).await {
            Ok(time_block) => {
                // 检查时间块是否已删除
                Ok(!time_block.is_deleted)
            }
            Err(_) => {
                // 找不到时间块，视为无效
                Ok(false)
            }
        }
    }

    /// 查询模板
    async fn find_template(pool: &SqlitePool, template_id: Uuid) -> AppResult<TimeBlockTemplate> {
        TimeBlockTemplateRepository::find_by_id(pool, template_id)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity_type: "TimeBlockTemplate".to_string(),
                entity_id: template_id.to_string(),
            })
    }
}
