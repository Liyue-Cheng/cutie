/// OpenAI 配置（开发阶段写死）
/// ⚠️ 生产环境应使用环境变量或配置文件

/// OpenAI API Base URL
pub const OPENAI_BASE_URL: &str = "https://open.bigmodel.cn/api/paas/v4/";

/// OpenAI API Key
/// ⚠️ 替换为你的实际 API Key
pub const OPENAI_API_KEY: &str = "fa827c8d5c4ecea6e7f7214d866a9397.b4ueRkRYaqIGdgFE";

/// 默认使用的模型
pub const DEFAULT_MODEL: &str = "glm-4.5-flash";

/// 默认最大 Token 数
pub const DEFAULT_MAX_TOKENS: u32 = 2000;

/// 请求超时时间（秒）
pub const REQUEST_TIMEOUT_SECS: u64 = 60;
