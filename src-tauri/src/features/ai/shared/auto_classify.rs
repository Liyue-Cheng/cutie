/// AI 自动分类功能
///
/// 异步调用 LLM 为任务选择合适的 Area
use crate::shared::core::error::AppResult;
use uuid::Uuid;

use super::client::OpenAIClient;

/// Area 摘要（用于 AI 分类）
#[derive(Debug, Clone)]
pub struct AreaOption {
    pub id: Uuid,
    pub name: String,
}

/// AI 自动分类任务到 Area
///
/// 返回 None 表示 AI 认为不需要分类，或者超时/失败
pub async fn classify_task_area(
    task_title: &str,
    available_areas: &[AreaOption],
) -> AppResult<Option<Uuid>> {
    if available_areas.is_empty() {
        return Ok(None);
    }

    // 构建 prompt
    let areas_list = available_areas
        .iter()
        .map(|area| format!("- {}", area.name))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        r#"你是一个任务分类助手。请根据任务标题，从给定的分类列表中选择最合适的一个。

任务标题: "{}"

可用分类:
{}

请仔细分析任务标题，选择最恰当的分类。如果没有合适的分类，输出 <none>。

只输出分类的标题（完全一致，不要有任何修改），不要有任何其他内容。如果不分类，只输出 <none>。

示例输出:
- 如果选择第一个分类: 输出对应的分类标题（完全一致）
- 如果都不合适: <none>"#,
        task_title, areas_list
    );

    let client = OpenAIClient::new();

    // 构建消息
    let messages = vec![crate::entities::ai::ChatMessage {
        role: "user".to_string(),
        text: prompt,
        images: vec![],
    }];

    // 开始计时
    let start_time = std::time::Instant::now();

    tracing::info!(
        target: "AI:CLASSIFY",
        task_title = %task_title,
        areas_count = available_areas.len(),
        "Sending classification request to AI"
    );

    // 设置 5 秒超时
    // 使用 chat_no_thinking 禁用思维链，确保只返回分类结果
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        client.chat_no_thinking(messages, None, Some(100)), // 只需要返回一个标题，100 tokens 足够
    )
    .await;

    let elapsed_ms = start_time.elapsed().as_millis();

    match result {
        Ok(Ok(response)) => {
            // 解析 AI 响应
            let reply = response.reply.trim();

            tracing::info!(
                target: "AI:CLASSIFY",
                reply = %reply,
                response_time_ms = elapsed_ms,
                model = %response.model,
                tokens_used = response.usage.total_tokens,
                "AI classification response received"
            );

            // 检查是否是 <none>
            if reply.contains("<none>") || reply.to_lowercase().contains("none") {
                tracing::info!(
                    target: "AI:CLASSIFY",
                    response_time_ms = elapsed_ms,
                    "AI decided not to classify (returned <none>)"
                );
                return Ok(None);
            }

            // 尝试根据标题查找 Area
            if let Some(area) = available_areas.iter().find(|a| a.name == reply) {
                tracing::info!(
                    target: "AI:CLASSIFY",
                    area_id = %area.id,
                    area_name = %area.name,
                    response_time_ms = elapsed_ms,
                    "✅ Task classified successfully"
                );
                Ok(Some(area.id))
            } else {
                // 模糊匹配：AI 可能输出了相似的名字
                let fuzzy_match = available_areas.iter().find(|a| {
                    // 忽略大小写和首尾空格
                    let normalized_ai = reply.trim().to_lowercase();
                    let normalized_area = a.name.trim().to_lowercase();
                    normalized_ai == normalized_area
                });

                if let Some(area) = fuzzy_match {
                    tracing::info!(
                        target: "AI:CLASSIFY",
                        area_id = %area.id,
                        area_name = %area.name,
                        ai_output = %reply,
                        response_time_ms = elapsed_ms,
                        "✅ Task classified successfully (fuzzy match)"
                    );
                    Ok(Some(area.id))
                } else {
                    tracing::warn!(
                        target: "AI:CLASSIFY",
                        reply = %reply,
                        response_time_ms = elapsed_ms,
                        available_areas = ?available_areas.iter().map(|a| &a.name).collect::<Vec<_>>(),
                        "AI returned area name not in available list"
                    );
                    Ok(None)
                }
            }
        }
        Ok(Err(e)) => {
            tracing::error!(
                target: "AI:CLASSIFY",
                error = %e,
                response_time_ms = elapsed_ms,
                "❌ AI classification failed"
            );
            Ok(None) // 失败不影响任务创建，返回 None
        }
        Err(_) => {
            tracing::warn!(
                target: "AI:CLASSIFY",
                response_time_ms = elapsed_ms,
                "⏱️ AI classification timeout (5s)"
            );
            Ok(None) // 超时返回 None
        }
    }
}
