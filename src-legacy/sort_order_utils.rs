use crate::shared::core::{SortOrderError, SortResult};
/// 排序工具模块
///
/// 使用 lexorank 库实现确定性的排序字符串生成
/// 基于 LexoRank 算法，提供高效的列表项排序功能
use lexorank::{Bucket, LexoRank, Rank};

/// 生成初始排序字符串
///
/// **预期行为:** 生成一个位于排序空间中间的初始字符串
pub fn generate_initial_sort_order() -> String {
    // 使用 bucket 0 和中间位置的 rank "n"
    let bucket = Bucket::new(0).expect("Failed to create bucket");
    let rank = Rank::new("n").expect("Failed to create rank");
    let lexo_rank = LexoRank::new(bucket, rank);
    lexo_rank.to_string()
}

/// 在两个排序字符串之间生成中间值
///
/// **预期行为:** 返回一个字典序位于prev和next之间的字符串
/// **前置条件:** prev < next（字典序）
/// **后置条件:** prev < result < next（字典序）
/// **错误处理:** 当输入无效时返回具体错误信息
pub fn get_mid_lexo_rank(prev: &str, next: &str) -> SortResult<String> {
    // 处理空字符串情况
    if prev.is_empty() && next.is_empty() {
        return Ok(generate_initial_sort_order());
    }

    if prev.is_empty() {
        return get_rank_before(next);
    }

    if next.is_empty() {
        return get_rank_after(prev);
    }

    // 解析两个 LexoRank，如果失败则返回错误
    let prev_rank =
        LexoRank::from_string(prev).map_err(|_| SortOrderError::InvalidFormat(prev.to_string()))?;

    let next_rank =
        LexoRank::from_string(next).map_err(|_| SortOrderError::InvalidFormat(next.to_string()))?;

    // 检查顺序是否正确
    if prev >= next {
        return Err(SortOrderError::InvalidOrder {
            prev: prev.to_string(),
            next: next.to_string(),
        });
    }

    // 使用 lexorank 的 between 方法生成中间值
    match prev_rank.between(&next_rank) {
        Some(mid_rank) => Ok(mid_rank.to_string()),
        None => Err(SortOrderError::IdenticalValues(prev.to_string())),
    }
}

/// 在指定字符串之前生成排序字符串
///
/// **预期行为:** 返回一个字典序小于target的字符串
/// **错误处理:** 当输入无效时返回具体错误信息
pub fn get_rank_before(target: &str) -> SortResult<String> {
    if target.is_empty() {
        return Ok(generate_initial_sort_order());
    }

    let target_rank = LexoRank::from_string(target)
        .map_err(|_| SortOrderError::InvalidFormat(target.to_string()))?;

    Ok(target_rank.prev().to_string())
}

/// 在指定字符串之后生成排序字符串
///
/// **预期行为:** 返回一个字典序大于target的字符串
/// **错误处理:** 当输入无效时返回具体错误信息
pub fn get_rank_after(target: &str) -> SortResult<String> {
    if target.is_empty() {
        return Ok(generate_initial_sort_order());
    }

    let target_rank = LexoRank::from_string(target)
        .map_err(|_| SortOrderError::InvalidFormat(target.to_string()))?;

    Ok(target_rank.next().to_string())
}

/// 验证排序字符串的有效性
///
/// **预期行为:** 检查字符串是否是有效的 LexoRank 格式
pub fn is_valid_sort_order(sort_order: &str) -> bool {
    if sort_order.is_empty() {
        return false;
    }

    LexoRank::from_string(sort_order).is_ok()
}

/// 比较两个排序字符串
///
/// **预期行为:** 返回字典序比较结果
pub fn compare_sort_orders(a: &str, b: &str) -> std::cmp::Ordering {
    // 尝试解析为 LexoRank 进行比较
    match (LexoRank::from_string(a), LexoRank::from_string(b)) {
        (Ok(rank_a), Ok(rank_b)) => {
            // 使用 LexoRank 的字符串表示进行比较
            rank_a.to_string().cmp(&rank_b.to_string())
        }
        _ => {
            // 如果解析失败，回退到字符串比较
            a.cmp(b)
        }
    }
}

/// 创建一个新的 LexoRank 实例
///
/// **预期行为:** 创建指定 bucket 和 rank 的 LexoRank
pub fn create_lexo_rank(
    bucket_value: u8,
    rank_value: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let bucket = Bucket::new(bucket_value)?;
    let rank = Rank::new(rank_value)?;
    let lexo_rank = LexoRank::new(bucket, rank);
    Ok(lexo_rank.to_string())
}

/// 从字符串解析 LexoRank
///
/// **预期行为:** 将字符串解析为 LexoRank 结构
pub fn parse_lexo_rank(value: &str) -> SortResult<(u8, String)> {
    let lexo_rank = LexoRank::from_string(value)
        .map_err(|_| SortOrderError::InvalidFormat(value.to_string()))?;
    Ok((
        lexo_rank.bucket().value(),
        lexo_rank.rank().value().to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    // ===== 我们自己的测试 =====
    #[test]
    fn test_generate_initial_sort_order() {
        let order = generate_initial_sort_order();
        assert!(!order.is_empty());
        assert!(is_valid_sort_order(&order));
        // 应该是 "0|n" 格式
        assert_eq!(order, "0|n");
    }

    #[test]
    fn test_get_mid_lexo_rank() {
        let initial = "0|n";
        let after = get_rank_after(initial).unwrap();
        let mid = get_mid_lexo_rank(initial, &after).unwrap();

        assert!(mid.as_str() > initial);
        assert!(mid.as_str() < after.as_str());
        assert!(is_valid_sort_order(&mid));
    }

    #[test]
    fn test_get_rank_before() {
        let initial = "0|n";
        let before = get_rank_before(initial).unwrap();
        assert!(before.as_str() < initial);
        assert!(is_valid_sort_order(&before));
    }

    #[test]
    fn test_get_rank_after() {
        let initial = "0|n";
        let after = get_rank_after(initial).unwrap();
        assert!(after.as_str() > initial);
        assert!(is_valid_sort_order(&after));
    }

    #[test]
    fn test_ordering_consistency() {
        let a = "0|n";
        let c = get_rank_after(a).unwrap();
        let mid = get_mid_lexo_rank(a, &c).unwrap();
        let b = get_mid_lexo_rank(a, &mid).unwrap();

        assert!(a < b.as_str());
        assert!(b.as_str() < mid.as_str());
        assert!(mid.as_str() < c.as_str());
    }

    #[test]
    fn test_is_valid_sort_order() {
        let valid_order = generate_initial_sort_order();
        assert!(is_valid_sort_order(&valid_order));
        assert!(is_valid_sort_order("0|1"));
        assert!(is_valid_sort_order("1|abc"));
        assert!(is_valid_sort_order("2|z9"));

        assert!(!is_valid_sort_order(""));
        assert!(!is_valid_sort_order("invalid_format"));
        assert!(!is_valid_sort_order("3|abc")); // bucket > 2
        assert!(!is_valid_sort_order("0|a0")); // rank ends with 0
    }

    #[test]
    fn test_edge_cases() {
        // 测试空字符串
        let before_empty = get_rank_before("").unwrap();
        let after_empty = get_rank_after("").unwrap();
        assert!(is_valid_sort_order(&before_empty));
        assert!(is_valid_sort_order(&after_empty));

        // 测试中间值生成
        let mid_empty_prev = get_mid_lexo_rank("", &after_empty).unwrap();
        let mid_empty_next = get_mid_lexo_rank(&before_empty, "").unwrap();
        assert!(is_valid_sort_order(&mid_empty_prev));
        assert!(is_valid_sort_order(&mid_empty_next));
    }

    #[test]
    fn test_create_and_parse_lexo_rank() {
        let rank_str = create_lexo_rank(0, "abc").expect("Failed to create LexoRank");
        assert!(is_valid_sort_order(&rank_str));
        assert_eq!(rank_str, "0|abc");

        let (bucket, rank) = parse_lexo_rank(&rank_str).expect("Failed to parse LexoRank");
        assert_eq!(bucket, 0);
        assert_eq!(rank, "abc");
    }

    #[test]
    fn test_lexorank_between_functionality() {
        // 测试基于实际 lexorank 库的 between 功能
        let test_cases = [
            ("0|1", "0|3", "0|2"),
            ("0|a", "0|z", "0|b"),
            ("0|1", "0|2", "0|11"),
            ("0|a", "0|b", "0|a1"),
        ];

        for (rank1, rank2, expected_between) in test_cases {
            let between = get_mid_lexo_rank(rank1, rank2).unwrap();
            assert_eq!(between, expected_between);
            assert!(rank1 < between.as_str());
            assert!(between.as_str() < rank2);
        }
    }

    #[test]
    fn test_compare_sort_orders() {
        assert_eq!(compare_sort_orders("0|1", "0|2"), Ordering::Less);
        assert_eq!(compare_sort_orders("0|2", "0|1"), Ordering::Greater);
        assert_eq!(compare_sort_orders("0|1", "0|1"), Ordering::Equal);
        assert_eq!(compare_sort_orders("0|a", "0|z"), Ordering::Less);
        assert_eq!(compare_sort_orders("1|a", "0|z"), Ordering::Greater);
    }

    // ===== 新的错误处理测试 =====

    #[test]
    fn test_error_handling_invalid_format() {
        // 测试无效格式
        assert!(get_mid_lexo_rank("invalid", "0|n").is_err());
        assert!(get_mid_lexo_rank("0|n", "invalid").is_err());
        assert!(get_rank_before("invalid").is_err());
        assert!(get_rank_after("invalid").is_err());

        // 测试错误类型
        match get_mid_lexo_rank("invalid", "0|n") {
            Err(SortOrderError::InvalidFormat(s)) => assert_eq!(s, "invalid"),
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_error_handling_invalid_order() {
        // 测试错误的顺序
        let result = get_mid_lexo_rank("0|z", "0|a");
        assert!(result.is_err());

        match result {
            Err(SortOrderError::InvalidOrder { prev, next }) => {
                assert_eq!(prev, "0|z");
                assert_eq!(next, "0|a");
            }
            _ => panic!("Expected InvalidOrder error"),
        }
    }

    // ===== 原作者的测试 - Constructor Tests =====
    #[test]
    fn create() {
        let lex_tuples = [(0, "2a", "0|2a"), (1, "01", "1|01"), (2, "abc", "2|abc")];

        for (bucket, value, lex_string) in lex_tuples {
            let lex_bucket = Bucket::new(bucket).unwrap();
            let lex_value = Rank::new(value).unwrap();
            let lexorank = LexoRank::new(lex_bucket, lex_value);

            assert_eq!(*lexorank.bucket(), Bucket::new(bucket).unwrap());
            assert_eq!(*lexorank.rank(), Rank::new(value).unwrap());
            assert_eq!(lexorank.to_string(), lex_string);
        }
    }

    #[test]
    #[should_panic(expected = "LexoRank bucket value must be between 0 and 2 inclusive. Found: 4")]
    fn create_with_invalid_bucket() {
        let buckets = [3, 4, 10, 100];

        for bucket in buckets {
            let bucket = Bucket::new(bucket);
            assert!(bucket.is_err());
        }

        Bucket::new(4).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Lexorank value must only include 0-9 and a-z and must not end with 0. Found: a0"
    )]
    fn create_with_invalid_rank() {
        let values = ["a90", "0", "12B", "C"];

        for value in values {
            let value = Rank::new(value);
            assert!(value.is_err());
        }

        Rank::new("a0").unwrap();
    }

    #[test]
    fn create_from_string() {
        let lex_tuples = [(0, "2a"), (1, "01"), (2, "abc")];

        for (bucket, value) in lex_tuples {
            let lex_string = format!("{}|{}", bucket, value);
            let lexorank = LexoRank::from_string(&lex_string).unwrap();

            assert_eq!(*lexorank.bucket(), Bucket::new(bucket).unwrap());
            assert_eq!(*lexorank.rank(), Rank::new(value).unwrap());
            assert_eq!(lexorank.to_string(), lex_string);
        }
    }

    #[test]
    #[should_panic(expected = "LexoRank bucket value must be between 0 and 2 inclusive. Found: 4")]
    fn create_from_string_with_invalid_bucket() {
        let buckets = [3, 4, 10, 100];

        for bucket in buckets {
            let lex_string = format!("{}|abc", bucket);
            let value = LexoRank::from_string(&lex_string);
            assert!(value.is_err());
        }

        LexoRank::from_string("4|abc").unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Lexorank value must only include 0-9 and a-z and must not end with 0. Found: a0"
    )]
    fn create_from_string_with_invalid_rank() {
        let values = ["a90", "0", "12B", "C"];

        for value in values {
            let lex_string = format!("0|{}", value);
            let value = LexoRank::from_string(&lex_string);
            assert!(value.is_err());
        }

        LexoRank::from_string("0|a0").unwrap();
    }

    // ===== 原作者的测试 - Between Tests =====
    #[test]
    fn between_ranks() {
        let test_cases = [
            ("1", "3", "2"),
            ("1", "9", "2"),
            ("a", "z", "b"),
            ("1", "2", "11"),
            ("a", "b", "a1"),
            ("12", "1a", "13"),
            ("101", "123", "102"),
            ("11", "12", "111"),
            ("az", "b", "az1"),
            ("1a1", "1a11", "1a101"),
            ("z4", "z41", "z401"),
            ("z4", "z401", "z4001"),
            ("z401", "z40100001", "z401000001"),
        ];

        for (rank1, rank2, between) in test_cases {
            println!("{} -> {} <- {}", rank1, between, rank2);
            let rank1 = Rank::new(rank1).unwrap();
            let rank2 = Rank::new(rank2).unwrap();
            let between = Rank::new(between).unwrap();
            assert_eq!(rank1.between(&rank2).unwrap(), between);
            assert_eq!(rank2.between(&rank1).unwrap(), between);
        }
    }

    #[test]
    fn between_equal_ranks() {
        let test_cases = ["1", "z", "1a1", "z4", "z401", "z40100001"];

        for rank in test_cases {
            println!("{} -> {} <- {}", rank, rank, rank);
            let rank1 = Rank::new(rank).unwrap();
            let rank2 = Rank::new(rank).unwrap();
            assert_eq!(rank1.between(&rank2), None);
            assert_eq!(rank2.between(&rank1), None);
        }
    }

    #[test]
    fn between_lexoranks() {
        let test_cases = [
            ("0|1", "0|3", "0|2"),
            ("0|1", "0|9", "0|2"),
            ("0|a", "0|z", "0|b"),
            ("0|1", "0|2", "0|11"),
            ("0|a", "0|b", "0|a1"),
            ("0|12", "0|1a", "0|13"),
            ("0|101", "0|123", "0|102"),
            ("0|11", "0|12", "0|111"),
            ("0|az", "0|b", "0|az1"),
            ("0|1a1", "0|1a11", "0|1a101"),
            ("0|z4", "0|z41", "0|z401"),
            ("0|z4", "0|z401", "0|z4001"),
            ("0|z401", "0|z40100001", "0|z401000001"),
        ];

        for (lexorank1, lexorank2, between) in test_cases {
            println!("{} -> {} <- {}", lexorank1, between, lexorank2);
            let lexorank1: LexoRank = lexorank1.try_into().unwrap();
            let lexorank2: LexoRank = lexorank2.try_into().unwrap();
            let between: LexoRank = between.try_into().unwrap();
            assert_eq!(lexorank1.between(&lexorank2).unwrap(), between);
            assert_eq!(lexorank2.between(&lexorank1).unwrap(), between);
        }
    }

    #[test]
    fn between_equal_lexoranks() {
        let test_cases = [
            ("0|1", "0|1"),
            ("2|z", "2|z"),
            ("0|1a1", "0|1a1"),
            ("2|z4", "2|z4"),
            ("0|z401", "0|z401"),
            ("1|z40100001", "1|z40100001"),
        ];

        for (lexorank1, rank2) in test_cases {
            println!("{} -> {} <- {}", lexorank1, lexorank1, rank2);
            let rank1: LexoRank = lexorank1.try_into().unwrap();
            let rank2: LexoRank = rank2.try_into().unwrap();
            assert_eq!(rank1.between(&rank2), None);
            assert_eq!(rank2.between(&rank1), None);
        }
    }

    // ===== 原作者的测试 - Increment Tests =====
    #[test]
    fn increment_bucket() {
        let bucket_pairs = [(0, 1), (1, 2), (2, 0)];

        for (before, after) in bucket_pairs {
            println!("{} -> {}", before, after);
            let before_bucket = Bucket::new(before).unwrap();
            let after_bucket = Bucket::new(after).unwrap();
            assert_eq!(before_bucket.next(), after_bucket);
        }
    }

    #[test]
    fn decrement_bucket() {
        let bucket_pairs = [(0, 2), (1, 0), (2, 1)];

        for (before, after) in bucket_pairs {
            println!("{} -> {}", before, after);
            let before_bucket = Bucket::new(before).unwrap();
            let after_bucket = Bucket::new(after).unwrap();
            assert_eq!(before_bucket.prev(), after_bucket);
        }
    }

    #[test]
    fn increment_rank() {
        let test_cases = [
            ("1", "2"),
            ("8", "9"),
            ("9", "a"),
            ("a", "b"),
            ("y", "z"),
            ("z", "z1"),
            ("11", "12"),
            ("2b", "2c"),
            ("109", "10a"),
            ("abz", "ac"),
            ("yzz", "z"),
            ("y2wzz", "y2x"),
            ("zzz", "zzz1"),
        ];

        for (before, after) in test_cases {
            println!("{} -> {}", before, after);
            let before_rank = Rank::new(before).unwrap();
            let after_rank = Rank::new(after).unwrap();
            assert_eq!(before_rank.next(), after_rank);
        }
    }

    #[test]
    fn decrement_rank() {
        let test_cases = [
            ("1", "01"),
            ("8", "7"),
            ("9", "8"),
            ("a", "9"),
            ("b", "a"),
            ("z", "y"),
            ("11", "1"),
            ("2c", "2b"),
            ("10a", "109"),
            ("abz", "aby"),
            ("z1", "z"),
            ("01", "001"),
            ("01001", "01"),
        ];

        for (before, after) in test_cases {
            println!("{} -> {}", before, after);
            let before_rank = Rank::new(before).unwrap();
            let after_rank = Rank::new(after).unwrap();
            assert_eq!(before_rank.prev(), after_rank);
        }
    }

    #[test]
    fn increment_lexorank() {
        let test_cases = [
            ("1|01", "1|02"),
            ("0|9", "0|a"),
            ("0|a", "0|b"),
            ("1|y", "1|z"),
            ("0|z", "0|z1"),
            ("2|11", "2|12"),
            ("0|2b", "0|2c"),
            ("0|109", "0|10a"),
            ("2|abz", "2|ac"),
            ("0|yzz", "0|z"),
            ("1|y2wzz", "1|y2x"),
            ("0|zzz", "0|zzz1"),
        ];

        for (before, after) in test_cases {
            println!("{} -> {}", before, after);
            let before_lexorank: LexoRank = before.try_into().unwrap();
            let after_lexorank: LexoRank = after.try_into().unwrap();
            assert_eq!(before_lexorank.next(), after_lexorank);
        }
    }

    #[test]
    fn decrement_lexorank() {
        let test_cases = [
            ("1|1", "1|01"),
            ("0|8", "0|7"),
            ("2|9", "2|8"),
            ("0|a", "0|9"),
            ("0|b", "0|a"),
            ("2|z", "2|y"),
            ("1|11", "1|1"),
            ("0|2c", "0|2b"),
            ("0|10a", "0|109"),
            ("1|abz", "1|aby"),
            ("0|z1", "0|z"),
            ("0|01", "0|001"),
            ("2|01001", "2|01"),
        ];

        for (before, after) in test_cases {
            println!("{} -> {}", before, after);
            let before_lexorank: LexoRank = before.try_into().unwrap();
            let after_lexorank: LexoRank = after.try_into().unwrap();
            assert_eq!(before_lexorank.prev(), after_lexorank);
        }
    }

    // ===== 原作者的测试 - Equality Tests =====
    #[test]
    fn compare_equal_buckets() {
        let bucket1 = Bucket::new(0).unwrap();
        let bucket2 = Bucket::new(0).unwrap();
        assert_eq!(bucket1, bucket2);
    }

    #[test]
    fn compare_unequal_buckets() {
        let bucket1 = Bucket::new(0).unwrap();
        let bucket2 = Bucket::new(1).unwrap();
        assert_ne!(bucket1, bucket2);
        assert!(
            bucket1 < bucket2,
            "{:?} was not less than {:?}",
            bucket1,
            bucket2
        );
        assert!(
            bucket2 > bucket1,
            "{:?} was not greater than {:?}",
            bucket2,
            bucket1
        );
    }

    #[test]
    fn compare_equal_ranks() {
        let rank1 = Rank::new("01").unwrap();
        let rank2 = Rank::new("01").unwrap();
        assert_eq!(rank1, rank2);
    }

    #[test]
    fn compare_unequal_ranks() {
        let rank1 = Rank::new("01").unwrap();
        let rank2 = Rank::new("02").unwrap();
        assert_ne!(rank1, rank2);
        assert!(rank1 < rank2, "{:?} was not less than {:?}", rank1, rank2);
        assert!(
            rank2 > rank1,
            "{:?} was not greater than {:?}",
            rank2,
            rank1
        );
    }

    #[test]
    fn compare_unequal_ranks_2() {
        let rank_pairs = [
            ("1", "9"),
            ("a", "z"),
            ("9", "a"),
            ("5", "f"),
            ("1322", "1323"),
            ("1a22", "1b21"),
            ("azdb", "xabd"),
            ("1zzz", "abz"),
            ("010001", "01001"),
        ];

        for (r1, r2) in rank_pairs {
            let rank1 = Rank::new(r1).unwrap();
            let rank2 = Rank::new(r2).unwrap();
            assert_ne!(rank1, rank2);
            assert!(rank1 < rank2, "{:?} was not less than {:?}", rank1, rank2);
            assert!(
                rank2 > rank1,
                "{:?} was not greater than {:?}",
                rank2,
                rank1
            );
        }
    }

    #[test]
    fn compare_equal_lexoranks() {
        let lexorank1: LexoRank = "0|01".try_into().unwrap();
        let lexorank2: LexoRank = "0|01".try_into().unwrap();
        assert_eq!(lexorank1, lexorank2);
    }

    #[test]
    fn compare_unequal_lexoranks() {
        let lexorank1: LexoRank = "0|01".try_into().unwrap();
        let lexorank2: LexoRank = "1|01".try_into().unwrap();
        assert_ne!(lexorank1, lexorank2);

        let lexorank1: LexoRank = "0|01".try_into().unwrap();
        let lexorank2: LexoRank = "0|02".try_into().unwrap();
        assert_ne!(lexorank1, lexorank2);
    }
}
