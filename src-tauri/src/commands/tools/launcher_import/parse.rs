//! 实例信息解析：查找版本 JSON、提取 MC 版本、识别加载器
//!
//! 版本提取复用 `minecraft/version/scan` 的 `extract_original_version`（多策略），
//! 加载器识别复用 `detect_loaders`（字符串特征匹配），本模块只做文件查找与结果归一化。

use std::path::{Path, PathBuf};

use crate::log_debug;
use crate::minecraft::version::scan::{detect_loaders, extract_original_version};
use crate::minecraft::version::state::VersionType;

/// 实例信息检测结果
#[derive(Debug, Clone, Default)]
pub struct DetectedInfo {
    pub mc_version: Option<String>,
    pub loader: Option<String>,
    pub loader_version: Option<String>,
}

/// 启动器元数据 JSON 文件名（导入时不应被当作版本 JSON）
const META_JSON_NAMES: &[&str] = &[
    "minecraftinstance.json",
    "instance.json",
    "config.json",
    "hmcl.json",
    "launcher-settings.json",
    "mmc-pack.json",
];

/// 解析单个实例目录：查找版本 JSON → 提取版本与加载器
pub fn detect_instance_info(instance_dir: &Path) -> DetectedInfo {
    let mut info = DetectedInfo::default();

    if let Some(json_path) = find_version_json(instance_dir) {
        if let Some(content) = super::detect::read_text_file(&json_path) {
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(json) => {
                    info.mc_version =
                        extract_original_version(&json, &content).map(|v| normalize_version(&v));
                    info.loader = detect_loader(&json, &content);
                    info.loader_version = extract_loader_version(&info.loader, &content);
                }
                Err(e) => log_debug!(
                    "[LauncherImport] 解析版本 JSON 失败 {}: {}",
                    json_path.display(),
                    e
                ),
            }
        }
    }

    // 兜底：JSON 缺失或未提取到版本时，从目录名提取（如 "1.20.1-fabric-0.15.11"）
    if info.mc_version.is_none() {
        let dir_name = instance_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        info.mc_version = extract_version_from_text(&dir_name);
    }

    info
}

/// 在实例目录中查找版本 JSON
///
/// 查找顺序：
/// 1. `{目录名}.json`（PCL2/HMCL 惯例）；
/// 2. 目录根下第一个能提取出版本号的非元数据 JSON；
/// 3. `.minecraft` 子目录下第一个能提取出版本号的 JSON。
pub fn find_version_json(instance_dir: &Path) -> Option<PathBuf> {
    let dir_name = instance_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let direct = instance_dir.join(format!("{}.json", dir_name));
    if is_candidate(&direct) && json_extracts_version(&direct) {
        return Some(direct);
    }

    if let Some(found) = scan_dir_for_json(instance_dir) {
        return Some(found);
    }

    let dot_minecraft = instance_dir.join(".minecraft");
    if dot_minecraft.is_dir() {
        if let Some(found) = scan_dir_for_json(&dot_minecraft) {
            return Some(found);
        }
    }

    None
}

/// 在单个目录内扫描：跳过元数据 JSON，返回第一个能提取出版本号的
fn scan_dir_for_json(dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut json_files: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == "json"))
        .collect();
    // 确定性排序，避免遍历顺序影响结果
    json_files.sort();

    for path in json_files {
        if !is_candidate(&path) {
            continue;
        }
        if json_extracts_version(&path) {
            return Some(path);
        }
    }
    None
}

/// 非元数据 JSON 才作为候选
fn is_candidate(path: &Path) -> bool {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    !META_JSON_NAMES.contains(&name.as_str())
}

/// JSON 文件能否提取出版本号
fn json_extracts_version(path: &Path) -> bool {
    match super::detect::read_text_file(path) {
        Some(content) => serde_json::from_str::<serde_json::Value>(&content)
            .ok()
            .and_then(|json| extract_original_version(&json, &content))
            .is_some(),
        None => false,
    }
}

/// 从版本 JSON 识别加载器（复用 scan 模块特征匹配）
///
/// 返回 loader 标识（forge/fabric/neoforge/optifine/liteloader/quilt），无则 None。
fn detect_loader(json: &serde_json::Value, content: &str) -> Option<String> {
    let (state, _, _, _, _, _, _) = detect_loaders(json, content);
    match state {
        VersionType::Forge => Some("forge".to_string()),
        VersionType::NeoForge => Some("neoforge".to_string()),
        VersionType::Fabric => Some("fabric".to_string()),
        VersionType::Quilt => Some("quilt".to_string()),
        VersionType::OptiFine => Some("optifine".to_string()),
        VersionType::LiteLoader => Some("liteloader".to_string()),
        _ => None,
    }
}

/// 从 JSON 内容提取加载器版本（maven 坐标末段，如 `net.fabricmc:fabric-loader:0.15.11` → `0.15.11`）
fn extract_loader_version(loader: &Option<String>, content: &str) -> Option<String> {
    let needle = match loader.as_deref()? {
        "forge" => "minecraftforge:forge",
        "neoforge" => "net.neoforged",
        "fabric" => "fabric-loader",
        "quilt" => "quilt-loader",
        "optifine" => "optifine:OptiFine",
        "liteloader" => "liteloader",
        _ => return None,
    };
    extract_version_from_needle(content, needle)
}

/// 在文本中定位 needle，读取到引号/逗号/换行/右花括号截断，取末段 `-` 后的版本号
pub(crate) fn extract_version_from_needle(text: &str, needle: &str) -> Option<String> {
    let idx = text.find(needle)?;
    let rest = &text[idx + needle.len()..];
    let end = rest.find(['"', ',', '\n', '}']).unwrap_or(rest.len());
    let segment = rest[..end].trim();
    if segment.is_empty() {
        return None;
    }
    // 形如 ":1.20.1-47.1.0" 或 ":0.15.11" → 去冒号
    let version = segment.trim_start_matches(':').trim();
    if version.is_empty() {
        return None;
    }
    Some(version.to_string())
}

/// 从任意文本提取版本号（如目录名 "1.20.1-fabric-0.15.11"）
fn extract_version_from_text(text: &str) -> Option<String> {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| regex::Regex::new(r"(\d+\.\d+(\.\d+)?(-\w+(\.\w+)*)?)").unwrap());
    re.captures(text).map(|c| normalize_version(&c[1]))
}

/// 版本号归一化：`20.1` → `1.20.1`，去除 `_unobfuscated` 等后缀
pub fn normalize_version(v: &str) -> String {
    let mut s = v.trim().to_string();
    if let Some(stripped) = s.strip_suffix("_unobfuscated") {
        s = stripped.to_string();
    }
    s = s.trim().to_string();
    // 补前导 1（如 "20.1"、"19.4" 这类省略主版本的写法）
    let starts_with_digit = s.as_bytes().first().is_some_and(|b| b.is_ascii_digit());
    if starts_with_digit && !s.starts_with("1.") && !s.contains('.') {
        // 无点号（如 "1"）不处理
    } else if starts_with_digit && !s.starts_with("1.") && !s.starts_with("0.") {
        if let Some(dot) = s.find('.') {
            let prefix_ok = s[..dot].parse::<u32>().is_ok();
            if prefix_ok {
                s = format!("1.{}", s);
            }
        }
    }
    log_debug!("[LauncherImport] normalize_version({}) -> {}", v, s);
    s
}
