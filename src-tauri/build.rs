//! Build script
//!
//! 1. 调用 Tauri build 生成 IPC 类型与配置
//! 2. 调用 Emscripten (emcc) 将 cubiomes C 源码编译为 WebAssembly
//!
//! ## cubiomes 来源
//! src-tauri/cubiomes/ 是 fork 仓库 https://github.com/MoTeam-cn/cubiomes 的 clone，
//! 支持最新版 MC_26_2（见 biomes.h MCVersion 枚举）。
//! cubiomes_wrapper.c 是项目自有的 WASM 封装层，已提交到 fork 仓库。

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

fn main() {
    tauri_build::build();

    // 自动编译 cubiomes 到 WASM（每次 cubiomes 源码变化时重新编译）
    compile_cubiomes_wasm();
    println!("cargo:rerun-if-changed=cubiomes");
}

/// 编译 cubiomes 到 WASM
fn compile_cubiomes_wasm() {
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
        println!("cargo:warning=Failed to create {}: {}", out_dir.display(), e);
        return;
    }

    println!("cargo:warning=Compiling cubiomes to WebAssembly via emcc...");

    // 源文件清单（与 fork 仓库 CMakeLists.txt 的核心 SOURCES 一致 + 项目自有的 wrapper）
    // 不含 loot/*.c 和 features/stronghold.c（wrapper 未使用 loot 和 stronghold 生成）
    let sources: [&str; 11] = [
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
    ];

    // 导出的 C 函数（前端 worker 通过 ccall/原始 _xxx 调用）
    // _malloc/_free 用于 JS 端分配/释放内存传递 buffer
    // pointer 模式 API（_static 后缀）：结果存入 C 端内部 buffer，
    // JS 端通过 _cubiomes_get_*_pointer 读取，避免每次 _malloc out_buffer
    let exported_functions = "_cubiomes_gen_biomes,\
                              _cubiomes_gen_biomes_with_height,\
                              _cubiomes_gen_biomes_static,\
                              _cubiomes_gen_biomes_with_height_static,\
                              _cubiomes_gen_biomes_at_y,\
                              _cubiomes_gen_biomes_at_y_with_height,\
                              _cubiomes_get_biome_data_pointer,\
                              _cubiomes_get_biome_data_size,\
                              _cubiomes_get_height_data_pointer,\
                              _cubiomes_get_height_data_size,\
                              _cubiomes_get_height_grid_dims,\
                              _cubiomes_init_biome_colors,\
                              _cubiomes_get_all_biome_colors,\
                              _cubiomes_get_image_dimensions,\
                              _cubiomes_free_static_buffers,\
                              _cubiomes_get_structure_pos,\
                              _cubiomes_is_viable,\
                              _cubiomes_get_region_size,\
                              _cubiomes_estimate_spawn,\
                              _cubiomes_first_stronghold,\
                              _cubiomes_find_strongholds,\
                              _cubiomes_is_slime_chunk,\
                              _cubiomes_find_ravines,\
                              _cubiomes_find_nether_fossils,\
                              _cubiomes_find_fossils,\
                              _cubiomes_get_biome_at_point,\
                              _malloc,\
                              _free";

    let mut cmd = Command::new(&emcc_path);
    cmd.args(&sources)
        .args(["-I", "cubiomes"])
        .args(["-O2", "-fwrapv"])
        .args(["-s", "WASM=1"])
        .args(["-s", "MODULARIZE=1"])
        .args(["-s", "EXPORT_NAME=createCubiomesModule"])
        .args(["-s", "ALLOW_MEMORY_GROWTH=1"])
        // ccall/cwrap/HEAPU8/HEAPU32/HEAP32/HEAPF32 是 JS 辅助方法（runtime methods），不是 C 导出函数
        // _malloc/_free 不应放在 RUNTIME_METHODS 中，它们已在 EXPORTED_FUNCTIONS 中声明
        // HEAPU8/HEAPU32/HEAP32：Emscripten 新版默认不把 HEAP 视图暴露到 Module 对象，
        // Worker 内通过 Module.HEAPU8.set() 写入 seed 字符串、通过 new Int32Array(Module.HEAPU8.buffer,...)
        // 读取 cubiomes 输出，必须显式导出
        // HEAPF32：读取 mapApproxHeight 输出的 float 高度数组
        .args(["-s", "EXPORTED_RUNTIME_METHODS=ccall,cwrap,HEAPU8,HEAPU32,HEAP32,HEAPF32"])
        .args(["-s", &format!("EXPORTED_FUNCTIONS={exported_functions}")])
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

    println!(
        "cargo:warning=cubiomes WASM compiled: {} ({} bytes), {} ({} bytes)",
        js_out.display(),
        js_out.metadata().map(|m| m.len()).unwrap_or(0),
        wasm_out.display(),
        wasm_out.metadata().map(|m| m.len()).unwrap_or(0),
    );
}

/// 判断是否需要重新编译：任一源文件比输出文件新，或输出文件不存在
fn needs_recompile(wasm_out: &Path, js_out: &Path) -> bool {
    if !wasm_out.exists() || !js_out.exists() {
        return true;
    }
    let out_mtime = wasm_out
        .metadata()
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    // 检查 cubiomes/ 下所有 .c 和 .h 文件
    let cubiomes_dir = Path::new("cubiomes");
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
            let mtime = path
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            if mtime > out_mtime {
                return true;
            }
        }
    }
    false
}

/// 查找 emcc 可执行文件，返回 (emcc 路径, 需要设置的 env vars)
///
/// env vars 仅在通过 emsdk 路径找到 emcc 时返回（用于设置 EM_CACHE 等）
fn find_emcc() -> Option<(PathBuf, Option<Vec<(&'static str, PathBuf)>>)> {
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
    let candidates = common_emsdk_paths();
    for emsdk in candidates {
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
