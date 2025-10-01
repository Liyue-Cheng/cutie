/// 测试辅助函数
use chrono::SecondsFormat;
use uuid::Uuid;

/// 生成测试用的 ISO 8601 时间字符串（使用 Z 格式匹配后端）
pub fn test_time(offset_hours: i64) -> String {
    let now = chrono::Utc::now();
    let target = now + chrono::Duration::hours(offset_hours);
    target.to_rfc3339_opts(SecondsFormat::AutoSi, true) // true = 使用 Z 而不是 +00:00
}

/// 生成今天的开始时间
pub fn today_start() -> String {
    let now = chrono::Utc::now();
    let date = now.date_naive();
    date.and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

/// 生成昨天的时间
pub fn yesterday() -> String {
    let now = chrono::Utc::now();
    let yesterday = now - chrono::Duration::days(1);
    yesterday.to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

/// 生成明天的时间
pub fn tomorrow() -> String {
    let now = chrono::Utc::now();
    let tomorrow = now + chrono::Duration::days(1);
    tomorrow.to_rfc3339_opts(SecondsFormat::AutoSi, true)
}

/// 解析 UUID（辅助函数）
pub fn parse_uuid(id: &str) -> Uuid {
    Uuid::parse_str(id).expect("Invalid UUID")
}

/// 提取响应中的 ID
pub fn extract_id(value: &serde_json::Value) -> Uuid {
    let id_str = value["id"].as_str().expect("Missing id field");
    parse_uuid(id_str)
}
