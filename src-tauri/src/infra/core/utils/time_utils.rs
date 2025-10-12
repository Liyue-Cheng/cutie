#![allow(unused)]
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, TimeZone, Timelike, Utc};

/// 时间工具模块
///
/// **预期行为:** 封装所有与DateTime<Utc>相关的、不依赖当前系统时间的计算
/// **后置条件:** 对于给定的DateTime输入，总是返回一个可预测的、正确计算后的DateTime输出
/// **边界情况:** 必须能正确处理闰年、以及月份边界
///
/// **核心原则（TIME_REFACTOR_RFC）**:
/// - 瞬时刻（Instant）：使用 DateTime<Utc>，存储与传输为 RFC3339（含 Z）
/// - 日历日期（Calendar Date）：使用 NaiveDate，存储与传输为 YYYY-MM-DD 字符串
/// - 禁止 UTC⇄本地时区的往返转换（已移除相关函数）

// ==================== 日历日期解析与格式化 ====================

/// 解析 YYYY-MM-DD 格式的日期字符串
///
/// **预期行为**：将日期字符串解析为 NaiveDate
/// **用途**：端点参数验证、数据库查询准备
///
/// ## 示例
/// ```ignore
/// use crate::infra::core::utils::time_utils::parse_date_yyyy_mm_dd;
/// let date = parse_date_yyyy_mm_dd("2025-10-08").unwrap();
/// assert_eq!(date.to_string(), "2025-10-08");
/// ```
pub fn parse_date_yyyy_mm_dd(date_str: &str) -> Result<NaiveDate, chrono::ParseError> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
}

/// 格式化 NaiveDate 为 YYYY-MM-DD 字符串
///
/// **预期行为**：将 NaiveDate 转换为标准日期字符串
/// **用途**：数据库绑定、API 响应构建
///
/// ## 示例
/// ```ignore
/// use chrono::NaiveDate;
/// use crate::infra::core::utils::time_utils::format_date_yyyy_mm_dd;
/// let date = NaiveDate::from_ymd_opt(2025, 10, 8).unwrap();
/// assert_eq!(format_date_yyyy_mm_dd(&date), "2025-10-08");
/// ```
pub fn format_date_yyyy_mm_dd(date: &NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// 将日期时间规范化为当日零点（UTC）
///
/// **预期行为:** 将任意时间戳转换为该日期的零点零分零秒
/// **后置条件:** 返回的DateTime的时分秒毫秒均为0
pub fn normalize_to_day_start(dt: DateTime<Utc>) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0)
        .single()
        .unwrap_or(dt) // 如果转换失败，返回原始时间
}

/// 获取当月第一天的零点
///
/// **预期行为:** 返回给定日期所在月份的第一天零点
pub fn get_month_start(dt: DateTime<Utc>) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(dt.year(), dt.month(), 1, 0, 0, 0)
        .single()
        .unwrap_or(dt)
}

/// 获取当月最后一天的23:59:59
///
/// **预期行为:** 返回给定日期所在月份的最后一天的最后一秒
pub fn get_month_end(dt: DateTime<Utc>) -> DateTime<Utc> {
    let next_month_start = if dt.month() == 12 {
        Utc.with_ymd_and_hms(dt.year() + 1, 1, 1, 0, 0, 0).single()
    } else {
        Utc.with_ymd_and_hms(dt.year(), dt.month() + 1, 1, 0, 0, 0)
            .single()
    };

    match next_month_start {
        Some(next_start) => next_start - Duration::seconds(1),
        None => dt, // 如果计算失败，返回原始时间
    }
}

/// 获取当周第一天（周一）的零点
///
/// **预期行为:** 返回给定日期所在周的周一零点
pub fn get_week_start(dt: DateTime<Utc>) -> DateTime<Utc> {
    let weekday = dt.weekday().num_days_from_monday();
    normalize_to_day_start(dt - Duration::days(weekday as i64))
}

/// 获取当周最后一天（周日）的23:59:59
///
/// **预期行为:** 返回给定日期所在周的周日最后一秒
pub fn get_week_end(dt: DateTime<Utc>) -> DateTime<Utc> {
    let weekday = dt.weekday().num_days_from_monday();
    let sunday = dt + Duration::days(6 - weekday as i64);
    normalize_to_day_start(sunday) + Duration::days(1) - Duration::seconds(1)
}

/// 添加工作日（跳过周末）
///
/// **预期行为:** 从给定日期开始，添加指定数量的工作日（周一到周五）
/// **边界情况:** 如果起始日期是周末，从下一个周一开始计算
pub fn add_business_days(dt: DateTime<Utc>, days: i32) -> DateTime<Utc> {
    let mut current = normalize_to_day_start(dt);
    let mut remaining_days = days.abs();
    let direction = if days >= 0 { 1 } else { -1 };

    while remaining_days > 0 {
        current = current + Duration::days(direction);
        let weekday = current.weekday().num_days_from_monday();

        // 周一到周五是工作日 (0-4)
        if weekday < 5 {
            remaining_days -= 1;
        }
    }

    current
}

/// 计算两个日期之间的工作日数量
///
/// **预期行为:** 计算start_date到end_date之间的工作日数量（不包括end_date）
pub fn count_business_days_between(start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> i32 {
    let start = normalize_to_day_start(start_date);
    let end = normalize_to_day_start(end_date);

    if start >= end {
        return 0;
    }

    let mut count = 0;
    let mut current = start;

    while current < end {
        let weekday = current.weekday().num_days_from_monday();
        if weekday < 5 {
            // 周一到周五
            count += 1;
        }
        current = current + Duration::days(1);
    }

    count
}

/// 检查给定年份是否为闰年
///
/// **预期行为:** 根据公历闰年规则判断年份
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// 获取Unix时间戳（秒）
///
/// **预期行为:** 将DateTime<Utc>转换为Unix时间戳
pub fn to_unix_timestamp(dt: DateTime<Utc>) -> i64 {
    dt.timestamp()
}

/// 从Unix时间戳创建DateTime
///
/// **预期行为:** 将Unix时间戳转换为DateTime<Utc>
pub fn from_unix_timestamp(timestamp: i64) -> Option<DateTime<Utc>> {
    Utc.timestamp_opt(timestamp, 0).single()
}

/// 格式化日期为ISO 8601字符串
///
/// **预期行为:** 将DateTime转换为标准ISO格式字符串
pub fn to_iso_string(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

/// 从ISO 8601字符串解析日期
///
/// **预期行为:** 将ISO格式字符串转换为DateTime<Utc>
pub fn from_iso_string(iso_string: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(iso_string).map(|dt| dt.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_to_day_start() {
        let dt = Utc.with_ymd_and_hms(2024, 10, 1, 15, 30, 45).unwrap();
        let normalized = normalize_to_day_start(dt);

        assert_eq!(normalized.year(), 2024);
        assert_eq!(normalized.month(), 10);
        assert_eq!(normalized.day(), 1);
        assert_eq!(normalized.hour(), 0);
        assert_eq!(normalized.minute(), 0);
        assert_eq!(normalized.second(), 0);
    }

    #[test]
    fn test_get_month_start() {
        let dt = Utc.with_ymd_and_hms(2024, 10, 15, 12, 0, 0).unwrap();
        let month_start = get_month_start(dt);

        assert_eq!(month_start.year(), 2024);
        assert_eq!(month_start.month(), 10);
        assert_eq!(month_start.day(), 1);
        assert_eq!(month_start.hour(), 0);
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(1900));
    }

    #[test]
    fn test_unix_timestamp_conversion() {
        let dt = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let timestamp = to_unix_timestamp(dt);
        let converted_back = from_unix_timestamp(timestamp).unwrap();

        assert_eq!(dt, converted_back);
    }

    #[test]
    fn test_add_business_days() {
        // 从周一开始，添加5个工作日，应该到下周一
        let monday = Utc.with_ymd_and_hms(2024, 10, 7, 0, 0, 0).unwrap(); // 假设是周一
        let result = add_business_days(monday, 5);

        // 应该跳过周末，到达下周一
        let expected_weekday = result.weekday().num_days_from_monday();
        assert_eq!(expected_weekday, 0); // 周一
    }
}
