/// 删除模板 - 单文件组件

// ==================== CABC 文档 ====================
/*
CABC for `delete_template`

## 1. 端点签名
DELETE /api/templates/:id

## 2. 预期行为简介

### 2.1 用户故事
> 作为用户,我想要删除不再使用的模板

### 2.2 核心业务逻辑
软删除模板(设置 is_deleted = TRUE)

## 3. 输入输出规范

### 3.1 请求 (Request)
无请求体

### 3.2 响应 (Responses)
**204 No Content:**
删除成功

**404 Not Found:**
模板不存在

## 4. 验证规则
无

## 5. 业务逻辑详解
1. 开启事务
2. 检查模板是否存在
3. 软删除模板
4. 提交事务
5. 返回 204

## 6. 边界情况
- 模板不存在: 返回 404
- 模板已删除: 返回 404

## 7. 预期副作用
### 数据库操作:
- UPDATE: 设置 is_deleted = TRUE
- 事务边界: begin() → commit()

### SSE 事件:
- template.deleted

## 8. 契约
### 前置条件:
- 模板存在且未删除

### 后置条件:
- 模板已软删除
*/

// ==================== 依赖引入 ====================
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    features::shared::TransactionHelper,
    shared::core::{AppError, AppResult},
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, id: Uuid) -> AppResult<()> {
        // 1. 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 2. 软删除模板
        database::soft_delete_in_tx(&mut tx, id).await?;

        // 3. 提交事务
        TransactionHelper::commit(tx).await?;

        Ok(())
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use sqlx::{Sqlite, Transaction};

    pub async fn soft_delete_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        id: Uuid,
    ) -> AppResult<()> {
        let query = r#"
            UPDATE templates
            SET is_deleted = TRUE
            WHERE id = ? AND is_deleted = FALSE
        "#;

        let result = sqlx::query(query)
            .bind(id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound {
                entity_type: "Template".to_string(),
                entity_id: id.to_string(),
            });
        }

        Ok(())
    }
}

