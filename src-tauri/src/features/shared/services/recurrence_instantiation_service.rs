/// 循环任务实例化服务
///
/// 核心逻辑：根据 RRULE 标准规则在某一天自动创建任务实例
use chrono::NaiveDate;
use rrule::RRuleSet;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    entities::{Task, TaskRecurrence, TaskRecurrenceLink, Template, TemplateRow},
    features::{
        shared::repositories::{
            TaskRecurrenceLinkRepository, TaskRecurrenceRepository, TaskRepository,
            TaskScheduleRepository,
        },
        shared::TransactionHelper,
    },
    infra::{
        core::{utils::time_utils, AppError, AppResult},
        Clock, IdGenerator,
    },
};

pub struct RecurrenceInstantiationService;

impl RecurrenceInstantiationService {
    /// 为某一天实例化所有生效的循环任务
    ///
    /// 返回该天的所有循环任务实例
    pub async fn instantiate_for_date(
        pool: &SqlitePool,
        id_generator: &dyn IdGenerator,
        clock: &dyn Clock,
        target_date: &NaiveDate,
    ) -> AppResult<Vec<Uuid>> {
        let date_str = time_utils::format_date_yyyy_mm_dd(target_date);

        tracing::info!(
            "🔄 [RECURRENCE] Starting instantiation for date: {}",
            date_str
        );

        // 1. 查询在该日期生效的所有循环规则
        let recurrences =
            TaskRecurrenceRepository::find_effective_for_date(pool, &date_str).await?;

        tracing::info!(
            "🔄 [RECURRENCE] Found {} active recurrence rules for date {}",
            recurrences.len(),
            date_str
        );

        if recurrences.is_empty() {
            tracing::warn!("🔄 [RECURRENCE] No active recurrence rules found!");
        } else {
            for (idx, rec) in recurrences.iter().enumerate() {
                tracing::info!(
                    "🔄 [RECURRENCE] Rule #{}: id={}, template_id={}, rule={}, is_active={}",
                    idx + 1,
                    rec.id,
                    rec.template_id,
                    rec.rule,
                    rec.is_active
                );
            }
        }

        let mut task_ids = Vec::new();

        // 2. 对每个循环规则，检查是否需要实例化
        for recurrence in recurrences {
            tracing::info!("🔄 [RECURRENCE] Processing recurrence: {}", recurrence.id);

            // 2.1 解析 RRULE，判断是否匹配今天
            tracing::info!(
                "🔄 [RECURRENCE] Checking RRULE match for: {}",
                recurrence.rule
            );
            let matches = Self::date_matches_rrule(target_date, &recurrence)?;
            tracing::info!("🔄 [RECURRENCE] RRULE match result: {}", matches);

            if !matches {
                tracing::warn!(
                    "🔄 [RECURRENCE] ❌ Recurrence {} does not match date {}",
                    recurrence.id,
                    date_str
                );
                continue;
            }

            tracing::info!(
                "🔄 [RECURRENCE] ✅ Recurrence {} matches date {}",
                recurrence.id,
                date_str
            );

            // 2.2 检查链接表是否已有今天的实例
            tracing::info!(
                "🔄 [RECURRENCE] Checking if link exists for recurrence {} on {}",
                recurrence.id,
                date_str
            );
            match TaskRecurrenceLinkRepository::find_link(pool, recurrence.id, &date_str).await? {
                Some(link) => {
                    tracing::info!(
                        "🔄 [RECURRENCE] Found existing link for task {}",
                        link.task_id
                    );
                    // 2.3 已有链接，验证任务是否仍属于今天
                    if Self::validate_task_instance(pool, link.task_id, &date_str).await? {
                        task_ids.push(link.task_id);
                        tracing::info!(
                            "🔄 [RECURRENCE] ✅ Existing valid task instance {} for recurrence {}",
                            link.task_id,
                            recurrence.id
                        );
                    } else {
                        tracing::warn!(
                            "🔄 [RECURRENCE] ⚠️ Task {} is no longer valid for date {}, user may have adjusted it",
                            link.task_id,
                            date_str
                        );
                    }
                }
                None => {
                    tracing::info!(
                        "🔄 [RECURRENCE] No existing link found, creating new task instance"
                    );
                    // 2.4 没有链接，创建新任务实例
                    match Self::create_task_instance(
                        pool,
                        id_generator,
                        clock,
                        &recurrence,
                        &date_str,
                    )
                    .await
                    {
                        Ok(task_id) => {
                            task_ids.push(task_id);
                            tracing::info!(
                                "🔄 [RECURRENCE] ✅ Created new task instance {} for recurrence {}",
                                task_id,
                                recurrence.id
                            );
                        }
                        Err(e) => {
                            tracing::error!(
                                "🔄 [RECURRENCE] ❌ Failed to create task instance for recurrence {}: {:?}",
                                recurrence.id,
                                e
                            );
                        }
                    }
                }
            }
        }

        Ok(task_ids)
    }

    /// 判断日期是否匹配 RRULE
    fn date_matches_rrule(date: &NaiveDate, recurrence: &TaskRecurrence) -> AppResult<bool> {
        tracing::info!("🔄 [RRULE] Parsing RRULE: {}", recurrence.rule);
        tracing::info!("🔄 [RRULE] Start date: {:?}", recurrence.start_date);

        // 确定 DTSTART：优先使用 start_date，否则使用 created_at 的日期部分
        let dtstart_date = if let Some(ref start_date) = recurrence.start_date {
            start_date.clone()
        } else {
            // 使用 created_at 的日期部分
            let created_date = recurrence.created_at.date_naive();
            crate::infra::core::utils::time_utils::format_date_yyyy_mm_dd(&created_date)
        };

        tracing::info!("🔄 [RRULE] Using DTSTART: {}", dtstart_date);

        // 构建完整的 RRULE 字符串（包含 DTSTART）
        // 将 start_date (YYYY-MM-DD) 转换为 RRULE 的 DTSTART 格式 (YYYYMMDD)
        let start_date_rrule = dtstart_date.replace("-", "");
        let full_rrule = format!("DTSTART:{}\nRRULE:{}", start_date_rrule, recurrence.rule);

        tracing::info!("🔄 [RRULE] Full RRULE string:\n{}", full_rrule);

        let rrule_set: RRuleSet = full_rrule.parse().map_err(|e| {
            tracing::error!("🔄 [RRULE] ❌ Failed to parse RRULE: {:?}", e);
            AppError::ValidationFailed(vec![crate::infra::core::ValidationError::new(
                "rule".to_string(),
                format!("Invalid RRULE: {:?}", e),
                "INVALID_RRULE".to_string(),
            )])
        })?;

        tracing::info!("🔄 [RRULE] ✅ Successfully parsed RRULE");
        tracing::info!("🔄 [RRULE] Target date: {}, checking occurrences...", date);

        // 检查该日期是否在 RRULE 生成的日期集合中
        let occurrences = rrule_set.into_iter();
        let mut count = 0;

        for occurrence in occurrences {
            count += 1;
            let occ_date = occurrence.date_naive();
            tracing::debug!("🔄 [RRULE] Occurrence #{}: {}", count, occ_date);

            if occ_date == *date {
                tracing::info!("🔄 [RRULE] ✅ Found matching occurrence: {}", occ_date);
                return Ok(true);
            }
            // 如果已经超过目标日期，停止检查
            if occ_date > *date {
                tracing::info!(
                    "🔄 [RRULE] Reached future date {}, stopping search",
                    occ_date
                );
                break;
            }

            // 限制检查次数，防止无限循环
            if count > 1000 {
                tracing::warn!("🔄 [RRULE] ⚠️ Checked 1000 occurrences, stopping");
                break;
            }
        }

        tracing::warn!(
            "🔄 [RRULE] ❌ No matching occurrence found after checking {} dates",
            count
        );
        Ok(false)
    }

    /// 创建任务实例
    async fn create_task_instance(
        pool: &SqlitePool,
        id_generator: &dyn IdGenerator,
        clock: &dyn Clock,
        recurrence: &TaskRecurrence,
        instance_date: &str,
    ) -> AppResult<Uuid> {
        tracing::info!(
            "🔄 [CREATE] Creating task instance for recurrence {} on {}",
            recurrence.id,
            instance_date
        );

        // 1. 查询模板
        tracing::info!("🔄 [CREATE] Looking up template {}", recurrence.template_id);
        let template = Self::find_template(pool, recurrence.template_id).await?;
        tracing::info!("🔄 [CREATE] ✅ Found template: {}", template.title);

        // 2. 生成ID和时间
        let task_id = id_generator.new_uuid();
        let now = clock.now_utc();
        tracing::info!("🔄 [CREATE] Generated task_id: {}", task_id);

        // 3. 开启事务
        tracing::info!("🔄 [CREATE] Starting transaction...");
        let mut tx = TransactionHelper::begin(pool).await?;

        // 4. 替换变量（目前只支持 {{date}}）
        let mut variables = std::collections::HashMap::new();
        variables.insert("date".to_string(), instance_date.to_string());

        let title = Self::replace_variables(&template.title, &variables);
        let glance_note = template
            .glance_note_template
            .as_ref()
            .map(|s| Self::replace_variables(s, &variables));
        let detail_note = template
            .detail_note_template
            .as_ref()
            .map(|s| Self::replace_variables(s, &variables));

        // 5. 创建任务
        let source_info_json = serde_json::json!({
            "source_type": "native::from_recurrence",
            "recurrence_id": recurrence.id.to_string(),
            "template_id": template.id.to_string(),
            "instance_date": instance_date,
        });

        let task = Task {
            id: task_id,
            title,
            glance_note,
            detail_note,
            estimated_duration: template.estimated_duration_template,
            subtasks: template.subtasks_template.clone(),
            sort_positions: std::collections::HashMap::new(),
            project_id: None,
            section_id: None,
            area_id: template.area_id,
            due_date: None,
            due_date_type: None,
            completed_at: None,
            archived_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            source_info: serde_json::from_value(source_info_json).ok(),
            external_source_id: None,
            external_source_provider: None,
            external_source_metadata: None,
            recurrence_id: Some(recurrence.id),
            recurrence_original_date: Some(instance_date.to_string()),
        };

        tracing::info!("🔄 [CREATE] Inserting task into database: {}", task.title);
        TaskRepository::insert_in_tx(&mut tx, &task).await?;
        tracing::info!("🔄 [CREATE] ✅ Task inserted");

        // 6. 创建日程记录
        tracing::info!("🔄 [CREATE] Creating schedule for date: {}", instance_date);
        TaskScheduleRepository::create_in_tx(&mut tx, task_id, instance_date).await?;
        tracing::info!("🔄 [CREATE] ✅ Schedule created");

        // 7. 创建循环链接
        tracing::info!("🔄 [CREATE] Creating recurrence link");
        let link = TaskRecurrenceLink::new(recurrence.id, instance_date.to_string(), task_id, now);
        TaskRecurrenceLinkRepository::insert_in_tx(&mut tx, &link).await?;
        tracing::info!("🔄 [CREATE] ✅ Recurrence link created");

        // 8. 提交事务
        tracing::info!("🔄 [CREATE] Committing transaction...");
        TransactionHelper::commit(tx).await?;
        tracing::info!("🔄 [CREATE] ✅ Transaction committed");

        tracing::info!(
            "🔄 [CREATE] 🎉 Successfully created task instance {}",
            task_id
        );
        Ok(task_id)
    }

    /// 验证任务实例是否仍属于该日期
    ///
    /// 检查条件：
    /// 1. 任务未删除
    /// 2. 任务未归档
    /// 3. 任务在该日期有日程记录
    async fn validate_task_instance(
        pool: &SqlitePool,
        task_id: Uuid,
        date_str: &str,
    ) -> AppResult<bool> {
        // 1. 查询任务
        let task = match TaskRepository::find_by_id(pool, task_id).await? {
            Some(t) => t,
            None => return Ok(false),
        };

        // 2. 检查任务状态
        if task.deleted_at.is_some() || task.archived_at.is_some() {
            return Ok(false);
        }

        // 3. 检查是否有该日期的日程记录
        let has_schedule =
            TaskScheduleRepository::has_schedule_for_day(pool, task_id, date_str).await?;

        Ok(has_schedule)
    }

    /// 查询模板
    async fn find_template(pool: &SqlitePool, template_id: Uuid) -> AppResult<Template> {
        let query = r#"
            SELECT id, title, glance_note_template, detail_note_template,
                   estimated_duration_template, subtasks_template, area_id, category,
                   sort_rank, created_at, updated_at, is_deleted
            FROM templates
            WHERE id = ? AND is_deleted = 0
        "#;

        let row = sqlx::query_as::<_, TemplateRow>(query)
            .bind(template_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
            })?;

        match row {
            Some(r) => Template::try_from(r)
                .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::QueryError(e))),
            None => Err(AppError::NotFound {
                entity_type: "Template".to_string(),
                entity_id: template_id.to_string(),
            }),
        }
    }

    /// 替换变量
    fn replace_variables(
        template: &str,
        variables: &std::collections::HashMap<String, String>,
    ) -> String {
        let mut result = template.to_string();
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }
}
