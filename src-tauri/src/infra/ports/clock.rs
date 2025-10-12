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

    /// 设置新的固定时间
    pub fn set_time(&mut self, new_time: DateTime<Utc>) {
        self.fixed_time = new_time;
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
        std::thread::sleep(std::time::Duration::from_millis(10));
        let now2 = clock.now_utc();
        assert!(now2 >= now1);
    }

    #[test]
    fn test_fixed_clock() {
        let fixed_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let clock = FixedClock::new(fixed_time);
        assert_eq!(clock.now_utc(), fixed_time);

        // 多次调用应该返回相同的时间
        assert_eq!(clock.now_utc(), clock.now_utc());
    }

    #[test]
    fn test_fixed_clock_set_time() {
        let time1 = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let time2 = Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap();

        let mut clock = FixedClock::new(time1);
        assert_eq!(clock.now_utc(), time1);

        clock.set_time(time2);
        assert_eq!(clock.now_utc(), time2);
    }
}
