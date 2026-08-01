//! 调用 Emscripten (emcc) 将 cubiomes C 源码编译为 WebAssembly
//!
//! ## cubiomes 来源
//! src-tauri/cubiomes/ 是 fork 仓库 https://github.com/MoTeam-cn/cubiomes 的 clone，
//! 支持最新版 MC_26_2（见 biomes.h MCVersion 枚举）。
//! cubiomes_wrapper.c 是项目自有的 WASM 封装层，已提交到 fork 仓库。
//!
//! ## 增量编译
//! 任一 .c/.h 源文件比已存在的 wasm/js 输出文件新时触发重编译，
//! 否则跳过。cargo:rerun-if-changed=cubiomes 由 build.rs 主入口声明。

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use super::emsdk::find_emcc;

/// 编译 cubiomes 到 WASM
pub fn compile_cubiomes_wasm() {
    let out_dir = PathBuf::from("resources/wasm");
    let wasm_out = out_dir.join("cubiomes.wasm");
    let js_out = out_dir.join("cubiomes.js");

    // 检查是否需要重新编译
    if !needs_recompile(&wasm_out, &js_out) {
        return;
    }

    // 查找 emcc
    let (emcc_path, env_setup) = match find_emcc() {
        Some(x) => x,
        None => {
            println!(
                "cargo:warning=Emscripten (emcc) not found. Skipping cubiomes WASM compilation."
            );
            println!(
                "cargo:warning=Install emsdk (https://emscripten.org/docs/getting_started/downloads.html)"
            );
            println!(
                "cargo:warning=and run `emsdk activate latest && source emsdk_env.sh` (or emsdk_env.ps1 on Windows)."
            );
            return;
        }
    };

    // 确保输出目录存在
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        println!(
            "cargo:warning=Failed to create {}: {}",
            out_dir.display(),
            e
        );
        return;
    }

    eprintln!("[build.rs] Compiling cubiomes to WebAssembly via emcc...");

    let mut cmd = Command::new(&emcc_path);
    cmd.args(sources())
        .args(["-I", "cubiomes"])
        .args(["-O2", "-fwrapv"])
        .args(["-s", "WASM=1"])
        .args(["-s", "MODULARIZE=1"])
        .args(["-s", "EXPORT_NAME=createCubiomesModule"])
        .args(["-s", "ALLOW_MEMORY_GROWTH=1"])
        // 显式设置内存上限 512MB：ALLOW_MEMORY_GROWTH=1 不设 MAXIMUM_MEMORY 时，
        // 不同 emcc 版本默认上限不一致（部分版本默认偏低），导致 1.16 layer stack
        // 的 allocCache 在拖动累积时 calloc 失败返回 NULL，触发 cubiomes 内部
        // memset(NULL) 的 WASM OOB。512MB 对单 Worker 足够（多 Worker 内存独立）。
        .args(["-s", "MAXIMUM_MEMORY=512MB"])
        // ccall/cwrap/HEAPU8/HEAPU32/HEAP32/HEAPF32 是 JS 辅助方法（runtime methods），不是 C 导出函数
        // _malloc/_free 不应放在 RUNTIME_METHODS 中，它们已在 EXPORTED_FUNCTIONS 中声明
        // HEAPU8/HEAPU32/HEAP32：Emscripten 新版默认不把 HEAP 视图暴露到 Module 对象，
        // Worker 内通过 Module.HEAPU8.set() 写入 seed 字符串、通过 new Int32Array(Module.HEAPU8.buffer,...)
        // 读取 cubiomes 输出，必须显式导出
        // HEAPF32：读取 mapApproxHeight 输出的 float 高度数组
        .args([
            "-s",
            "EXPORTED_RUNTIME_METHODS=ccall,cwrap,HEAPU8,HEAPU32,HEAP32,HEAPF32",
        ])
        .args([
            "-s",
            &format!("EXPORTED_FUNCTIONS={}", exported_functions()),
        ])
        .arg("-o")
        .arg(&js_out);

    // 设置 emsdk 需要的环境变量（EM_CACHE 等.）
    if let Some(setup) = env_setup {
        for (k, v) in setup {
            cmd.env(k, v);
        }
    }

    // 用 output() 捕获 stdout/stderr，避免 cargo 管道缓冲区满导致 emcc 阻塞卡死
    // （emcc 编译时输出大量 clang 警告到 stderr，status() 模式下 cargo 的管道写满后
    //  emcc 会阻塞在 write() 上，表现为 build.rs 永久挂起）
    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            println!("cargo:warning=Failed to execute emcc: {}", e);
            return;
        }
    };

    if !output.status.success() {
        println!(
            "cargo:warning=emcc compilation failed (exit code {:?}). WASM not updated.",
            output.status.code()
        );
        // 打印 emcc stderr 帮助诊断（最多 20 行）
        let stderr = String::from_utf8_lossy(&output.stderr);
        for line in stderr.lines().take(20) {
            println!("cargo:warning=emcc: {}", line);
        }
        return;
    }

    eprintln!(
        "[build.rs] cubiomes WASM compiled: {} ({} bytes), {} ({} bytes)",
        js_out.display(),
        js_out.metadata().map(|m| m.len()).unwrap_or(0),
        wasm_out.display(),
        wasm_out.metadata().map(|m| m.len()).unwrap_or(0),
    );
}

/// cubiomes 源文件清单（与 fork 仓库 CMakeLists.txt 的核心 SOURCES 一致 + 项目自有的 wrapper）
///
/// 不含 loot/*.c 和 features/stronghold.c（wrapper 未使用 loot 和 stronghold 生成）
fn sources() -> [&'static str; 11] {
    [
        "cubiomes/biomenoise.c",
        "cubiomes/biomes.c",
        "cubiomes/finders.c",
        "cubiomes/generator.c",
        "cubiomes/layers.c",
        "cubiomes/noise.c",
        "cubiomes/terrainnoise.c",
        "cubiomes/quadbase.c",
        "cubiomes/util.c",
        "cubiomes/xradv.c",
        // cubiomes_wrapper.c 提供高级 API（gen_biomes/get_structure_pos/...），
        // WASM 端通过 ccall 调用这些封装函数，避免 JS 直接操作 cubiomes 内部结构
        "cubiomes/cubiomes_wrapper.c",
    ]
}

/// 导出的 C 函数（前端 worker 通过 ccall/原始 _xxx 调用）
///
/// - `_malloc`/`_free` 用于 JS 端分配/释放内存传递 buffer
/// - pointer 模式 API（`_static` 后缀）：结果存入 C 端内部 buffer，
///   JS 端通过 `_cubiomes_get_*_pointer` 读取，避免每次 _malloc out_buffer
fn exported_functions() -> String {
    let fns = [
        "_cubiomes_gen_biomes",
        "_cubiomes_gen_biomes_with_height",
        "_cubiomes_gen_biomes_static",
        "_cubiomes_gen_biomes_with_height_static",
        "_cubiomes_gen_biomes_at_y",
        "_cubiomes_gen_biomes_at_y_with_height",
        "_cubiomes_get_biome_data_pointer",
        "_cubiomes_get_biome_data_size",
        "_cubiomes_get_height_data_pointer",
        "_cubiomes_get_height_data_size",
        "_cubiomes_get_height_grid_dims",
        "_cubiomes_init_biome_colors",
        "_cubiomes_get_all_biome_colors",
        "_cubiomes_get_image_dimensions",
        "_cubiomes_free_static_buffers",
        "_cubiomes_get_structure_pos",
        "_cubiomes_is_viable",
        "_cubiomes_get_region_size",
        "_cubiomes_estimate_spawn",
        "_cubiomes_first_stronghold",
        "_cubiomes_find_strongholds",
        "_cubiomes_is_slime_chunk",
        "_cubiomes_find_ravines",
        "_cubiomes_find_nether_fossils",
        "_cubiomes_find_fossils",
        "_cubiomes_get_biome_at_point",
        "_malloc",
        "_free",
    ];
    fns.join(",")
}

/// 判断是否需要重新编译：任一源文件比输出文件新，或输出文件不存在
///
/// 同时为每个 .c/.h 源文件声明 `cargo:rerun-if-changed`，
/// 因为 `cargo:rerun-if-changed=cubiomes` 只检查目录本身时间戳（文件增删），
/// 不检查目录内文件的内容修改。
fn needs_recompile(wasm_out: &Path, js_out: &Path) -> bool {
    if !wasm_out.exists() || !js_out.exists() {
        return true;
    }
    let out_mtime = wasm_out
        .metadata()
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    // 检查 cubiomes/ 下所有 .c 和 .h 文件，并为每个文件声明 rerun-if-changed
    let cubiomes_dir = Path::new("cubiomes");
    let mut needs = false;
    if let Ok(entries) = std::fs::read_dir(cubiomes_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let is_source = path
                .extension()
                .map(|ext| ext == "c" || ext == "h")
                .unwrap_or(false);
            if !is_source {
                continue;
            }
            // 声明 rerun-if-changed=<file>，确保文件内容修改触发 build.rs 重跑
            if let Some(s) = path.to_str() {
                println!("cargo:rerun-if-changed={}", s);
            }
            let mtime = path
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            if mtime > out_mtime {
                needs = true;
            }
        }
    }
    needs
}
