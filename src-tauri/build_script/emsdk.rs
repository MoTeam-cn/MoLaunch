//! emcc 可执行文件查找与环境变量配置
//!
//! 查找顺序：
//! 1. EMSCRIPTEN_ROOT 环境变量
//! 2. PATH（where/which emcc）
//! 3. 常见 emsdk 安装位置（自动配置 EMSDK/EM_CACHE/EM_CONFIG/PATH）
//!
//! 第三种情况返回额外 env vars，调用方需注入到 Command 中。

use std::path::{Path, PathBuf};
use std::process::Command;

/// emsdk 环境变量条目（名称 → 路径）
type EmccEnv = Vec<(&'static str, PathBuf)>;

/// 查找结果：(emcc 路径, 需要设置的 env vars)
///
/// env vars 仅在通过 emsdk 路径找到 emcc 时返回（用于设置 EM_CACHE 等）
pub fn find_emcc() -> Option<(PathBuf, Option<EmccEnv>)> {
    // 1. EMSCRIPTEN_ROOT 环境变量（emsdk activate 后会设置）
    if let Ok(root) = std::env::var("EMSCRIPTEN_ROOT") {
        let root = PathBuf::from(root);
        if let Some(emcc) = find_emcc_in_dir(&root) {
            return Some((emcc, None));
        }
    }

    // 2. PATH（where/which）
    if let Some(emcc) = find_emcc_in_path() {
        return Some((emcc, None));
    }

    // 3. 常见 emsdk 安装位置
    for emsdk in common_emsdk_paths() {
        if !emsdk.exists() {
            continue;
        }
        let emscripten_dir = emsdk.join("upstream").join("emscripten");
        if let Some(emcc) = find_emcc_in_dir(&emscripten_dir) {
            // 设置 emsdk 需要的完整环境变量
            // .emscripten 配置用 os.getenv('EM_CONFIG') 定位 emsdk 根目录，
            // 缺少 EM_CONFIG 会导致 emcc 找不到 node/python/llvm（exit code 9009）
            let em_cache = emscripten_dir.join("cache");
            let em_config = emsdk.join(".emscripten");

            // 扫描 node/python 版本目录（如 node/22.16.0_64bit/bin）
            let node_bin = find_subdir_bin(&emsdk.join("node"), "bin");
            let python_bin = find_subdir_bin(&emsdk.join("python"), "");
            let upstream_bin = emsdk.join("upstream").join("bin");

            // 构建 PATH：emscripten + node + python + upstream/bin + 原 PATH
            let current_path = std::env::var("PATH").unwrap_or_default();
            let separator = if cfg!(windows) { ";" } else { ":" };
            let mut path_parts: Vec<String> = vec![
                emscripten_dir.display().to_string(),
                upstream_bin.display().to_string(),
            ];
            if let Some(ref nb) = node_bin {
                path_parts.push(nb.display().to_string());
            }
            if let Some(ref pb) = python_bin {
                path_parts.push(pb.display().to_string());
            }
            path_parts.push(current_path);
            let new_path = path_parts.join(separator);

            let env = vec![
                ("EMSDK", emsdk.clone()),
                ("EMSCRIPTEN_ROOT", emscripten_dir.clone()),
                ("EM_CACHE", em_cache),
                ("EM_CONFIG", em_config),
                ("PATH", PathBuf::from(new_path)),
            ];
            return Some((emcc, Some(env)));
        }
    }

    None
}

/// 在 dir 下找第一个子目录，返回子目录/bin（bin_subdir 为空时返回子目录本身）
///
/// 用于扫描 emsdk 的 node/22.16.0_64bit/bin 和 python/3.13.3_64bit
fn find_subdir_bin(dir: &Path, bin_subdir: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if bin_subdir.is_empty() {
                return Some(path);
            }
            let with_bin = path.join(bin_subdir);
            if with_bin.exists() {
                return Some(with_bin);
            }
            // 某些 emsdk 版本 node 直接在版本目录下，无 bin 子目录
            return Some(path);
        }
    }
    None
}

/// 在指定目录下查找 emcc 可执行文件
fn find_emcc_in_dir(dir: &Path) -> Option<PathBuf> {
    for name in ["emcc", "emcc.exe", "emcc.bat"] {
        let p = dir.join(name);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// 通过 `where emcc` (Windows) / `which emcc` (Unix) 在 PATH 中查找
fn find_emcc_in_path() -> Option<PathBuf> {
    let cmd = if cfg!(windows) { "where" } else { "which" };
    let output = Command::new(cmd).arg("emcc").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next()?;
    let p = PathBuf::from(line.trim());
    if p.exists() {
        Some(p)
    } else {
        None
    }
}

/// 常见 emsdk 安装路径
fn common_emsdk_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(home) = std::env::var("USERPROFILE") {
        paths.push(PathBuf::from(&home).join("Desktop").join("emsdk"));
        paths.push(PathBuf::from(&home).join("emsdk"));
    }
    if let Ok(home) = std::env::var("HOME") {
        paths.push(PathBuf::from(&home).join("emsdk"));
        paths.push(PathBuf::from(&home).join(".emsdk"));
    }
    // Unix 全局安装位置
    paths.push(PathBuf::from("/usr/local/emsdk"));
    paths.push(PathBuf::from("/opt/emsdk"));
    paths
}
