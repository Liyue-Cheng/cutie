use chrono::{DateTime, Utc};

/// 时钟接口定义
///
/// **预期行为简介:** 提供获取当前"时刻"的唯一、统一的途径
///
/// ## 已知适配器
/// - SystemClock: 生产适配器，其now_utc方法内部调用Utc::now()
/// - FixedClock: 测试适配器，其now_utc方法返回一个在创建时预设的固定时间戳
pub trait Clock: Send + Sync {
    /// 返回当前时间的UTC表示
    ///
    /// **预期行为简介:** 返回当前时间的UTC表示
    /// **输入输出规范:**
    /// - **前置条件:** 无
    /// - **后置条件:** 必须返回一个DateTime<Utc>类型的时间戳
    /// **边界情况:** 无
    /// **预期副作用:** 无纯粹的副作用，但其返回值是不确定的（在生产环境中）
    fn now_utc(&self) -> DateTime<Utc>;
}

/// 系统时钟适配器（生产用）
#[derive(Debug, Clone)]
pub struct SystemClock;

impl SystemClock {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// 固定时钟适配器（测试用）
#[derive(Debug, Clone)]
pub struct FixedClock {
    fixed_time: DateTime<Utc>,
}

impl FixedClock {
    /// 创建一个返回固定时间的时钟
    pub fn new(fixed_time: DateTime<Utc>) -> Self {
        Self { fixed_time }
    }

    /// 创建一个返回当前时间的固定时钟（用于测试快照）
    pub fn now() -> Self {
        Self::new(Utc::now())
    }

    /// 更新固定时间
    pub fn set_time(&mut self, new_time: DateTime<Utc>) {
        self.fixed_time = new_time;
    }

    /// 前进指定的秒数
    pub fn advance_seconds(&mut self, seconds: i64) {
        self.fixed_time = self.fixed_time + chrono::Duration::seconds(seconds);
    }

    /// 前进指定的分钟数
    pub fn advance_minutes(&mut self, minutes: i64) {
        self.fixed_time = self.fixed_time + chrono::Duration::minutes(minutes);
    }

    /// 前进指定的小时数
    pub fn advance_hours(&mut self, hours: i64) {
        self.fixed_time = self.fixed_time + chrono::Duration::hours(hours);
    }

    /// 前进指定的天数
    pub fn advance_days(&mut self, days: i64) {
        self.fixed_time = self.fixed_time + chrono::Duration::days(days);
    }
}

impl Clock for FixedClock {
    fn now_utc(&self) -> DateTime<Utc> {
        self.fixed_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_system_clock() {
        let clock = SystemClock::new();
        let now1 = clock.now_utc();

        // 等待一小段时间
        std::thread::sleep(std::time::Duration::from_millis(1));

        let now2 = clock.now_utc();

        // 系统时钟应该返回递增的时间
        assert!(now2 >= now1);
    }

    #[test]
    fn test_fixed_clock() {
        let fixed_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let clock = FixedClock::new(fixed_time);

        // 固定时钟应该总是返回相同的时间
        assert_eq!(clock.now_utc(), fixed_time);
        assert_eq!(clock.now_utc(), fixed_time);
    }

    #[test]
    fn test_fixed_clock_advance() {
        let initial_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let mut clock = FixedClock::new(initial_time);

        // 前进1小时
        clock.advance_hours(1);
        let expected_time = Utc.with_ymd_and_hms(2024, 1, 1, 13, 0, 0).unwrap();
        assert_eq!(clock.now_utc(), expected_time);

        // 前进1天
        clock.advance_days(1);
        let expected_time = Utc.with_ymd_and_hms(2024, 1, 2, 13, 0, 0).unwrap();
        assert_eq!(clock.now_utc(), expected_time);
    }

    #[test]
    fn test_fixed_clock_set_time() {
        let initial_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let mut clock = FixedClock::new(initial_time);

        let new_time = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();
        clock.set_time(new_time);

        assert_eq!(clock.now_utc(), new_time);
    }

    #[test]
    fn test_clock_trait_object() {
        let system_clock: Box<dyn Clock> = Box::new(SystemClock::new());
        let fixed_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let fixed_clock: Box<dyn Clock> = Box::new(FixedClock::new(fixed_time));

        // 测试trait object的使用
        let _now1 = system_clock.now_utc();
        let now2 = fixed_clock.now_utc();

        assert_eq!(now2, fixed_time);
    }
}
