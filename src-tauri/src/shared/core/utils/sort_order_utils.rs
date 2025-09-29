/// 排序工具模块
///
/// **预期行为:** 实现一个确定性的排序字符串生成算法（如LexoRank的简化版）
/// **后置条件:** get_mid_lexo_rank("a", "c")必须总是返回一个介于"a"和"c"之间的字符串（如"b"）
/// **边界情况:** 必须能正确处理在列表头部、尾部插入，以及两个相邻字符串之间没有空间时的情况

const CHARSET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const BASE: usize = CHARSET.len();
const MIN_CHAR: u8 = CHARSET[0]; // '0'
const MAX_CHAR: u8 = CHARSET[BASE - 1]; // 'z'

/// 生成初始排序字符串
///
/// **预期行为:** 生成一个位于排序空间中间的初始字符串
pub fn generate_initial_sort_order() -> String {
    "n".to_string() // 位于字符集中间位置
}

/// 在两个排序字符串之间生成中间值
///
/// **预期行为:** 返回一个字典序位于prev和next之间的字符串
/// **前置条件:** prev < next（字典序）
/// **后置条件:** prev < result < next（字典序）
pub fn get_mid_lexo_rank(prev: &str, next: &str) -> String {
    if prev.is_empty() {
        return get_rank_before(next);
    }

    if next.is_empty() {
        return get_rank_after(prev);
    }

    if prev >= next {
        // 如果顺序错误，返回next之后的排序
        return get_rank_after(next);
    }

    // 找到第一个不同的位置
    let mut i = 0;
    let prev_bytes = prev.as_bytes();
    let next_bytes = next.as_bytes();
    let min_len = prev_bytes.len().min(next_bytes.len());

    while i < min_len && prev_bytes[i] == next_bytes[i] {
        i += 1;
    }

    // 构建结果字符串
    let mut result = Vec::new();

    // 复制相同的前缀
    result.extend_from_slice(&prev_bytes[..i]);

    if i < prev_bytes.len() && i < next_bytes.len() {
        // 两个字符串在位置i处不同
        let prev_char = prev_bytes[i];
        let next_char = next_bytes[i];

        if char_index(next_char) - char_index(prev_char) > 1 {
            // 有空间插入中间字符
            let mid_index = (char_index(prev_char) + char_index(next_char)) / 2;
            result.push(CHARSET[mid_index]);
        } else {
            // 没有空间，需要扩展
            result.push(prev_char);
            result.extend(get_rank_between_suffix(
                &prev_bytes[i + 1..],
                &next_bytes[i + 1..],
            ));
        }
    } else if i < prev_bytes.len() {
        // next是prev的前缀，在prev后添加字符
        result.extend_from_slice(&prev_bytes[..i + 1]);
        if i + 1 < prev_bytes.len() {
            result.extend(increment_string(&prev_bytes[i + 1..]));
        } else {
            result.push(get_mid_char(MIN_CHAR));
        }
    } else {
        // prev是next的前缀，在next前添加字符
        if i < next_bytes.len() {
            let next_char = next_bytes[i];
            if next_char > MIN_CHAR {
                let mid_index = char_index(next_char) / 2;
                result.push(CHARSET[mid_index]);
            } else {
                result.push(MIN_CHAR);
                result.extend(get_rank_before_suffix(&next_bytes[i + 1..]));
            }
        }
    }

    String::from_utf8(result).unwrap_or_else(|_| generate_initial_sort_order())
}

/// 在指定字符串之前生成排序字符串
///
/// **预期行为:** 返回一个字典序小于target的字符串
pub fn get_rank_before(target: &str) -> String {
    if target.is_empty() {
        return MIN_CHAR.to_string();
    }

    let target_bytes = target.as_bytes();
    let mut result = Vec::new();

    // 找到第一个不是最小字符的位置
    let mut i = 0;
    while i < target_bytes.len() && target_bytes[i] == MIN_CHAR {
        result.push(MIN_CHAR);
        i += 1;
    }

    if i < target_bytes.len() {
        let char_idx = char_index(target_bytes[i]);
        if char_idx > 0 {
            let mid_index = char_idx / 2;
            result.push(CHARSET[mid_index]);
        } else {
            result.push(MIN_CHAR);
            result.push(get_mid_char(MAX_CHAR));
        }
    } else {
        // 所有字符都是最小字符
        result.push(get_mid_char(MAX_CHAR));
    }

    String::from_utf8(result).unwrap_or_else(|_| MIN_CHAR.to_string())
}

/// 在指定字符串之后生成排序字符串
///
/// **预期行为:** 返回一个字典序大于target的字符串
pub fn get_rank_after(target: &str) -> String {
    if target.is_empty() {
        return generate_initial_sort_order();
    }

    let target_bytes = target.as_bytes();
    let mut result = Vec::new();

    // 尝试递增最后一个字符
    let mut i = target_bytes.len();
    let mut carry = true;

    while i > 0 && carry {
        i -= 1;
        let char_idx = char_index(target_bytes[i]);

        if char_idx < BASE - 1 {
            // 可以递增
            result = target_bytes[..i].to_vec();
            result.push(CHARSET[char_idx + 1]);
            carry = false;
        }
    }

    if carry {
        // 所有字符都是最大字符，需要扩展
        result = target_bytes.to_vec();
        result.push(get_mid_char(MIN_CHAR));
    }

    String::from_utf8(result).unwrap_or_else(|_| generate_initial_sort_order())
}

/// 获取字符在字符集中的索引
fn char_index(ch: u8) -> usize {
    CHARSET.iter().position(|&c| c == ch).unwrap_or(0)
}

/// 获取两个字符的中间字符
fn get_mid_char(base_char: u8) -> u8 {
    let base_idx = char_index(base_char);
    let mid_idx = (base_idx + BASE) / 2;
    CHARSET[mid_idx % BASE]
}

/// 在两个后缀之间生成排序后缀
fn get_rank_between_suffix(prev_suffix: &[u8], next_suffix: &[u8]) -> Vec<u8> {
    if prev_suffix.is_empty() && next_suffix.is_empty() {
        return vec![get_mid_char(MIN_CHAR)];
    }

    if prev_suffix.is_empty() {
        return get_rank_before_suffix(next_suffix);
    }

    if next_suffix.is_empty() {
        return increment_string(prev_suffix);
    }

    // 递归处理
    let mid_rank = get_mid_lexo_rank(
        &String::from_utf8_lossy(prev_suffix),
        &String::from_utf8_lossy(next_suffix),
    );

    mid_rank.as_bytes().to_vec()
}

/// 在后缀之前生成排序后缀
fn get_rank_before_suffix(suffix: &[u8]) -> Vec<u8> {
    if suffix.is_empty() {
        return vec![get_mid_char(MAX_CHAR)];
    }

    let before_rank = get_rank_before(&String::from_utf8_lossy(suffix));
    before_rank.as_bytes().to_vec()
}

/// 递增字符串
fn increment_string(s: &[u8]) -> Vec<u8> {
    let mut result = s.to_vec();
    let mut i = result.len();
    let mut carry = true;

    while i > 0 && carry {
        i -= 1;
        let char_idx = char_index(result[i]);

        if char_idx < BASE - 1 {
            result[i] = CHARSET[char_idx + 1];
            carry = false;
        } else {
            result[i] = MIN_CHAR;
        }
    }

    if carry {
        result.insert(0, CHARSET[1]); // 在前面添加字符
    }

    result
}

/// 验证排序字符串的有效性
///
/// **预期行为:** 检查字符串是否只包含有效的排序字符
pub fn is_valid_sort_order(sort_order: &str) -> bool {
    if sort_order.is_empty() {
        return false;
    }

    sort_order
        .as_bytes()
        .iter()
        .all(|&ch| CHARSET.contains(&ch))
}

/// 比较两个排序字符串
///
/// **预期行为:** 返回字典序比较结果
pub fn compare_sort_orders(a: &str, b: &str) -> std::cmp::Ordering {
    a.cmp(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_initial_sort_order() {
        let order = generate_initial_sort_order();
        assert!(!order.is_empty());
        assert!(is_valid_sort_order(&order));
    }

    #[test]
    fn test_get_mid_lexo_rank() {
        let mid = get_mid_lexo_rank("a", "c");
        assert!(mid.as_str() > "a");
        assert!(mid.as_str() < "c");
        assert!(is_valid_sort_order(&mid));
    }

    #[test]
    fn test_get_rank_before() {
        let before = get_rank_before("b");
        assert!(before.as_str() < "b");
        assert!(is_valid_sort_order(&before));
    }

    #[test]
    fn test_get_rank_after() {
        let after = get_rank_after("b");
        assert!(after.as_str() > "b");
        assert!(is_valid_sort_order(&after));
    }

    #[test]
    fn test_ordering_consistency() {
        let a = "a";
        let mid = get_mid_lexo_rank(a, "c");
        let b = get_mid_lexo_rank(a, &mid);

        assert!(a < b.as_str());
        assert!(b < mid);
        assert!(mid.as_str() < "c");
    }

    #[test]
    fn test_is_valid_sort_order() {
        assert!(is_valid_sort_order("abc123"));
        assert!(is_valid_sort_order("0"));
        assert!(is_valid_sort_order("z"));
        assert!(!is_valid_sort_order(""));
        assert!(!is_valid_sort_order("abc@123")); // 包含非法字符
    }

    #[test]
    fn test_edge_cases() {
        // 测试相邻字符
        let mid = get_mid_lexo_rank("a", "b");
        assert!(mid.as_str() > "a");
        assert!(mid.as_str() < "b");

        // 测试空字符串
        let before_empty = get_rank_before("");
        let after_empty = get_rank_after("");
        assert!(is_valid_sort_order(&before_empty));
        assert!(is_valid_sort_order(&after_empty));
    }
}

