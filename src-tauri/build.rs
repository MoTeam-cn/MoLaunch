//! Build script
//!
//! 1. 调用 Tauri build 生成 IPC 类型与配置
//! 2. 调用 Emscripten (emcc) 将 cubiomes C 源码编译为 WebAssembly
//!
//! emcc 相关逻辑已模块化到 build_script/ 子目录：
//! - build_script/cubiomes_wasm.rs：编译入口、源文件清单、增量编译判断
//! - build_script/emsdk.rs：emcc 可执行文件查找与环境变量配置

mod build_script;

use build_script::cubiomes_wasm::compile_cubiomes_wasm;

fn main() {
    tauri_build::build();

    // 自动编译 cubiomes 到 WASM（每次 cubiomes 源码变化时重新编译）
    compile_cubiomes_wasm();
    println!("cargo:rerun-if-changed=cubiomes");
}
