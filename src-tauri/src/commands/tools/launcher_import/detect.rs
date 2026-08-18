//! 启动器数据导入：来源探测与实例枚举入口
//!
//! 各启动器探测/枚举逻辑分别位于同目录的 `pcl.rs` / `hmcl.rs` / `multimc.rs` / `curseforge.rs`，
//! 本文件仅负责编排与跨平台基础路径工具。

use std::path::{Path, PathBuf};

use crate::log_info;

use super::parse::detect_instance_info;
use crate::commands::tools::types::{ImportableInstance, LauncherKind, LauncherSource};

/// 探测本机已安装的启动器，返回所有可导入实例
pub async fn list_sources() -> Vec<LauncherSource> {
    let mut sources: Vec<LauncherSource> = vec![
        super::pcl::detect_pcl2(),
        super::pcl::detect_pcl2ce(),
        super::hmcl::detect_hmcl(),
        super::multimc::detect_multimc(),
        super::multimc::detect_prism(),
        super::curseforge::detect_curseforge(),
    ];
    sources.retain(|s| !s.instances.is_empty());
    // 跨来源去重：同一实例可能被 PCL2 注册表与 PCL2CE 配置同时匹配，
    // 或不同启动器来源指向同一路径
    let mut seen = std::collections::HashSet::new();
    for source in &mut sources {
        source.instances.retain(|i| seen.insert(i.path.clone()));
    }
    sources.retain(|s| !s.instances.is_empty());
    let total: usize = sources.iter().map(|s| s.instances.len()).sum();
    log_info!(
        "[LauncherImport] 探测到 {} 个来源，共 {} 个可导入实例",
        sources.len(),
        total
    );
    sources
}

/// 扫描用户手动选择的路径（Generic 来源）
///
/// 支持三种布局：
/// - 路径本身是实例（含版本 JSON 或 `.minecraft`）；
/// - `versions/` 子目录布局（PCL2/HMCL 风格，子目录含 `{name}.json` 或 `.minecraft`）；
/// - `instances/` 子目录布局（MultiMC 风格）。
pub fn scan_generic_path(path: &Path) -> Result<LauncherSource, String> {
    let path = normalize_user_path(path)?;
    let base_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "通用实例文件夹".to_string());

    let mut instances: Vec<ImportableInstance> = Vec::new();

    // 布局 1：versions/ 子目录（PCL2/HMCL 风格）
    let versions_dir = path.join("versions");
    if versions_dir.is_dir() {
        for dir in sorted_subdirs(&versions_dir) {
            let name = dir.file_name().unwrap_or_default().to_string_lossy();
            if has_own_json(&dir) || dir.join(".minecraft").is_dir() {
                instances.push(instance_from_dir(&name, &dir));
            }
        }
    }

    // 布局 2：instances/ 子目录（MultiMC 风格）
    let instances_dir = path.join("instances");
    if instances_dir.is_dir() {
        for dir in sorted_subdirs(&instances_dir) {
            let name = dir.file_name().unwrap_or_default().to_string_lossy();
            if dir.join("instance.cfg").is_file() {
                instances.push(instance_from_dir(&name, &dir));
            }
        }
    }

    // 布局 3：路径本身是实例
    if instances.is_empty()
        && (super::parse::find_version_json(&path).is_some() || path.join(".minecraft").is_dir())
    {
        instances.push(instance_from_dir(&base_name, &path));
    }

    // 去重（同一实例可能被多种布局匹配）
    let mut seen = std::collections::HashSet::new();
    instances.retain(|i| seen.insert(i.path.clone()));
    instances.sort_by(|a, b| a.name.cmp(&b.name));

    if instances.is_empty() {
        return Err(format!("目录 {} 下未发现可导入的实例", path.display()));
    }

    Ok(LauncherSource {
        kind: LauncherKind::Generic,
        label: LauncherKind::Generic.label().to_string(),
        base_path: path.to_string_lossy().to_string(),
        instances,
    })
}

/// 规范化用户输入的路径：转绝对路径、去除 Windows 长路径 `\\?\` 前缀、校验存在
///
/// 注意：不使用 `canonicalize()`——Windows 上它返回 `\\?\C:\...` 扩展前缀路径，
/// 直接展示给用户不友好（如 `\\?\C:\Users\...\无人入眠`）。
pub(super) fn normalize_user_path(path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("获取当前目录失败: {}", e))?
            .join(path)
    };
    let stripped = strip_extended_prefix(&absolute);
    if !stripped.is_dir() {
        return Err(format!("不是有效目录: {}", stripped.display()));
    }
    Ok(stripped)
}

/// 去除 Windows 扩展路径前缀 `\\?\`（含 UNC 形式 `\\?\UNC\`），其他平台原样返回
pub(super) fn strip_extended_prefix(path: &Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let s = path.to_string_lossy();
        if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{}", rest));
        }
        if let Some(rest) = s.strip_prefix(r"\\?\") {
            return PathBuf::from(rest);
        }
        path.to_path_buf()
    }
    #[cfg(not(target_os = "windows"))]
    {
        path.to_path_buf()
    }
}

/// 读取文本文件（容错编码）：UTF-8（含 BOM）→ GB18030（GBK 超集）回退
///
/// 中文 Windows 下 HMCL/PCL 等启动器的 JSON/INI 配置文件常为 GBK/GB18030 编码，
/// UTF-8 严格解析会失败或产生乱码，需按编码回退。
pub(super) fn read_text_file(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(&bytes);
    match std::str::from_utf8(bytes) {
        Ok(s) => Some(s.to_string()),
        Err(_) => {
            let (decoded, _, _) = encoding_rs::GB18030.decode(bytes);
            Some(decoded.into_owned())
        }
    }
}

/// 目录 → 可导入实例（自动补充版本/加载器检测信息）
///
/// 实例路径统一经 `strip_extended_prefix` 规范化：注册表/配置中的路径值
/// （PCL LaunchFolders、HMCL gameDir、MultiMC InstanceDir）可能残留 `\\?\` 前缀。
pub(super) fn instance_from_dir(name: &str, path: &Path) -> ImportableInstance {
    let path = strip_extended_prefix(path);
    let info = detect_instance_info(&path);
    ImportableInstance {
        name: name.to_string(),
        path: path.to_string_lossy().to_string(),
        mc_version: info.mc_version,
        loader: info.loader,
        loader_version: info.loader_version,
    }
}

/// 平台数据目录（Windows: %APPDATA%，macOS: ~/Library/Application Support，Linux: XDG_DATA_HOME）
pub(super) fn data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA").map(PathBuf::from)
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
    {
        let _ = PathBuf::new();
        None
    }
}

/// 用户主目录
pub(super) fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from).or({
        #[cfg(target_os = "windows")]
        {
            std::env::var_os("USERPROFILE").map(PathBuf::from)
        }
        #[cfg(not(target_os = "windows"))]
        {
            None
        }
    })
}

/// 返回目录下按文件名排序的子目录（确定性遍历）
pub(super) fn sorted_subdirs(dir: &Path) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect()
        })
        .unwrap_or_default();
    dirs.sort();
    dirs
}

/// Program Files 目录（Windows；其他平台返回 None）
pub(super) fn program_files() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("ProgramFiles").map(PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

/// 判断目录内是否存在 `{name}.json`（PCL2/HMCL 版本目录标志）
pub(super) fn has_own_json(dir: &Path) -> bool {
    let name = dir.file_name().unwrap_or_default().to_string_lossy();
    dir.join(format!("{}.json", name)).is_file()
}
