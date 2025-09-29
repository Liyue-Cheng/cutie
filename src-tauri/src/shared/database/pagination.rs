/// 分页结果
#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    /// 数据项
    pub items: Vec<T>,
    /// 当前页码（从1开始）
    pub page: u32,
    /// 每页大小
    pub page_size: u32,
    /// 总项目数
    pub total_count: u64,
    /// 总页数
    pub total_pages: u32,
    /// 是否有下一页
    pub has_next: bool,
    /// 是否有上一页
    pub has_previous: bool,
}

impl<T> PaginatedResult<T> {
    /// 创建新的分页结果
    pub fn new(items: Vec<T>, page: u32, page_size: u32, total_count: u64) -> Self {
        let total_pages = ((total_count as f64) / (page_size as f64)).ceil() as u32;
        let has_next = page < total_pages;
        let has_previous = page > 1;

        Self {
            items,
            page,
            page_size,
            total_count,
            total_pages,
            has_next,
            has_previous,
        }
    }

    /// 创建空的分页结果
    pub fn empty(page: u32, page_size: u32) -> Self {
        Self::new(Vec::new(), page, page_size, 0)
    }

    /// 获取当前页的起始索引
    pub fn start_index(&self) -> u64 {
        if self.page == 0 {
            0
        } else {
            ((self.page - 1) * self.page_size) as u64
        }
    }

    /// 获取当前页的结束索引
    pub fn end_index(&self) -> u64 {
        let end = self.start_index() + self.items.len() as u64;
        end.min(self.total_count)
    }

    /// 映射数据项类型
    pub fn map<U, F>(self, f: F) -> PaginatedResult<U>
    where
        F: FnMut(T) -> U,
    {
        PaginatedResult {
            items: self.items.into_iter().map(f).collect(),
            page: self.page,
            page_size: self.page_size,
            total_count: self.total_count,
            total_pages: self.total_pages,
            has_next: self.has_next,
            has_previous: self.has_previous,
        }
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 获取项目数量
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

/// 分页参数
#[derive(Debug, Clone)]
pub struct PaginationParams {
    /// 页码（从1开始）
    pub page: u32,
    /// 每页大小
    pub page_size: u32,
}

impl PaginationParams {
    /// 创建新的分页参数
    pub fn new(page: u32, page_size: u32) -> Self {
        Self {
            page: page.max(1), // 确保页码至少为1
            page_size: page_size.max(1).min(100), // 限制页面大小在1-100之间
        }
    }

    /// 默认分页参数
    pub fn default() -> Self {
        Self::new(1, 20)
    }

    /// 计算偏移量
    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.page_size
    }

    /// 获取限制数量
    pub fn limit(&self) -> u32 {
        self.page_size
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self::default()
    }
}

/// 分页查询构建器
#[derive(Debug, Clone)]
pub struct PaginationQuery {
    /// 分页参数
    pub params: PaginationParams,
    /// 排序字段
    pub sort_by: Option<String>,
    /// 排序方向
    pub sort_order: SortOrder,
    /// 过滤条件
    pub filters: Vec<FilterCondition>,
}

/// 排序方向
#[derive(Debug, Clone)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::Asc
    }
}

/// 过滤条件
#[derive(Debug, Clone)]
pub struct FilterCondition {
    /// 字段名
    pub field: String,
    /// 操作符
    pub operator: FilterOperator,
    /// 值
    pub value: String,
}

/// 过滤操作符
#[derive(Debug, Clone)]
pub enum FilterOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Like,
    NotLike,
    In,
    NotIn,
    IsNull,
    IsNotNull,
}

impl PaginationQuery {
    /// 创建新的分页查询
    pub fn new(page: u32, page_size: u32) -> Self {
        Self {
            params: PaginationParams::new(page, page_size),
            sort_by: None,
            sort_order: SortOrder::default(),
            filters: Vec::new(),
        }
    }

    /// 设置排序
    pub fn sort_by(mut self, field: impl Into<String>, order: SortOrder) -> Self {
        self.sort_by = Some(field.into());
        self.sort_order = order;
        self
    }

    /// 添加过滤条件
    pub fn filter(mut self, field: impl Into<String>, operator: FilterOperator, value: impl Into<String>) -> Self {
        self.filters.push(FilterCondition {
            field: field.into(),
            operator,
            value: value.into(),
        });
        self
    }

    /// 构建SQL的ORDER BY子句
    pub fn build_order_by(&self) -> String {
        match &self.sort_by {
            Some(field) => {
                let direction = match self.sort_order {
                    SortOrder::Asc => "ASC",
                    SortOrder::Desc => "DESC",
                };
                format!("ORDER BY {} {}", field, direction)
            }
            None => String::new(),
        }
    }

    /// 构建SQL的LIMIT和OFFSET子句
    pub fn build_limit_offset(&self) -> String {
        format!("LIMIT {} OFFSET {}", self.params.limit(), self.params.offset())
    }
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self::new(1, 20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paginated_result_creation() {
        let items = vec![1, 2, 3, 4, 5];
        let result = PaginatedResult::new(items, 1, 5, 20);

        assert_eq!(result.page, 1);
        assert_eq!(result.page_size, 5);
        assert_eq!(result.total_count, 20);
        assert_eq!(result.total_pages, 4);
        assert!(result.has_next);
        assert!(!result.has_previous);
        assert_eq!(result.len(), 5);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_paginated_result_empty() {
        let result: PaginatedResult<i32> = PaginatedResult::empty(1, 10);

        assert_eq!(result.page, 1);
        assert_eq!(result.page_size, 10);
        assert_eq!(result.total_count, 0);
        assert_eq!(result.total_pages, 0);
        assert!(!result.has_next);
        assert!(!result.has_previous);
        assert_eq!(result.len(), 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_paginated_result_indices() {
        let items = vec![1, 2, 3];
        let result = PaginatedResult::new(items, 2, 3, 10);

        assert_eq!(result.start_index(), 3);
        assert_eq!(result.end_index(), 6);
    }

    #[test]
    fn test_paginated_result_map() {
        let items = vec![1, 2, 3];
        let result = PaginatedResult::new(items, 1, 3, 10);
        let mapped = result.map(|x| x * 2);

        assert_eq!(mapped.items, vec![2, 4, 6]);
        assert_eq!(mapped.page, 1);
        assert_eq!(mapped.total_count, 10);
    }

    #[test]
    fn test_pagination_params() {
        let params = PaginationParams::new(2, 15);

        assert_eq!(params.page, 2);
        assert_eq!(params.page_size, 15);
        assert_eq!(params.offset(), 15);
        assert_eq!(params.limit(), 15);
    }

    #[test]
    fn test_pagination_params_bounds() {
        // 测试页码下界
        let params = PaginationParams::new(0, 10);
        assert_eq!(params.page, 1);

        // 测试页面大小下界
        let params = PaginationParams::new(1, 0);
        assert_eq!(params.page_size, 1);

        // 测试页面大小上界
        let params = PaginationParams::new(1, 200);
        assert_eq!(params.page_size, 100);
    }

    #[test]
    fn test_pagination_query_builder() {
        let query = PaginationQuery::new(1, 10)
            .sort_by("created_at", SortOrder::Desc)
            .filter("status", FilterOperator::Equal, "active");

        assert_eq!(query.params.page, 1);
        assert_eq!(query.params.page_size, 10);
        assert_eq!(query.sort_by, Some("created_at".to_string()));
        assert_eq!(query.filters.len(), 1);
        assert_eq!(query.filters[0].field, "status");
        assert_eq!(query.filters[0].value, "active");
    }

    #[test]
    fn test_pagination_query_sql_building() {
        let query = PaginationQuery::new(2, 5)
            .sort_by("name", SortOrder::Asc);

        assert_eq!(query.build_order_by(), "ORDER BY name ASC");
        assert_eq!(query.build_limit_offset(), "LIMIT 5 OFFSET 5");
    }

    #[test]
    fn test_pagination_query_no_sort() {
        let query = PaginationQuery::new(1, 10);
        assert_eq!(query.build_order_by(), "");
    }
}

