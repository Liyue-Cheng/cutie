pub mod auto_classify;
pub mod client;
pub mod config;

pub use auto_classify::{classify_task_area, AreaOption};
pub use client::OpenAIClient;
