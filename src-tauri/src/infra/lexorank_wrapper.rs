use lexorank::{Bucket, LexoRank, Rank};

use crate::infra::core::{AppError, AppResult};

/// LexoRank 服务包装器
///
/// 提供业务层友好的 API，封装第三方库细节并将错误转换为 `AppError`
pub struct LexoRankService;

impl LexoRankService {
    /// 生成初始 rank（用于空列表的第一个任务）
    pub fn initial_rank() -> String {
        // 使用中间 bucket=1，rank 选取中位字符串，最后字符避免 0
        let bucket = Bucket::new(1).expect("bucket 1 应该合法");
        let rank = Rank::new("hzzzzz").expect("固定初始 rank 应该合法");
        LexoRank::new(bucket, rank).to_string()
    }

    /// 在两个 rank 之间生成新 rank
    ///
    /// # 参数
    /// - `prev`: 前一个任务的 rank（`None` 表示列表开头）
    /// - `next`: 后一个任务的 rank（`None` 表示列表末尾）
    pub fn generate_between(prev: Option<&str>, next: Option<&str>) -> AppResult<String> {
        match (prev, next) {
            (None, None) => Ok(Self::initial_rank()),
            (None, Some(next_rank)) => {
                let parsed_next = Self::parse_rank(next_rank, "next_rank")?;
                Ok(parsed_next.prev().to_string())
            }
            (Some(prev_rank), None) => {
                let parsed_prev = Self::parse_rank(prev_rank, "prev_rank")?;
                Ok(parsed_prev.next().to_string())
            }
            (Some(prev_rank), Some(next_rank)) => {
                let parsed_prev = Self::parse_rank(prev_rank, "prev_rank")?;
                let parsed_next = Self::parse_rank(next_rank, "next_rank")?;
                parsed_prev
                    .between(&parsed_next)
                    .map(|rank| rank.to_string())
                    .ok_or_else(|| {
                        AppError::validation_error(
                            "rank_calculation",
                            "Failed to calculate rank between neighbors",
                            "RANK_CALCULATION_FAILED",
                        )
                    })
            }
        }
    }

    /// 验证 rank 字符串格式
    pub fn validate_rank(rank: &str) -> AppResult<()> {
        LexoRank::from_string(rank).map(|_| ()).map_err(|err| {
            AppError::validation_error(
                "rank",
                format!("Invalid rank format: {:?}", err),
                "INVALID_RANK_FORMAT",
            )
        })
    }

    fn parse_rank(value: &str, field: &str) -> AppResult<LexoRank> {
        LexoRank::from_string(value).map_err(|err| {
            AppError::validation_error(
                field,
                format!("Invalid rank format: {:?}", err),
                "INVALID_RANK_FORMAT",
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_rank_is_valid() {
        let rank = LexoRankService::initial_rank();
        assert!(!rank.is_empty());
        assert!(LexoRankService::validate_rank(&rank).is_ok());
    }

    #[test]
    fn generate_between_empty_list() {
        let rank = LexoRankService::generate_between(None, None).unwrap();
        assert!(!rank.is_empty());
    }

    #[test]
    fn generate_between_start() {
        let next = LexoRankService::initial_rank();
        let new_rank = LexoRankService::generate_between(None, Some(&next)).unwrap();
        assert!(new_rank < next);
    }

    #[test]
    fn generate_between_end() {
        let prev = LexoRankService::initial_rank();
        let new_rank = LexoRankService::generate_between(Some(&prev), None).unwrap();
        assert!(new_rank > prev);
    }

    #[test]
    fn generate_between_middle() {
        let rank1 = LexoRankService::initial_rank();
        let rank2 = LexoRankService::generate_between(Some(&rank1), None).unwrap();
        let middle = LexoRankService::generate_between(Some(&rank1), Some(&rank2)).unwrap();
        assert!(rank1 < middle && middle < rank2);
    }

    #[test]
    fn validate_rank_rejects_invalid_value() {
        let result = LexoRankService::validate_rank("invalid");
        assert!(matches!(result, Err(AppError::ValidationFailed(_))));
    }
}
