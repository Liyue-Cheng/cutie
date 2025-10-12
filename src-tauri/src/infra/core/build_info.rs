// 包含由build.rs生成的构建信息
include!(concat!(env!("OUT_DIR"), "/built.rs"));

/// 获取应用版本
pub fn version() -> &'static str {
    PKG_VERSION
}

/// 获取构建时间
pub fn build_time() -> &'static str {
    BUILT_TIME_UTC
}

/// 获取Rust版本
pub fn rust_version() -> &'static str {
    RUSTC_VERSION
}

/// 获取Git提交哈希
pub fn git_commit_hash() -> Option<&'static str> {
    GIT_COMMIT_HASH
}

/// 获取Git分支
pub fn git_branch() -> Option<&'static str> {
    GIT_HEAD_REF
}

/// 获取构建目标
pub fn target() -> &'static str {
    TARGET
}

/// 获取构建配置
pub fn profile() -> &'static str {
    PROFILE
}

/// 获取完整的版本信息字符串
pub fn full_version_info() -> String {
    let mut info = format!("{} ({})", version(), build_time());

    if let Some(commit) = git_commit_hash() {
        info.push_str(&format!(" [{}]", &commit[..8])); // 只显示前8位
    }

    if let Some(branch) = git_branch() {
        info.push_str(&format!(" on {}", branch));
    }

    info
}

/// 获取所有构建信息的结构化数据
pub fn build_info() -> BuildInfo {
    BuildInfo {
        version: version(),
        build_time: build_time(),
        rust_version: rust_version(),
        git_commit_hash: git_commit_hash(),
        git_branch: git_branch(),
        target: target(),
        profile: profile(),
    }
}

/// 构建信息结构体
#[derive(Debug, Clone)]
pub struct BuildInfo {
    pub version: &'static str,
    pub build_time: &'static str,
    pub rust_version: &'static str,
    pub git_commit_hash: Option<&'static str>,
    pub git_branch: Option<&'static str>,
    pub target: &'static str,
    pub profile: &'static str,
}
