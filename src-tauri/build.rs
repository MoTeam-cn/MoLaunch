//! Build script
//!
//! 1. 调用 Tauri build 生成 IPC 类型与配置
//! 2. 调用 Emscripten (emcc) 将 cubiomes C 源码编译为 WebAssembly
//! 3. 自动构建 updater.exe（Windows 便携版更新器，仅 Windows 平台）
//!
//! emcc 相关逻辑已模块化到 build_script/ 子目录：
//! - build_script/cubiomes_wasm.rs：编译入口、源文件清单、增量编译判断
//! - build_script/emsdk.rs：emcc 可执行文件查找与环境变量配置
//! - build_script/updater.rs：updater.exe 增量构建（仅 Windows）

mod build_script;

use build_script::cubiomes_wasm::compile_cubiomes_wasm;
use std::path::Path;

fn main() {
    // 先检查 easytier-core 嵌入式资源（缺失时给出中文下载指引并终止，避免 include_bytes! 报晦涩编译错误）
    build_script::easytier::check_easytier();

    tauri_build::build();

    // 同步项目许可协议（根目录 LICENSE → resources/LICENSE.txt 副本）
    // resources.rs 通过 include_str! 在编译期嵌入二进制，确保每次打包都包含最新许可协议
    sync_license();

    // 自动编译 cubiomes 到 WASM（每次 cubiomes 源码变化时重新编译）
    compile_cubiomes_wasm();
    println!("cargo:rerun-if-changed=cubiomes");

    // 自动构建 updater.exe 并复制到 resources/updater/（仅 Windows）
    #[cfg(target_os = "windows")]
    build_script::updater::build_updater();
}

/// 同步项目许可协议到资源目录（副本引用）
///
/// 项目根目录 LICENSE 是许可协议的唯一权威副本；此处每次构建时将其复制到
/// `resources/LICENSE.txt`，供 `resources.rs` 的 include_str! 编译期嵌入二进制，
/// 保证「设置 - 更多 - 许可协议」展示的文本与仓库一致、且每次打包都包含。
/// 内容无变化时不写盘，避免 include_str! 触发无意义的整体重编译。
fn sync_license() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src = Path::new(manifest_dir).join("../LICENSE");
    // 规范化路径（去掉 ../ 段），保证 fs::read 与 rerun-if-changed 使用同一绝对路径
    let src = src.canonicalize().unwrap_or(src);
    let dst = Path::new(manifest_dir)
        .join("resources")
        .join("LICENSE.txt");
    if let Ok(content) = std::fs::read(&src) {
        let changed = std::fs::read(&dst).map(|c| c != content).unwrap_or(true);
        if changed {
            let _ = std::fs::write(&dst, content);
        }
    }
    // 根 LICENSE 变化时重新运行本构建脚本（进而重编译资源模块）
    println!("cargo:rerun-if-changed={}", src.display());
}
