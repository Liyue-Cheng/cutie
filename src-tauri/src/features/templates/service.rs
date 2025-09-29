/// 模板业务逻辑层

use chrono::Utc;
use uuid::Uuid;

use crate::shared::{
    core::{
        create_standard_variables, render_template, AppError, AppResult, Task, Template,
    },
    database::TemplateRepository,
};

use super::payloads::{
    CreateTaskFromTemplatePayload, CreateTemplatePayload, TemplateStatsResponse,
    UpdateTemplatePayload, VariableUsage,
};

/// 模板服务
pub struct TemplateService<R: TemplateRepository> {
    repository: R,
}

impl<R: TemplateRepository> TemplateService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// 创建新模板
    pub async fn create_template(&self, payload: CreateTemplatePayload) -> AppResult<Template> {
        let now = Utc::now();
        let template_id = Uuid::new_v4();

        let mut template = Template::new(template_id, payload.name, payload.title_template, now);

        // 设置可选字段
        if let Some(glance_note) = payload.glance_note_template {
            template.glance_note_template = Some(glance_note);
        }
        if let Some(detail_note) = payload.detail_note_template {
            template.detail_note_template = Some(detail_note);
        }
        if let Some(duration) = payload.estimated_duration_template {
            template.estimated_duration_template = Some(duration);
        }
        if let Some(subtasks) = payload.subtasks_template {
            template.subtasks_template = Some(subtasks);
        }
        if let Some(area_id) = payload.area_id {
            template.area_id = Some(area_id);
        }

        self.repository.create(&template).await
    }

    /// 根据ID获取模板
    pub async fn get_template(&self, id: Uuid) -> AppResult<Template> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("Template", id.to_string()))
    }

    /// 更新模板
    pub async fn update_template(
        &self,
        id: Uuid,
        payload: UpdateTemplatePayload,
    ) -> AppResult<Template> {
        let mut template = self.get_template(id).await?;
        let now = Utc::now();

        // 更新字段
        if let Some(name) = payload.name {
            template.name = name;
        }
        if let Some(title_template) = payload.title_template {
            template.title_template = title_template;
        }
        if let Some(glance_note_template) = payload.glance_note_template {
            template.glance_note_template = glance_note_template;
        }
        if let Some(detail_note_template) = payload.detail_note_template {
            template.detail_note_template = detail_note_template;
        }
        if let Some(estimated_duration_template) = payload.estimated_duration_template {
            template.estimated_duration_template = estimated_duration_template;
        }
        if let Some(subtasks_template) = payload.subtasks_template {
            template.subtasks_template = subtasks_template;
        }
        if let Some(area_id) = payload.area_id {
            template.area_id = area_id;
        }

        template.updated_at = now;
        self.repository.update(&template).await
    }

    /// 删除模板
    pub async fn delete_template(&self, id: Uuid) -> AppResult<()> {
        // 检查模板是否存在
        self.get_template(id).await?;

        self.repository.delete(id).await
    }

    /// 克隆模板
    pub async fn clone_template(&self, id: Uuid, new_name: String) -> AppResult<Template> {
        let original_template = self.get_template(id).await?;
        let now = Utc::now();
        let new_id = Uuid::new_v4();

        let mut cloned_template = Template::new(new_id, new_name, original_template.title_template, now);

        // 复制所有字段
        cloned_template.glance_note_template = original_template.glance_note_template;
        cloned_template.detail_note_template = original_template.detail_note_template;
        cloned_template.estimated_duration_template = original_template.estimated_duration_template;
        cloned_template.subtasks_template = original_template.subtasks_template;
        cloned_template.area_id = original_template.area_id;

        self.repository.create(&cloned_template).await
    }

    /// 基于模板创建任务
    pub async fn create_task_from_template(
        &self,
        template_id: Uuid,
        payload: CreateTaskFromTemplatePayload,
    ) -> AppResult<Task> {
        let template = self.get_template(template_id).await?;
        let now = Utc::now();

        // 创建变量映射
        let mut variables = create_standard_variables(now);

        // 添加自定义变量
        if let Some(custom_vars) = payload.custom_variables {
            variables.extend(custom_vars);
        }

        // 渲染模板
        let title = render_template(&template.title_template, &variables);

        let glance_note = template
            .glance_note_template
            .as_ref()
            .map(|t| render_template(t, &variables));

        let detail_note = template
            .detail_note_template
            .as_ref()
            .map(|t| render_template(t, &variables));

        // 创建任务
        let task_id = Uuid::new_v4();
        let mut task = Task::new(task_id, title, now);

        task.glance_note = glance_note;
        task.detail_note = detail_note;
        task.estimated_duration = template.estimated_duration_template;
        task.subtasks = template.subtasks_template.clone();
        task.area_id = template.area_id;

        // TODO: 这里应该调用任务服务来创建任务，包括排序逻辑
        // 为了简化，这里直接返回任务对象
        Ok(task)
    }

    /// 搜索模板
    pub async fn search_templates(
        &self,
        query: Option<String>,
        area_id: Option<Uuid>,
        variable: Option<String>,
        limit: Option<usize>,
    ) -> AppResult<Vec<Template>> {
        let mut templates = if let Some(q) = query {
            // 按名称搜索
            self.repository.search_by_name(&q).await?
        } else if let Some(area_id) = area_id {
            // 按领域查找
            self.repository.find_by_area_id(area_id).await?
        } else if let Some(variable) = variable {
            // 按变量查找
            self.repository.find_with_variable(&variable).await?
        } else {
            // 获取所有模板
            self.repository.find_all().await?
        };

        // 应用限制
        if let Some(limit) = limit {
            templates.truncate(limit);
        }

        Ok(templates)
    }

    /// 获取模板统计
    pub async fn get_template_stats(&self) -> AppResult<TemplateStatsResponse> {
        let all_templates = self.repository.find_all().await?;

        let total_count = all_templates.len() as i64;

        // 按领域分组统计
        let mut by_area = std::collections::HashMap::new();
        for template in &all_templates {
            let area_key = template
                .area_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "无领域".to_string());
            *by_area.entry(area_key).or_insert(0) += 1;
        }

        // 统计包含变量的模板
        let templates_with_variables = all_templates
            .iter()
            .filter(|t| t.has_variables())
            .count() as i64;

        // 统计变量使用情况
        let mut variable_usage = std::collections::HashMap::new();
        for template in &all_templates {
            let variables = template.get_variables();
            for var in variables {
                let entry = variable_usage.entry(var).or_insert((0, std::collections::HashSet::new()));
                entry.0 += 1; // 使用次数
                entry.1.insert(template.id); // 使用该变量的模板集合
            }
        }

        let most_used_variables: Vec<VariableUsage> = variable_usage
            .into_iter()
            .map(|(var_name, (usage_count, template_set))| VariableUsage {
                variable_name: var_name,
                usage_count,
                template_count: template_set.len() as i64,
            })
            .collect::<Vec<_>>();

        // 计算平均复杂度（基于变量数量）
        let total_variables: usize = all_templates
            .iter()
            .map(|t| t.get_variables().len())
            .sum();

        let avg_complexity = if total_count > 0 {
            total_variables as f64 / total_count as f64
        } else {
            0.0
        };

        Ok(TemplateStatsResponse {
            total_count,
            by_area,
            templates_with_variables,
            most_used_variables,
            avg_complexity,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        features::templates::repository::SqlxTemplateRepository,
        shared::{core::ContextType, database::connection::create_test_database},
    };

    #[tokio::test]
    async fn test_create_template() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxTemplateRepository::new(pool);
        let service = TemplateService::new(repository);

        let payload = CreateTemplatePayload {
            name: "Daily Standup".to_string(),
            title_template: "Daily Standup - {{date}}".to_string(),
            glance_note_template: Some("Standup meeting for {{date}}".to_string()),
            detail_note_template: None,
            estimated_duration_template: Some(15),
            subtasks_template: None,
            area_id: None,
        };

        let template = service.create_template(payload).await.unwrap();
        assert_eq!(template.name, "Daily Standup");
        assert!(template.has_variables());
    }

    #[tokio::test]
    async fn test_create_task_from_template() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxTemplateRepository::new(pool);
        let service = TemplateService::new(repository);

        // 创建模板
        let template = Template::new(
            Uuid::new_v4(),
            "Test Template".to_string(),
            "Task for {{date}}".to_string(),
            Utc::now(),
        );
        let created_template = service.repository.create(&template).await.unwrap();

        // 基于模板创建任务
        let payload = CreateTaskFromTemplatePayload {
            context: super::payloads::CreationContextPayload {
                context_type: ContextType::Misc,
                context_id: "floating".to_string(),
            },
            custom_variables: None,
        };

        let task = service
            .create_task_from_template(created_template.id, payload)
            .await
            .unwrap();

        // 验证任务标题已被渲染
        assert!(task.title.contains("Task for"));
        assert!(!task.title.contains("{{date}}"));
    }

    #[tokio::test]
    async fn test_clone_template() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxTemplateRepository::new(pool);
        let service = TemplateService::new(repository);

        // 创建原始模板
        let original = Template::new(
            Uuid::new_v4(),
            "Original Template".to_string(),
            "Original {{date}}".to_string(),
            Utc::now(),
        );
        let created_original = service.repository.create(&original).await.unwrap();

        // 克隆模板
        let cloned = service
            .clone_template(created_original.id, "Cloned Template".to_string())
            .await
            .unwrap();

        assert_eq!(cloned.name, "Cloned Template");
        assert_eq!(cloned.title_template, original.title_template);
        assert_ne!(cloned.id, original.id);
    }
}
