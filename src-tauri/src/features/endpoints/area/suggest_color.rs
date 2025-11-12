/// Area AI 自动染色端点 - 单文件组件
/// POST /api/areas/suggest-color
// ==================== CABC 文档 ====================
/*
1. 端点签名：POST /api/areas/suggest-color
2. 用户故事：作为用户，我想要 AI 根据 Area 名称自动推荐合适的颜色
3. 输入输出：
   - 输入：{ area_name: string }
   - 输出：{ suggested_color: string }
4. 验证规则：
   - area_name 不能为空
5. 业务逻辑：
   - 调用 AI 模型（使用"快速模型"配置）
   - 根据 Area 名称语义推荐合适的颜色
   - 返回 HEX 格式的颜色值（如 #4A90E2）
6. 边界情况：
   - AI 配置未设置：返回错误提示
   - AI 调用失败：返回错误信息
   - 返回的颜色格式不正确：使用默认颜色
7. 副作用：无（只读操作）
8. 契约：
   - 前置条件：用户已配置 AI 快速模型
   - 后置条件：返回有效的 HEX 颜色值
*/
// ==================== 依赖引入 ====================
use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::features::ai::shared::{load_quick_model_config, OpenAIClient};
use crate::infra::core::{AppError, AppResult};
use crate::infra::http::error_handler::success_response;
use crate::startup::AppState;

// ==================== 请求/响应结构 ====================
#[derive(Deserialize)]
pub struct SuggestColorRequest {
    pub area_name: String,
}

#[derive(Serialize)]
pub struct SuggestColorResponse {
    pub suggested_color: String,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<SuggestColorRequest>,
) -> impl IntoResponse {
    match logic::execute(&app_state, request).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        request: SuggestColorRequest,
    ) -> AppResult<SuggestColorResponse> {
        // 1. 验证输入
        let area_name = request.area_name.trim();
        if area_name.is_empty() {
            return Err(AppError::validation_error(
                "area_name",
                "Area 名称不能为空",
                "EMPTY_AREA_NAME",
            ));
        }

        // 2. 加载 AI 快速模型配置
        let model_config = load_quick_model_config(app_state.db_pool())
            .await
            .map_err(|e| {
                AppError::configuration_error(format!(
                    "无法加载 AI 配置: {}. 请在设置中配置 AI 快速模型",
                    e
                ))
            })?
            .require_complete("AI 快速模型")?;

        // 3. 创建 AI 客户端
        let client = OpenAIClient::new(model_config);

        // 4. 构建 AI 提示词
        let system_prompt = r#"你是一个色彩设计专家。根据用户提供的 Area（领域/分类）名称，推荐一个合适的颜色。

要求：
1. 只返回一个 HEX 格式的颜色值（如 #4A90E2）
2. 颜色应该符合该领域的语义和情感色彩
3. 颜色应该适合在浅色背景上显示，具有良好的对比度
4. 不要返回任何解释，只返回颜色值"#;

        let user_message = crate::entities::ai::ChatMessage {
            role: "user".to_string(),
            text: format!("Area 名称: {}", area_name),
            images: vec![],
        };

        // 5. 调用 AI
        let ai_response = client
            .chat(
                vec![user_message],
                Some(system_prompt.to_string()),
                Some(50),
            )
            .await
            .map_err(|e| {
                AppError::external_dependency_failed("OpenAI", format!("AI 调用失败: {}", e))
            })?;

        // 6. 解析颜色值
        let suggested_color = parse_color_from_response(&ai_response.reply)?;

        Ok(SuggestColorResponse { suggested_color })
    }
}

/// 从 AI 响应中解析颜色值
fn parse_color_from_response(response: &str) -> AppResult<String> {
    // 提取 HEX 颜色值（支持 #RRGGBB 格式）
    let color_regex = regex::Regex::new(r"#[0-9A-Fa-f]{6}").unwrap();

    if let Some(matched) = color_regex.find(response) {
        Ok(matched.as_str().to_string())
    } else {
        // 如果没有找到有效颜色，返回默认颜色
        eprintln!("⚠️  AI 返回的颜色格式不正确: {}，使用默认颜色", response);
        Ok("#4A90E2".to_string()) // 默认蓝色
    }
}

#[cfg(test)]
mod tests {
    use super::parse_color_from_response;

    #[test]
    fn test_parse_color_from_response() {
        assert_eq!(parse_color_from_response("#4A90E2").unwrap(), "#4A90E2");
        assert_eq!(
            parse_color_from_response("推荐颜色是 #56C568").unwrap(),
            "#56C568"
        );
        assert_eq!(
            parse_color_from_response("这是一个无效的响应").unwrap(),
            "#4A90E2"
        ); // 默认颜色
    }
}
