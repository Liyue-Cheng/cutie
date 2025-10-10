/// TaskRecurrence 核心 CRUD 仓库
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::{
    entities::{TaskRecurrence, TaskRecurrenceRow, UpdateTaskRecurrenceRequest},
    shared::core::{AppError, AppResult, DbError},
};

pub struct TaskRecurrenceRepository;

impl TaskRecurrenceRepository {
    /// 在事务中查询循环规则
    pub async fn find_by_id_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        recurrence_id: Uuid,
    ) -> AppResult<Option<TaskRecurrence>> {
        let query = r#"
            SELECT id, template_id, rule, time_type, start_date, end_date, timezone, is_active,
                   created_at, updated_at
            FROM task_recurrences
            WHERE id = ?
        "#;

        let row = sqlx::query_as::<_, TaskRecurrenceRow>(query)
            .bind(recurrence_id.to_string())
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        match row {
            Some(r) => {
                let recurrence = TaskRecurrence::try_from(r)
                    .map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))?;
                Ok(Some(recurrence))
            }
            None => Ok(None),
        }
    }

    /// 非事务查询循环规则
    pub async fn find_by_id(
        pool: &SqlitePool,
        recurrence_id: Uuid,
    ) -> AppResult<Option<TaskRecurrence>> {
        let query = r#"
            SELECT id, template_id, rule, time_type, start_date, end_date, timezone, is_active,
                   created_at, updated_at
            FROM task_recurrences
            WHERE id = ?
        "#;

        let row = sqlx::query_as::<_, TaskRecurrenceRow>(query)
            .bind(recurrence_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        match row {
            Some(r) => {
                let recurrence = TaskRecurrence::try_from(r)
                    .map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))?;
                Ok(Some(recurrence))
            }
            None => Ok(None),
        }
    }

    /// 插入循环规则
    pub async fn insert_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        recurrence: &TaskRecurrence,
    ) -> AppResult<()> {
        let query = r#"
            INSERT INTO task_recurrences (
                id, template_id, rule, time_type, start_date, end_date, timezone, is_active,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(recurrence.id.to_string())
            .bind(recurrence.template_id.to_string())
            .bind(&recurrence.rule)
            .bind(recurrence.time_type.as_str())
            .bind(&recurrence.start_date)
            .bind(&recurrence.end_date)
            .bind(&recurrence.timezone)
            .bind(recurrence.is_active)
            .bind(recurrence.created_at)
            .bind(recurrence.updated_at)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 更新循环规则
    pub async fn update_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        recurrence_id: Uuid,
        request: &UpdateTaskRecurrenceRequest,
        updated_at: DateTime<Utc>,
    ) -> AppResult<TaskRecurrence> {
        // 构建动态 SET 子句
        let mut set_clauses = vec![];
        if request.template_id.is_some() {
            set_clauses.push("template_id = ?");
        }
        if request.rule.is_some() {
            set_clauses.push("rule = ?");
        }
        if request.time_type.is_some() {
            set_clauses.push("time_type = ?");
        }
        if request.start_date.is_some() {
            set_clauses.push("start_date = ?");
        }
        if request.end_date.is_some() {
            set_clauses.push("end_date = ?");
        }
        if request.timezone.is_some() {
            set_clauses.push("timezone = ?");
        }
        if request.is_active.is_some() {
            set_clauses.push("is_active = ?");
        }
        set_clauses.push("updated_at = ?");

        if set_clauses.is_empty() {
            // 如果没有任何更新字段，直接返回原对象
            return Self::find_by_id_in_tx(tx, recurrence_id)
                .await?
                .ok_or_else(|| AppError::NotFound {
                    entity_type: "TaskRecurrence".to_string(),
                    entity_id: recurrence_id.to_string(),
                });
        }

        let set_clause = set_clauses.join(", ");
        let query = format!("UPDATE task_recurrences SET {} WHERE id = ?", set_clause);

        let mut q = sqlx::query(&query);

        // 绑定参数
        if let Some(ref template_id) = request.template_id {
            q = q.bind(template_id.to_string());
        }
        if let Some(ref rule) = request.rule {
            q = q.bind(rule);
        }
        if let Some(ref time_type) = request.time_type {
            q = q.bind(time_type.as_str());
        }
        if let Some(ref start_date_opt) = request.start_date {
            let bind_val: Option<String> = start_date_opt.clone();
            q = q.bind(bind_val);
        }
        if let Some(ref end_date_opt) = request.end_date {
            let bind_val: Option<String> = end_date_opt.clone();
            q = q.bind(bind_val);
        }
        if let Some(ref timezone_opt) = request.timezone {
            let bind_val: Option<String> = timezone_opt.clone();
            q = q.bind(bind_val);
        }
        if let Some(is_active) = request.is_active {
            q = q.bind(is_active);
        }
        q = q.bind(updated_at);
        q = q.bind(recurrence_id.to_string());

        q.execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        // 查询更新后的循环规则
        Self::find_by_id_in_tx(tx, recurrence_id)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity_type: "TaskRecurrence".to_string(),
                entity_id: recurrence_id.to_string(),
            })
    }

    /// 软删除循环规则（实际上是标记为不激活）
    pub async fn deactivate_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        recurrence_id: Uuid,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()> {
        let query = r#"
            UPDATE task_recurrences
            SET is_active = 0, updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(updated_at)
            .bind(recurrence_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 物理删除循环规则
    pub async fn delete_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        recurrence_id: Uuid,
    ) -> AppResult<()> {
        let query = r#"
            DELETE FROM task_recurrences WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(recurrence_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 查询所有激活的循环规则
    pub async fn find_all_active(pool: &SqlitePool) -> AppResult<Vec<TaskRecurrence>> {
        let query = r#"
            SELECT id, template_id, rule, time_type, start_date, end_date, timezone, is_active,
                   created_at, updated_at
            FROM task_recurrences
            WHERE is_active = 1
            ORDER BY created_at DESC
        "#;

        let rows = sqlx::query_as::<_, TaskRecurrenceRow>(query)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let recurrences: Result<Vec<TaskRecurrence>, _> =
            rows.into_iter().map(TaskRecurrence::try_from).collect();

        recurrences.map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))
    }

    /// 查询某个模板的所有循环规则
    pub async fn find_by_template_id(
        pool: &SqlitePool,
        template_id: Uuid,
    ) -> AppResult<Vec<TaskRecurrence>> {
        let query = r#"
            SELECT id, template_id, rule, time_type, start_date, end_date, timezone, is_active,
                   created_at, updated_at
            FROM task_recurrences
            WHERE template_id = ?
            ORDER BY created_at DESC
        "#;

        let rows = sqlx::query_as::<_, TaskRecurrenceRow>(query)
            .bind(template_id.to_string())
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let recurrences: Result<Vec<TaskRecurrence>, _> =
            rows.into_iter().map(TaskRecurrence::try_from).collect();

        recurrences.map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))
    }

    /// 查询在某一天生效的所有循环规则
    pub async fn find_effective_for_date(
        pool: &SqlitePool,
        date: &str,
    ) -> AppResult<Vec<TaskRecurrence>> {
        let query = r#"
            SELECT id, template_id, rule, time_type, start_date, end_date, timezone, is_active,
                   created_at, updated_at
            FROM task_recurrences
            WHERE is_active = 1
              AND (start_date IS NULL OR start_date <= ?)
              AND (end_date IS NULL OR end_date >= ?)
            ORDER BY created_at ASC
        "#;

        let rows = sqlx::query_as::<_, TaskRecurrenceRow>(query)
            .bind(date)
            .bind(date)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let recurrences: Result<Vec<TaskRecurrence>, _> =
            rows.into_iter().map(TaskRecurrence::try_from).collect();

        recurrences.map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))
    }
}
