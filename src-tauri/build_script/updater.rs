//! 自动构建 updater.exe（Windows 便携版更新器）
//!
//! updater 是独立 Cargo 项目（`src-tauri/updater/`），产物 `molaunch_updater.exe`
//! 需复制到 `src-tauri/resources/updater/updater.exe` 供主程序 `include_bytes!` 嵌入。
//!
//! ## 增量编译
//!
//! 任一 `updater/src/*.rs` 或 `updater/Cargo.toml` 比已存在的产物新时触发重编译，
//! 否则跳过。CI 工作流也会显式构建 updater.exe，本模块作为本地开发的双保险。
//!
//! ## 非 Windows 平台
//!
//! 本模块整体 `#[cfg(target_os = "windows")]`，非 Windows 平台编译期排除
//! （主程序 `resources.rs` 的 `include_bytes!` 同样按 target_os 条件编译）。

#![cfg(target_os = "windows")]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

/// updater 项目根目录（相对于 src-tauri/）
const UPDATER_DIR: &str = "updater";

/// updater 源码目录
const UPDATER_SRC_DIR: &str = "updater/src";

/// updater 构建产物相对路径（相对于 src-tauri/）
const UPDATER_BUILD_OUTPUT: &str = "updater/target/release/molaunch_updater.exe";

/// 主程序嵌入资源目标路径（相对于 src-tauri/）
const RESOURCES_TARGET: &str = "resources/updater/updater.exe";

/// 构建 updater.exe 并复制到 resources 目录
pub fn build_updater() {
    let target = PathBuf::from(RESOURCES_TARGET);

    if !needs_rebuild(&target) {
        return;
    }

    let updater_dir = Path::new(UPDATER_DIR);
    if !updater_dir.join("Cargo.toml").exists() {
        println!("cargo:warning=updater/Cargo.toml 不存在，跳过 updater.exe 构建");
        println!("cargo:warning=请确保 src-tauri/updater/ 目录已初始化");
        return;
    }

    eprintln!("[build.rs] Building updater.exe via cargo...");

    let output = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(updater_dir)
        // 用 output() 捕获 stdout/stderr，避免 cargo 管道缓冲区满导致阻塞
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            println!("cargo:warning=启动 cargo 构建 updater 失败: {}", e);
            println!("cargo:warning=请手动运行: cd src-tauri/updater && cargo build --release");
            return;
        }
    };

    if !output.status.success() {
        println!(
            "cargo:warning=updater cargo build 失败 (exit code {:?})",
            output.status.code()
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        for line in stderr.lines().take(20) {
            println!("cargo:warning=updater: {}", line);
        }
        return;
    }

    // 复制产物到 resources 目录
    let build_output = PathBuf::from(UPDATER_BUILD_OUTPUT);
    if !build_output.exists() {
        println!(
            "cargo:warning=updater 构建产物不存在: {}",
            build_output.display()
        );
        return;
    }

    if let Some(parent) = target.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            println!("cargo:warning=创建 {} 失败: {}", parent.display(), e);
            return;
        }
    }

    if let Err(e) = std::fs::copy(&build_output, &target) {
        println!(
            "cargo:warning=复制 updater.exe 失败: {} -> {}: {}",
            build_output.display(),
            target.display(),
            e
        );
        return;
    }

    let size = target.metadata().map(|m| m.len()).unwrap_or(0);
    eprintln!(
        "[build.rs] updater.exe 已生成: {} ({} bytes)",
        target.display(),
        size
    );
}

/// 判断是否需要重新构建：产物不存在，或任一源文件比产物新
///
/// 同时为每个源文件声明 `cargo:rerun-if-changed`，
/// 确保文件内容修改时触发 build.rs 重跑。
fn needs_rebuild(target: &Path) -> bool {
    if !target.exists() {
        return true;
    }
    let target_mtime = target
        .metadata()
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let mut needs = false;

    // 检查 updater/Cargo.toml
    let cargo_toml = Path::new(UPDATER_DIR).join("Cargo.toml");
    if cargo_toml.exists() {
        println!("cargo:rerun-if-changed={}", cargo_toml.display());
        let mtime = cargo_toml
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        if mtime > target_mtime {
            needs = true;
        }
    }

    // 检查 updater/build.rs（winres 配置变化时需要重建）
    let build_rs = Path::new(UPDATER_DIR).join("build.rs");
    if build_rs.exists() {
        println!("cargo:rerun-if-changed={}", build_rs.display());
        let mtime = build_rs
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        if mtime > target_mtime {
            needs = true;
        }
    }

    // 检查 updater/src/*.rs
    let src_dir = Path::new(UPDATER_SRC_DIR);
    if let Ok(entries) = std::fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let is_rust = path.extension().map(|ext| ext == "rs").unwrap_or(false);
            if !is_rust {
                continue;
            }
            println!("cargo:rerun-if-changed={}", path.display());
            let mtime = path
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            if mtime > target_mtime {
                needs = true;
            }
        }
    }

    needs
}
