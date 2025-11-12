/// Cutie API Documentation Composer
///
/// 从所有endpoint文件中提取CABC文档，生成完整的API手册
use regex::Regex;
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct EndpointDoc {
    feature: String,
    endpoint_name: String,
    method: String,
    path: String,
    content: String,
    file_path: PathBuf,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let src_dir = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        // 工具从 src-tauri 目录运行，所以默认是 src
        PathBuf::from("src")
    };

    let output_path = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        // 输出到项目根目录的 docs/API.md
        PathBuf::from("../docs/API.md")
    };

    println!("🔍 Scanning endpoints in: {}", src_dir.display());

    match extract_all_docs(&src_dir) {
        Ok(docs) => {
            if docs.is_empty() {
                eprintln!("⚠️  No CABC documentation found!");
                std::process::exit(1);
            }

            println!("📝 Found {} endpoint documentation(s)", docs.len());

            match generate_markdown(&docs, &output_path) {
                Ok(_) => {
                    println!("✅ API documentation generated: {}", output_path.display());
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("❌ Failed to generate documentation: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to extract documentation: {}", e);
            std::process::exit(1);
        }
    }
}

/// 提取所有端点文档
fn extract_all_docs(src_dir: &Path) -> Result<Vec<EndpointDoc>, String> {
    let features_dir = src_dir.join("features");
    if !features_dir.exists() {
        return Err(format!(
            "Features directory not found: {}",
            features_dir.display()
        ));
    }

    let mut docs = Vec::new();

    // 遍历所有feature目录
    for feature_entry in
        fs::read_dir(&features_dir).map_err(|e| format!("Failed to read features dir: {}", e))?
    {
        let feature_entry = feature_entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let feature_path = feature_entry.path();

        if !feature_path.is_dir() {
            continue;
        }

        let feature_name = feature_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // 跳过shared目录
        if feature_name == "shared" {
            continue;
        }

        // 查找endpoints目录
        let endpoints_dir = feature_path.join("endpoints");
        if !endpoints_dir.exists() {
            continue;
        }

        // 扫描所有endpoint文件
        for endpoint_entry in fs::read_dir(&endpoints_dir)
            .map_err(|e| format!("Failed to read endpoints dir: {}", e))?
        {
            let endpoint_entry =
                endpoint_entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let endpoint_path = endpoint_entry.path();

            if !endpoint_path.is_file()
                || endpoint_path.extension().and_then(|s| s.to_str()) != Some("rs")
            {
                continue;
            }

            // 读取文件内容
            let content = fs::read_to_string(&endpoint_path)
                .map_err(|e| format!("Failed to read {}: {}", endpoint_path.display(), e))?;

            // 提取CABC文档
            if let Some(doc) = extract_cabc_doc(&content, &feature_name, &endpoint_path) {
                docs.push(doc);
            }
        }
    }

    Ok(docs)
}

/// 从文件内容中提取CABC文档
fn extract_cabc_doc(content: &str, feature: &str, file_path: &Path) -> Option<EndpointDoc> {
    // 匹配 /* CABC for `name` ... */ 格式的注释
    let re = Regex::new(r#"(?s)/\*\s*CABC for `([^`]+)`\s*(.*?)\*/"#).ok()?;

    let captures = re.captures(content)?;
    let endpoint_name = captures.get(1)?.as_str().to_string();
    let doc_content = captures.get(2)?.as_str().to_string();

    // 提取HTTP方法和路径（从文档的第一部分）
    let (method, path) = extract_endpoint_signature(&doc_content)?;

    Some(EndpointDoc {
        feature: feature.to_string(),
        endpoint_name,
        method,
        path,
        content: doc_content.trim().to_string(),
        file_path: file_path.to_path_buf(),
    })
}

/// 从文档内容中提取端点签名（HTTP方法和路径）
fn extract_endpoint_signature(content: &str) -> Option<(String, String)> {
    // 匹配 "## 1. 端点签名 (Endpoint Signature)" 后的内容
    let sig_re = Regex::new(r#"##\s*1\.\s*端点签名.*?\n\s*([A-Z]+)\s+(/[^\s]+)"#).ok()?;

    let captures = sig_re.captures(content)?;
    let method = captures.get(1)?.as_str().to_string();
    let path = captures.get(2)?.as_str().to_string();

    Some((method, path))
}

/// 生成Markdown文档
fn generate_markdown(docs: &[EndpointDoc], output_path: &Path) -> Result<(), String> {
    // 确保输出目录存在
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    let mut file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    // 写入文档头部
    writeln!(file, "# Cutie API Reference").map_err(|e| format!("Failed to write: {}", e))?;
    writeln!(file).map_err(|e| format!("Failed to write: {}", e))?;
    writeln!(
        file,
        "> 本文档由 `doc-composer` 工具自动生成，请勿手动编辑。"
    )
    .map_err(|e| format!("Failed to write: {}", e))?;
    writeln!(
        file,
        "> 源文件位置：`src-tauri/src/features/*/endpoints/*.rs`"
    )
    .map_err(|e| format!("Failed to write: {}", e))?;
    writeln!(file).map_err(|e| format!("Failed to write: {}", e))?;

    // 按feature分组
    let mut grouped: BTreeMap<String, Vec<&EndpointDoc>> = BTreeMap::new();
    for doc in docs {
        grouped.entry(doc.feature.clone()).or_default().push(doc);
    }

    // 生成目录
    writeln!(file, "## Table of Contents").map_err(|e| format!("Failed to write: {}", e))?;
    writeln!(file).map_err(|e| format!("Failed to write: {}", e))?;

    for (feature, endpoints) in &grouped {
        let feature_title = feature_display_name(feature);
        writeln!(file, "- [{}](#{feature})", feature_title)
            .map_err(|e| format!("Failed to write: {}", e))?;

        for endpoint in endpoints {
            let anchor = format!(
                "{}-{}",
                endpoint.method.to_lowercase(),
                endpoint
                    .path
                    .replace('/', "")
                    .replace('{', "")
                    .replace('}', "")
            );
            writeln!(
                file,
                "  - [{} {}](#{})",
                endpoint.method, endpoint.path, anchor
            )
            .map_err(|e| format!("Failed to write: {}", e))?;
        }
    }
    writeln!(file).map_err(|e| format!("Failed to write: {}", e))?;

    // 按feature输出文档
    for (feature, endpoints) in grouped {
        let feature_title = feature_display_name(&feature);
        writeln!(file, "---").map_err(|e| format!("Failed to write: {}", e))?;
        writeln!(file).map_err(|e| format!("Failed to write: {}", e))?;
        writeln!(file, "## {}", feature_title).map_err(|e| format!("Failed to write: {}", e))?;
        writeln!(file).map_err(|e| format!("Failed to write: {}", e))?;

        // 对endpoint按HTTP方法和路径排序
        let mut sorted_endpoints = endpoints;
        sorted_endpoints.sort_by(|a, b| a.path.cmp(&b.path).then_with(|| a.method.cmp(&b.method)));

        for doc in sorted_endpoints {
            writeln!(file, "### {} {}", doc.method, doc.path)
                .map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(file).map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(file, "<details>").map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(
                file,
                "<summary>源文件: <code>{}</code></summary>",
                doc.file_path
                    .strip_prefix("src-tauri/")
                    .unwrap_or(&doc.file_path)
                    .display()
            )
            .map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(file, "</details>").map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(file).map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(file, "{}", doc.content).map_err(|e| format!("Failed to write: {}", e))?;
            writeln!(file).map_err(|e| format!("Failed to write: {}", e))?;
        }
    }

    Ok(())
}

/// 将feature名称转换为显示名称
fn feature_display_name(feature: &str) -> String {
    match feature {
        "tasks" => "Tasks (任务管理)".to_string(),
        "areas" => "Areas (领域管理)".to_string(),
        "time_blocks" => "Time Blocks (时间块管理)".to_string(),
        "views" => "Views (视图查询)".to_string(),
        "view_preferences" => "View Preferences (视图偏好)".to_string(),
        _ => {
            // 首字母大写
            let mut chars = feature.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }
    }
}
