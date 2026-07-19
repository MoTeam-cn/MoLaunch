//! Java 搜索模块
//! 参考PCL2的Java搜索逻辑

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::detect::detect_java;
use super::JavaRuntime;

/// 搜索关键词（参考PCL2，共67个）
const SEARCH_KEYWORDS: &[&str] = &[
    "java",
    "jdk",
    "jre",
    "env",
    "run",
    "mc",
    "dragon",
    "well",
    "bin",
    "sdk",
    "candidate",
    "current",
    "software",
    "cache",
    "temp",
    "corretto",
    "roaming",
    "users",
    "craft",
    "program",
    "net",
    "oracle",
    "game",
    "file",
    "data",
    "jvm",
    "server",
    "client",
    "mojang",
    "eclipse",
    "microsoft",
    "hotspot",
    "runtime",
    "x86",
    "x64",
    "arm",
    "forge",
    "optifine",
    "hmcl",
    "mod",
    "fabric",
    "download",
    "launch",
    "path",
    "version",
    "pcl",
    "zulu",
    "local",
    "packages",
    "jbr",
    "bellsoft",
    "liberica",
    "graal",
    "adoptium",
    "temurin",
    "semerulu",
    "1.",
];

/// 搜索系统中的Java
pub fn search_java() -> Vec<JavaRuntime> {
    search_java_with_paths(&[])
}

/// 带额外搜索路径的 Java 搜索
///
/// `extra_paths` 用于追加搜索根目录（如游戏目录、APPDATA 等），会全遍历搜索。
/// 参考 PCL2 `JavaSearchFolder(..., IsFullSearch:=True)`。
pub fn search_java_with_paths(extra_paths: &[PathBuf]) -> Vec<JavaRuntime> {
    crate::log_separator!("Java Search");
    crate::log_info!("[Java] Starting Java search...");

    // 1. 收集候选路径（环境变量 / 全磁盘 / 用户目录 / 启动器目录 / runtime / 额外路径）
    let java_candidates = collect_java_candidates(extra_paths);

    crate::log_info!(
        "[Java] Found {} candidates, verifying...",
        java_candidates.len()
    );

    // 2. 验证所有候选Java
    let mut java_list = verify_java_candidates(&java_candidates);

    // 3. 排序：大版本优先，其次 64 位优先
    java_list.sort_by(|a, b| {
        b.major_version
            .cmp(&a.major_version)
            .then(b.is_64bit.cmp(&a.is_64bit))
    });

    crate::log_info!(
        "[Java] Search completed, found {} valid Java installations",
        java_list.len()
    );
    crate::log_separator!("Java Search End");

    java_list
}

/// 候选路径收集器：维护去重后的候选列表
struct CandidateCollector {
    candidates: Vec<PathBuf>,
    seen_paths: HashSet<String>,
}

impl CandidateCollector {
    fn new() -> Self {
        Self {
            candidates: Vec::new(),
            seen_paths: HashSet::new(),
        }
    }

    fn add(&mut self, path: &Path) {
        let path_str = path.to_string_lossy().to_lowercase().replace("\\", "/");
        if !self.seen_paths.contains(&path_str) {
            self.seen_paths.insert(path_str);
            self.candidates.push(path.to_path_buf());
            crate::log_debug!("[Java] Candidate: {}", path.display());
        }
    }

    fn into_inner(self) -> Vec<PathBuf> {
        self.candidates
    }
}

/// 执行全部搜索步骤，返回去重后的候选 Java 可执行文件路径列表
fn collect_java_candidates(extra_paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut collector = CandidateCollector::new();

    // Step 1: 环境变量扫描
    crate::log_info!("[Java] Step 1: Checking environment variables...");
    collect_from_env(&mut collector);

    // Step 2: 全磁盘扫描（关键词匹配）
    crate::log_info!("[Java] Step 2: Searching local drives...");
    for drive in get_local_drives() {
        search_folder_recursive(&drive, &mut collector, false);
    }

    // Step 3: 用户目录深度搜索
    crate::log_info!("[Java] Step 3: Searching user directories...");
    if let Some(user_profile) = std::env::var_os("USERPROFILE") {
        let base = Path::new(&user_profile);
        search_folder_recursive(base, &mut collector, false);
        // .jdks 全搜索
        search_folder_recursive(&base.join(".jdks"), &mut collector, true);
        // .sdkman 全搜索
        search_folder_recursive(
            &base.join(".sdkman/candidates/java"),
            &mut collector,
            true,
        );
    }

    // Step 4: 启动器目录全搜索
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            crate::log_debug!(
                "[Java] Step 4: Searching launcher directory: {}",
                exe_dir.display()
            );
            search_folder_recursive(exe_dir, &mut collector, true);
        }
    }

    // Step 5: APPDATA\.minecraft\runtime\（PCL2/官启自动下载的 Java 存放处）
    // 与 PCL2 一致，runtime 下的 Java 跨游戏目录共享，必须搜索
    crate::log_info!("[Java] Step 5: Searching APPDATA .minecraft runtime...");
    if let Ok(appdata) = std::env::var("APPDATA") {
        let runtime_dir = Path::new(&appdata).join(".minecraft").join("runtime");
        if runtime_dir.exists() {
            crate::log_debug!(
                "[Java] Step 5: Searching runtime directory: {}",
                runtime_dir.display()
            );
            search_folder_recursive(&runtime_dir, &mut collector, true);
        }
    }

    // Step 6: 调用方追加的额外搜索路径（如游戏目录）
    for (i, extra) in extra_paths.iter().enumerate() {
        if extra.exists() && extra.is_dir() {
            crate::log_debug!(
                "[Java] Step 6.{}: Searching extra path: {}",
                i,
                extra.display()
            );
            search_folder_recursive(extra, &mut collector, true);
        }
    }

    collector.into_inner()
}

/// Step 1: 从环境变量（PATH、JAVA_HOME）收集候选 Java
fn collect_from_env(collector: &mut CandidateCollector) {
    let mut env_paths = String::new();
    if let Ok(path) = std::env::var("PATH") {
        env_paths.push_str(&path);
    }
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        crate::log_debug!("[Java] JAVA_HOME: {}", java_home);
        env_paths.push(';');
        env_paths.push_str(&java_home);
        env_paths.push(';');
        env_paths.push_str(&format!("{}\\bin", java_home));
    }
    for dir in std::env::split_paths(&env_paths) {
        let dir_str = dir.to_string_lossy().to_lowercase().replace("\\", "/");
        if dir_str.is_empty() {
            continue;
        }
        // 粗略检查 javaw.exe
        let javaw_path = dir.join("javaw.exe");
        let java_path = dir.join("java.exe");
        if javaw_path.exists() {
            collector.add(&javaw_path);
        } else if java_path.exists() {
            collector.add(&java_path);
        }
    }
}

/// 验证候选 Java 路径列表，返回成功检测的 JavaRuntime 列表
fn verify_java_candidates(candidates: &[PathBuf]) -> Vec<JavaRuntime> {
    let mut java_list = Vec::new();
    for path in candidates {
        match detect_java(path) {
            Ok(java) => {
                crate::log_info!("[Java] Valid: {} ({})", java.version, java.path_folder);
                java_list.push(java);
            }
            Err(e) => {
                crate::log_debug!("[Java] Invalid {}: {}", path.display(), e);
            }
        }
    }
    java_list
}

/// 递归搜索文件夹（参考PCL2的JavaSearchFolder）
fn search_folder_recursive(dir: &Path, collector: &mut CandidateCollector, is_full_search: bool) {
    if !dir.exists() || !dir.is_dir() {
        return;
    }

    // 检查当前目录是否有 javaw.exe 或 java.exe
    let javaw_path = dir.join("javaw.exe");
    let java_path = dir.join("java.exe");
    if javaw_path.exists() {
        collector.add(&javaw_path);
    } else if java_path.exists() {
        collector.add(&java_path);
    }

    // 遍历子目录
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        // 跳过符号链接
        if is_symlink(&path) {
            continue;
        }

        let dir_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();

        // 判断是否需要递归搜索
        let should_search = is_full_search ||
            dir_name == "users" ||
            dir_name.parse::<f64>().is_ok() ||  // 数字开头
            dir_name == "bin" ||
            SEARCH_KEYWORDS.iter().any(|kw| dir_name.contains(kw));

        if should_search {
            search_folder_recursive(&path, collector, false);
        }
    }
}

fn is_symlink(path: &Path) -> bool {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata.file_type().is_symlink(),
        Err(_) => false,
    }
}

fn get_local_drives() -> Vec<PathBuf> {
    let mut drives = Vec::new();
    for letter in 'A'..='Z' {
        let drive = format!("{}:\\", letter);
        let path = PathBuf::from(&drive);
        if path.exists() {
            drives.push(path);
        }
    }
    drives
}
