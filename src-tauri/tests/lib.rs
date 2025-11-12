mod endpoint;
/// Cutie 集成测试套件
///
/// 组织结构：
/// - infrastructure/  测试基础设施（数据库、HTTP客户端、Fixtures）
/// - unit/            单元测试（Repository、Assembler等）
/// - endpoint/        端点测试（单个HTTP端点）
/// - integration/     业务集成测试（多端点协同）
mod infrastructure;
mod integration;
mod unit;
