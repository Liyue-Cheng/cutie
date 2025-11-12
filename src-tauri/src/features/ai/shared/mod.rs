pub mod auto_classify;
pub mod client;
pub mod config;
pub mod settings;

pub use auto_classify::{classify_task_area, AreaOption};
pub use client::OpenAIClient;
pub use settings::{
    load_conversation_model_config, load_quick_model_config, AiModelConfig,
    AI_CONVERSATION_API_BASE_URL, AI_CONVERSATION_API_KEY, AI_CONVERSATION_MODEL,
    AI_QUICK_API_BASE_URL, AI_QUICK_API_KEY, AI_QUICK_MODEL,
};
