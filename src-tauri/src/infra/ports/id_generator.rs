use uuid::Uuid;

/// ID生成器接口定义
///
/// **预期行为简介:** 提供生成全局唯一标识符（UUID）的唯一途径
///
/// ## 已知适配器
/// - UuidV4Generator: 生产适配器，其new_uuid方法内部调用Uuid::new_v4()
/// - SequentialIdGenerator: 测试适配器，其new_uuid方法按预设顺序返回一系列固定的UUID值
pub trait IdGenerator: Send + Sync {
    /// 返回一个新的、符合UUID规范的唯一标识符
    ///
    /// **预期行为简介:** 返回一个新的、符合UUID规范的唯一标识符
    /// **输入输出规范:**
    /// - **前置条件:** 无
    /// - **后置条件:** 必须返回一个Uuid类型的值。在生产环境中，此值在理论上应是全局唯一的
    /// **边界情况:** 无
    /// **预期副作用:** 无纯粹的副作用，但其返回值是不确定的（在生产环境中）
    fn new_uuid(&self) -> Uuid;
}

/// UUID V4生成器适配器（生产用）
#[derive(Debug, Clone)]
pub struct UuidV4Generator;

impl UuidV4Generator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UuidV4Generator {
    fn default() -> Self {
        Self::new()
    }
}

impl IdGenerator for UuidV4Generator {
    fn new_uuid(&self) -> Uuid {
        Uuid::new_v4()
    }
}

/// 顺序ID生成器适配器（测试用）
#[derive(Debug, Clone)]
pub struct SequentialIdGenerator {
    uuids: Vec<Uuid>,
    current_index: std::sync::Arc<std::sync::Mutex<usize>>,
}

impl SequentialIdGenerator {
    /// 创建一个顺序ID生成器
    pub fn new(uuids: Vec<Uuid>) -> Self {
        Self {
            uuids,
            current_index: std::sync::Arc::new(std::sync::Mutex::new(0)),
        }
    }

    /// 从字符串UUID列表创建
    pub fn from_strings(uuid_strings: &[&str]) -> Result<Self, uuid::Error> {
        let uuids: Result<Vec<Uuid>, _> = uuid_strings.iter().map(|s| Uuid::parse_str(s)).collect();
        Ok(Self::new(uuids?))
    }

    /// 重置索引到开头
    pub fn reset(&self) {
        if let Ok(mut index) = self.current_index.lock() {
            *index = 0;
        }
    }

    /// 添加更多UUID到序列
    pub fn add_uuids(&mut self, additional_uuids: Vec<Uuid>) {
        self.uuids.extend(additional_uuids);
    }
}

impl IdGenerator for SequentialIdGenerator {
    fn new_uuid(&self) -> Uuid {
        if let Ok(mut index) = self.current_index.lock() {
            if *index < self.uuids.len() {
                let uuid = self.uuids[*index];
                *index += 1;
                uuid
            } else {
                // 如果超出预设序列，循环使用或生成新的
                *index = 0;
                if !self.uuids.is_empty() {
                    self.uuids[0]
                } else {
                    // 如果没有预设UUID，回退到随机生成
                    Uuid::new_v4()
                }
            }
        } else {
            // 如果锁获取失败，回退到随机生成
            Uuid::new_v4()
        }
    }
}

/// 确定性ID生成器（测试用）
/// 基于种子生成可重复的UUID序列
#[derive(Debug, Clone)]
pub struct DeterministicIdGenerator {
    seed: u64,
    counter: std::sync::Arc<std::sync::Mutex<u64>>,
}

impl DeterministicIdGenerator {
    /// 创建一个基于种子的确定性ID生成器
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            counter: std::sync::Arc::new(std::sync::Mutex::new(0)),
        }
    }

    /// 重置计数器
    pub fn reset(&self) {
        if let Ok(mut counter) = self.counter.lock() {
            *counter = 0;
        }
    }
}

impl IdGenerator for DeterministicIdGenerator {
    fn new_uuid(&self) -> Uuid {
        if let Ok(mut counter) = self.counter.lock() {
            let combined = self.seed.wrapping_add(*counter);
            *counter += 1;

            // 使用组合值生成确定性UUID
            let bytes = [
                (combined >> 56) as u8,
                (combined >> 48) as u8,
                (combined >> 40) as u8,
                (combined >> 32) as u8,
                (combined >> 24) as u8,
                (combined >> 16) as u8,
                (combined >> 8) as u8,
                combined as u8,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ];

            Uuid::from_bytes(bytes)
        } else {
            // 回退到随机生成
            Uuid::new_v4()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_v4_generator() {
        let generator = UuidV4Generator::new();
        let id1 = generator.new_uuid();
        let id2 = generator.new_uuid();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_sequential_id_generator() {
        let uuid1 = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let uuid2 = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

        let generator = SequentialIdGenerator::new(vec![uuid1, uuid2]);
        assert_eq!(generator.new_uuid(), uuid1);
        assert_eq!(generator.new_uuid(), uuid2);

        // 超出序列后循环
        assert_eq!(generator.new_uuid(), uuid1);
    }

    #[test]
    fn test_sequential_id_generator_reset() {
        let uuid1 = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let uuid2 = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

        let generator = SequentialIdGenerator::new(vec![uuid1, uuid2]);
        assert_eq!(generator.new_uuid(), uuid1);

        generator.reset();
        assert_eq!(generator.new_uuid(), uuid1);
    }

    #[test]
    fn test_deterministic_id_generator() {
        let generator = DeterministicIdGenerator::new(42);
        let id1 = generator.new_uuid();
        let id2 = generator.new_uuid();
        assert_ne!(id1, id2);

        // 重置后应该生成相同的序列
        generator.reset();
        let id3 = generator.new_uuid();
        assert_eq!(id1, id3);
    }
}
