use std::sync::Arc;
// use std::collections::HashMap;
// use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{CreateTaskData, CreateTemplateData, CreationContext, TaskService, UpdateTemplateData};
use crate::common::error::{AppError, AppResult};
use crate::common::utils::template_utils::{create_standard_variables, render_template};
use crate::core::models::Template;
use crate::ports::{Clock, IdGenerator};
use crate::repositories::{TaskRepository, TemplateRepository};

/// 模板服务
///
/// **预期行为简介:** 封装所有与Template相关的业务逻辑，包括模板的CRUD操作和基于模板创建任务
pub struct TemplateService {
    /// 时钟服务
    clock: Arc<dyn Clock>,

    /// ID生成器
    id_generator: Arc<dyn IdGenerator>,

    /// 模板仓库
    template_repository: Arc<dyn TemplateRepository>,

    /// 任务仓库
    task_repository: Arc<dyn TaskRepository>,

    /// 任务服务（用于跨服务调用）
    task_service: Option<Arc<TaskService>>,
}

impl TemplateService {
    /// 创建新的模板服务
    pub fn new(
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
        template_repository: Arc<dyn TemplateRepository>,
        task_repository: Arc<dyn TaskRepository>,
    ) -> Self {
        Self {
            clock,
            id_generator,
            template_repository,
            task_repository,
            task_service: None,
        }
    }

    /// 设置任务服务引用（避免循环依赖）
    pub fn set_task_service(&mut self, task_service: Arc<TaskService>) {
        self.task_service = Some(task_service);
    }

    /// 创建模板
    pub async fn create_template(&self, data: CreateTemplateData) -> AppResult<Template> {
        // 验证输入
        if let Err(validation_errors) = data.validate() {
            return Err(AppError::ValidationFailed(validation_errors));
        }

        let mut tx = self.task_repository.begin_transaction().await?;

        // 检查名称是否可用
        let name_available = self
            .template_repository
            .is_name_available(&data.name, None)
            .await?;
        if !name_available {
            return Err(AppError::conflict(format!(
                "模板名称 '{}' 已存在",
                data.name
            )));
        }

        // 创建模板
        let template_id = self.id_generator.new_uuid();
        let now = self.clock.now_utc();

        let template = Template {
            id: template_id,
            name: data.name,
            title_template: data.title_template,
            glance_note_template: data.glance_note_template,
            detail_note_template: data.detail_note_template,
            estimated_duration_template: data.estimated_duration_template,
            subtasks_template: data.subtasks_template,
            area_id: data.area_id,
            created_at: now,
            updated_at: now,
            is_deleted: false,
        };

        let created_template = self.template_repository.create(&mut tx, &template).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(created_template)
    }

    /// 更新模板
    pub async fn update_template(
        &self,
        template_id: Uuid,
        updates: UpdateTemplateData,
    ) -> AppResult<Template> {
        let mut tx = self.task_repository.begin_transaction().await?;

        // 查找现有模板
        let mut current_template = self
            .template_repository
            .find_by_id(template_id)
            .await?
            .ok_or_else(|| AppError::not_found("Template", template_id.to_string()))?;

        // 验证名称唯一性（如果更新了名称）
        if let Some(ref new_name) = updates.name {
            let name_available = self
                .template_repository
                .is_name_available(new_name, Some(template_id))
                .await?;
            if !name_available {
                return Err(AppError::conflict(format!(
                    "模板名称 '{}' 已存在",
                    new_name
                )));
            }
        }

        // 应用更新
        if let Some(name) = updates.name {
            current_template.name = name;
        }
        if let Some(title_template) = updates.title_template {
            current_template.title_template = title_template;
        }
        if let Some(glance_note_template) = updates.glance_note_template {
            current_template.glance_note_template = glance_note_template;
        }
        if let Some(detail_note_template) = updates.detail_note_template {
            current_template.detail_note_template = detail_note_template;
        }
        if let Some(estimated_duration_template) = updates.estimated_duration_template {
            current_template.estimated_duration_template = estimated_duration_template;
        }
        if let Some(subtasks_template) = updates.subtasks_template {
            current_template.subtasks_template = subtasks_template;
        }
        if let Some(area_id) = updates.area_id {
            current_template.area_id = area_id;
        }

        current_template.updated_at = self.clock.now_utc();

        let updated_template = self
            .template_repository
            .update(&mut tx, &current_template)
            .await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(updated_template)
    }

    /// 基于模板创建任务
    ///
    /// **函数签名:** `pub async fn create_task_from_template(&self, template_id: Uuid, context: &CreationContext) -> Result<Task, AppError>`
    /// **预期行为简介:** 读取一个模板，解析变量，并据此创建一个具体的任务实例。
    /// **执行过程 (Process):**
    /// 1. **验证模板:** 调用 `TemplateRepository::find_by_id(template_id)`。若不存在，返回`NotFound`。
    /// 2. **构建`CreateTaskData`:**
    ///    a. 从模板中提取`title_template`, `notes_template`等。
    ///    b. 使用`template_utils::render_template`，用当前上下文（如日期）替换模板中的变量（如`{{date}}`）。
    ///    c. 将渲染后的结果填充到一个`CreateTaskData`结构体中。
    /// 3. **核心操作 (委托):** **调用 `self.task_service.create_in_context(create_task_data, context)`。**
    /// 4. **返回:** 直接返回`task_service`调用的结果。
    /// **预期副作用:** 与`TaskService::create_in_context`完全相同。
    pub async fn create_task_from_template(
        &self,
        template_id: Uuid,
        context: &CreationContext,
    ) -> AppResult<crate::core::models::Task> {
        // 1. 验证模板
        let template = self
            .template_repository
            .find_by_id(template_id)
            .await?
            .ok_or_else(|| AppError::not_found("Template", template_id.to_string()))?;

        // 2. 构建CreateTaskData
        let now = self.clock.now_utc();
        let variables = create_standard_variables(now);

        // a. 从模板中提取模板字符串
        // b. 使用template_utils::render_template替换变量
        let rendered_title = render_template(&template.title_template, &variables);
        let rendered_glance_note = template
            .glance_note_template
            .as_ref()
            .map(|template_str| render_template(template_str, &variables));
        let rendered_detail_note = template
            .detail_note_template
            .as_ref()
            .map(|template_str| render_template(template_str, &variables));

        // c. 填充到CreateTaskData结构体
        let create_task_data = CreateTaskData {
            title: rendered_title,
            glance_note: rendered_glance_note,
            detail_note: rendered_detail_note,
            estimated_duration: template.estimated_duration_template,
            subtasks: template.subtasks_template.clone(),
            area_id: template.area_id,
            due_date: None, // 模板不包含具体的截止日期
            due_date_type: None,
        };

        // 3. 核心操作（委托）
        if let Some(ref task_service) = self.task_service {
            task_service
                .create_in_context(create_task_data, context)
                .await
        } else {
            Err(AppError::UnspecifiedInternalError)
        }
    }

    /// 获取模板详情
    pub async fn get_template(&self, template_id: Uuid) -> AppResult<Option<Template>> {
        self.template_repository
            .find_by_id(template_id)
            .await
            .map_err(AppError::from)
    }

    /// 获取所有模板
    pub async fn get_all_templates(&self) -> AppResult<Vec<Template>> {
        self.template_repository
            .find_all()
            .await
            .map_err(AppError::from)
    }

    /// 搜索模板
    pub async fn search_templates(&self, name_query: &str) -> AppResult<Vec<Template>> {
        self.template_repository
            .search_by_name(name_query)
            .await
            .map_err(AppError::from)
    }

    /// 根据领域查找模板
    pub async fn get_templates_by_area(&self, area_id: Uuid) -> AppResult<Vec<Template>> {
        self.template_repository
            .find_by_area_id(area_id)
            .await
            .map_err(AppError::from)
    }

    /// 查找包含特定变量的模板
    pub async fn find_templates_with_variable(
        &self,
        variable_name: &str,
    ) -> AppResult<Vec<Template>> {
        self.template_repository
            .find_containing_variable(variable_name)
            .await
            .map_err(AppError::from)
    }

    /// 克隆模板
    pub async fn clone_template(&self, template_id: Uuid, new_name: String) -> AppResult<Template> {
        let mut tx = self.task_repository.begin_transaction().await?;

        let cloned_template = self
            .template_repository
            .clone_template(&mut tx, template_id, new_name)
            .await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(cloned_template)
    }

    /// 删除模板
    pub async fn delete_template(&self, template_id: Uuid) -> AppResult<()> {
        let mut tx = self.task_repository.begin_transaction().await?;

        self.template_repository
            .soft_delete(&mut tx, template_id)
            .await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(())
    }

    /// 批量删除模板
    pub async fn batch_delete_templates(&self, template_ids: Vec<Uuid>) -> AppResult<()> {
        let mut tx = self.task_repository.begin_transaction().await?;

        self.template_repository
            .batch_soft_delete(&mut tx, &template_ids)
            .await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(())
    }

    /// 导出模板
    pub async fn export_templates(
        &self,
        template_ids: Option<Vec<Uuid>>,
    ) -> AppResult<Vec<Template>> {
        let ids_ref = template_ids.as_ref().map(|v| v.as_slice());
        self.template_repository
            .export_templates(ids_ref)
            .await
            .map_err(AppError::from)
    }

    /// 获取模板统计
    pub async fn get_template_statistics(&self) -> AppResult<crate::repositories::TemplateCount> {
        self.template_repository
            .count_templates()
            .await
            .map_err(AppError::from)
    }

    /// 获取模板使用统计
    pub async fn get_usage_statistics(
        &self,
    ) -> AppResult<Vec<crate::repositories::TemplateUsageStats>> {
        self.template_repository
            .get_usage_stats()
            .await
            .map_err(AppError::from)
    }
}
