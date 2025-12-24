/// 批量加载日期范围内的每日任务视图 - 单文件组件
///
/// GET /api/views/daily-range?start_view_key=daily::2025-10-01&end_view_key=daily::2025-10-07
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::Entry, HashMap};
use uuid::Uuid;

use crate::{
    entities::{Task, TaskCardDto},
    features::shared::{RecurrenceInstantiationService, ViewTaskCardAssembler},
    infra::{
        core::{utils::time_utils, AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

const MAX_RANGE_DAYS: i64 = 62;

// ==================== 文档层 ====================
/*
CABC for `get_daily_tasks_batch`

## 1. 端点签名
GET /api/views/daily-range?start_view_key=daily::2025-10-01&end_view_key=daily::2025-10-07

## 2. 场景
一次性为日历或多天看板加载一个日期范围内的任务，避免前端循环触发几十个 /daily/:date 请求。

## 3. 输入输出
- Query:
  - `start_view_key`：必须是 `daily::YYYY-MM-DD`
  - `end_view_key`：同上
- 响应：
```json
{
  "range": {
    "start_view_key": "daily::2025-10-01",
    "end_view_key": "daily::2025-10-07",
    "start_date": "2025-10-01",
    "end_date": "2025-10-07",
    "total_days": 7
  },
  "views": [
    {
      "view_key": "daily::2025-10-01",
      "date": "2025-10-01",
      "count": 4,
      "tasks": [TaskCardDto, ...]
    }
  ],
  "total_tasks": 24
}
```

## 4. 约束
- 仅接受 daily view key
- 范围上限 62 天，超出报错
- 自动为每一天触发循环任务实例化再查询

## 5. 主要流程
1. 校验 view key → 解析出日期
2. 确保范围合法并生成日期列表
3. 逐日调用 `RecurrenceInstantiationService::instantiate_for_date`
4. 单次查询拿到范围内所有任务 + 对应日期
5. 批量装配 TaskCard 再按日期聚合成视图
6. 返回 range 元信息 + 每日数据
*/

// ==================== 请求 & 响应结构 ====================
#[derive(Debug, Deserialize)]
pub struct BatchDailyTasksQuery {
    pub start_view_key: String,
    pub end_view_key: String,
}

#[derive(Debug, Serialize)]
pub struct DailyRangeMeta {
    pub start_view_key: String,
    pub end_view_key: String,
    pub start_date: String,
    pub end_date: String,
    pub total_days: usize,
}

#[derive(Debug, Serialize)]
pub struct DailyViewTasksPayload {
    pub view_key: String,
    pub date: String,
    pub count: usize,
    pub tasks: Vec<TaskCardDto>,
}

#[derive(Debug, Serialize)]
pub struct BatchDailyTasksResponse {
    pub range: DailyRangeMeta,
    pub views: Vec<DailyViewTasksPayload>,
    pub total_tasks: usize,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Query(query): Query<BatchDailyTasksQuery>,
) -> Response {
    match logic::execute(&app_state, query).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    const DAILY_PREFIX: &str = "daily::";

    pub fn parse_daily_view_key(field: &str, value: &str) -> AppResult<NaiveDate> {
        if !value.starts_with(DAILY_PREFIX) {
            return Err(AppError::validation_error(
                field,
                "view_key 仅支持 daily::YYYY-MM-DD 格式",
                "INVALID_VIEW_KEY_TYPE",
            ));
        }

        let date_part = &value[DAILY_PREFIX.len()..];
        time_utils::parse_date_yyyy_mm_dd(date_part).map_err(|_| {
            AppError::validation_error(
                field,
                "日期格式错误，请使用 YYYY-MM-DD",
                "INVALID_DATE_FORMAT",
            )
        })
    }

    pub fn ensure_range(start: &NaiveDate, end: &NaiveDate) -> AppResult<()> {
        if end < start {
            return Err(AppError::validation_error(
                "end_view_key",
                "结束日期必须晚于或等于开始日期",
                "INVALID_RANGE_ORDER",
            ));
        }

        let span = (*end - *start).num_days() + 1;
        if span > MAX_RANGE_DAYS {
            return Err(AppError::validation_error(
                "end_view_key",
                format!("日期范围最多支持 {} 天（当前 {} 天）", MAX_RANGE_DAYS, span),
                "RANGE_TOO_LARGE",
            ));
        }

        Ok(())
    }

    pub fn build_date_list(start: &NaiveDate, end: &NaiveDate) -> Vec<NaiveDate> {
        let mut dates = Vec::new();
        let mut cursor = *start;
        while cursor <= *end {
            dates.push(cursor);
            cursor += Duration::days(1);
        }
        dates
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        query: BatchDailyTasksQuery,
    ) -> AppResult<BatchDailyTasksResponse> {
        let start_date = validation::parse_daily_view_key("start_view_key", &query.start_view_key)?;
        let end_date = validation::parse_daily_view_key("end_view_key", &query.end_view_key)?;
        validation::ensure_range(&start_date, &end_date)?;

        let date_list = validation::build_date_list(&start_date, &end_date);
        instantiate_recurrences(app_state, &date_list).await?;

        let records =
            database::find_tasks_for_range(app_state.db_pool(), &start_date, &end_date).await?;

        let (ids_by_date, unique_tasks) = group_tasks(records);
        let task_cards = ViewTaskCardAssembler::assemble_batch(
            unique_tasks.into_values().collect(),
            app_state.db_pool(),
        )
        .await?;

        let mut card_lookup: HashMap<Uuid, TaskCardDto> = HashMap::new();
        for card in task_cards {
            card_lookup.insert(card.id, card);
        }

        let date_strings: Vec<String> = date_list
            .iter()
            .map(time_utils::format_date_yyyy_mm_dd)
            .collect();

        let mut views = Vec::with_capacity(date_strings.len());
        let mut total_tasks = 0usize;
        for date_str in &date_strings {
            let mut tasks = Vec::new();
            if let Some(task_ids) = ids_by_date.get(date_str) {
                for task_id in task_ids {
                    if let Some(card) = card_lookup.get(task_id) {
                        tasks.push(card.clone());
                    }
                }
            }

            total_tasks += tasks.len();
            views.push(DailyViewTasksPayload {
                view_key: format!("daily::{}", date_str),
                date: date_str.clone(),
                count: tasks.len(),
                tasks,
            });
        }

        let start_date_str = date_strings
            .first()
            .cloned()
            .unwrap_or_else(|| time_utils::format_date_yyyy_mm_dd(&start_date));
        let end_date_str = date_strings
            .last()
            .cloned()
            .unwrap_or_else(|| start_date_str.clone());

        Ok(BatchDailyTasksResponse {
            range: DailyRangeMeta {
                start_view_key: format!("daily::{}", start_date_str),
                end_view_key: format!("daily::{}", end_date_str),
                start_date: start_date_str,
                end_date: end_date_str,
                total_days: date_strings.len(),
            },
            views,
            total_tasks,
        })
    }

    async fn instantiate_recurrences(app_state: &AppState, dates: &[NaiveDate]) -> AppResult<()> {
        for date in dates {
            RecurrenceInstantiationService::instantiate_for_date(
                app_state.db_pool(),
                app_state.id_generator().as_ref(),
                app_state.clock().as_ref(),
                date,
            )
            .await?;
        }
        Ok(())
    }

    fn group_tasks(
        records: Vec<database::DailyTaskRecord>,
    ) -> (HashMap<String, Vec<Uuid>>, HashMap<Uuid, Task>) {
        let mut ids_by_date: HashMap<String, Vec<Uuid>> = HashMap::new();
        let mut unique_tasks: HashMap<Uuid, Task> = HashMap::new();

        for record in records {
            let date_key = record.scheduled_date;
            let task_id = record.task.id;

            ids_by_date
                .entry(date_key)
                .or_insert_with(Vec::new)
                .push(task_id);

            match unique_tasks.entry(task_id) {
                Entry::Occupied(_) => {}
                Entry::Vacant(entry) => {
                    entry.insert(record.task);
                }
            }
        }

        (ids_by_date, unique_tasks)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::{
        entities::TaskRow,
        infra::core::{AppError, DbError},
    };
    use sqlx::FromRow;

    #[derive(Debug)]
    pub struct DailyTaskRecord {
        pub scheduled_date: String,
        pub task: Task,
    }

    #[derive(Debug, FromRow)]
    struct TaskWithDateRow {
        scheduled_date: String,
        #[sqlx(flatten)]
        task: TaskRow,
    }

    /// 查询日期范围内的所有任务
    /// 排除 EXPIRE 类型且已过期的循环任务（recurrence_original_date < today）
    pub async fn find_tasks_for_range(
        pool: &sqlx::SqlitePool,
        start_date: &NaiveDate,
        end_date: &NaiveDate,
    ) -> AppResult<Vec<DailyTaskRecord>> {
        let start_str = time_utils::format_date_yyyy_mm_dd(start_date);
        let end_str = time_utils::format_date_yyyy_mm_dd(end_date);
        // 获取今天的日期用于过滤过期的循环任务
        let today_str = time_utils::format_date_yyyy_mm_dd(&chrono::Local::now().date_naive());

        let query = r#"
            SELECT
                ts.scheduled_date AS scheduled_date,
                t.*
            FROM task_schedules ts
            INNER JOIN tasks t ON t.id = ts.task_id
            WHERE ts.scheduled_date BETWEEN ? AND ?
              AND t.deleted_at IS NULL
              AND t.archived_at IS NULL
              AND NOT (
                  -- 排除 EXPIRE 类型且已过期的循环任务
                  t.recurrence_id IS NOT NULL
                  AND t.recurrence_original_date IS NOT NULL
                  AND t.recurrence_original_date < ?
                  AND EXISTS (
                      SELECT 1 FROM task_recurrences tr
                      WHERE tr.id = t.recurrence_id
                        AND tr.expiry_behavior = 'EXPIRE'
                  )
              )
            ORDER BY ts.scheduled_date ASC, t.created_at DESC
        "#;

        let rows = sqlx::query_as::<_, TaskWithDateRow>(query)
            .bind(&start_str)
            .bind(&end_str)
            .bind(&today_str)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        rows.into_iter()
            .map(|row| {
                let task = Task::try_from(row.task)
                    .map_err(|err| AppError::DatabaseError(DbError::QueryError(err)))?;
                Ok(DailyTaskRecord {
                    scheduled_date: row.scheduled_date,
                    task,
                })
            })
            .collect()
    }
}
