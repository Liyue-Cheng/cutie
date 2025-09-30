/// TypeScript类型导出模块
///
/// 此模块用于触发ts-rs生成TypeScript类型定义文件
/// 运行 `cargo test export_typescript_types` 即可生成所有类型定义

#[cfg(test)]
mod tests {
    use crate::entities::task::{ContextType, DueDateType, Outcome, SourceInfo, Subtask, Task};

    /// 导出所有TypeScript类型定义
    ///
    /// 运行此测试将在 ../src/types/generated/ 目录下生成所有 .ts 文件
    #[test]
    fn export_typescript_types() {
        // 导出Task相关类型
        Subtask::export().expect("Failed to export Subtask");
        SourceInfo::export().expect("Failed to export SourceInfo");
        DueDateType::export().expect("Failed to export DueDateType");
        Outcome::export().expect("Failed to export Outcome");
        ContextType::export().expect("Failed to export ContextType");
        Task::export().expect("Failed to export Task");

        println!("✅ Successfully exported all TypeScript type definitions!");
        println!("📁 Generated files location: src/types/generated/");
    }
}
