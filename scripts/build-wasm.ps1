# 手动构建 cubiomes WASM（种子地图工具）
#
# 用途：将 src-tauri/cubiomes/ 下的 C 源码编译为 WebAssembly，
#       输出到 src-tauri/resources/wasm/cubiomes.{js,wasm}。
#
# 何时运行：
#   - 首次拉取项目后（若 resources/wasm/cubiomes.wasm 缺失）
#   - 替换为原站 minecraftsearch.com 修改版 cubiomes 源码后
#   - 修改 cubiomes_wrapper.c 或 cubiomes C 源码后
#
# 前置条件：
#   已安装 emsdk 并激活：
#     cd <emsdk 目录>
#     ./emsdk install latest
#     ./emsdk activate latest
#     ./emsdk_env.ps1   (Windows)
#
# 运行方式：
#   npm run build:wasm
#
# 注意：build.rs 已恢复自动编译（cargo build 会触发）。
# 此脚本作为手动构建入口，便于单独验证 WASM 编译是否成功。

$ErrorActionPreference = "Stop"

# 切换到 src-tauri 目录（cubiomes 源码与 resources/wasm 均在此目录下）
Set-Location "$PSScriptRoot/../src-tauri"

# 自动检测并激活 emsdk 环境（若 emcc 不在 PATH）
if (-not (Get-Command emcc -ErrorAction SilentlyContinue)) {
    $emsdkCandidates = @(
        "$env:USERPROFILE\Desktop\emsdk",
        "$env:USERPROFILE\emsdk",
        "C:\emsdk",
        "D:\emsdk"
    )
    $emsdkFound = $false
    foreach ($p in $emsdkCandidates) {
        $envScript = Join-Path $p "emsdk_env.ps1"
        if (Test-Path $envScript) {
            Write-Host "Activating emsdk at $p ..." -ForegroundColor Cyan
            . $envScript
            $emsdkFound = $true
            break
        }
    }
    if (-not $emsdkFound) {
        Write-Error "emcc not found in PATH and no emsdk installation detected. Please install emsdk."
        exit 1
    }
}

# 源文件清单（与 build.rs compile_cubiomes_wasm 的 sources 一致）
$sources = @(
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
    # cubiomes_wrapper.c 提供高级 API（gen_biomes/get_structure_pos/...），
    # WASM 端通过 ccall 调用这些封装函数
    "cubiomes/cubiomes_wrapper.c"
)

# 导出的 C 函数（前端 worker 通过 _xxx 调用）
# _malloc/_free 用于 JS 端分配/释放内存传递 buffer
$exportedFunctions = "_cubiomes_gen_biomes," +
                     "_cubiomes_gen_biomes_with_height," +
                     "_cubiomes_get_structure_pos," +
                     "_cubiomes_is_viable," +
                     "_cubiomes_get_region_size," +
                     "_cubiomes_estimate_spawn," +
                     "_cubiomes_first_stronghold," +
                     "_malloc,_free"

# 确保输出目录存在
New-Item -ItemType Directory -Force -Path "resources/wasm" | Out-Null

Write-Host "Compiling cubiomes to WebAssembly via emcc..." -ForegroundColor Cyan

# 调用 emcc 编译
# 参数说明（与 build.rs 一致）：
#   -I cubiomes            头文件搜索路径
#   -O2 -fwrapv            优化级别 + 整数溢出回绕（cubiomes 依赖有符号溢出行为）
#   -s WASM=1              输出 WASM 而非 asm.js
#   -s MODULARIZE=1        包装为工厂函数（createCubiomesModule）
#   -s EXPORT_NAME=...     工厂函数名
#   -s ALLOW_MEMORY_GROWTH=1  允许内存自动增长（cubiomes 大范围生成需要）
#   -s EXPORTED_RUNTIME_METHODS=...  导出 JS 辅助方法（ccall/cwrap/HEAP 视图）
#   -s EXPORTED_FUNCTIONS=...        导出 C 函数
emcc @sources `
    -I cubiomes `
    -O2 -fwrapv `
    -s WASM=1 `
    -s MODULARIZE=1 `
    -s EXPORT_NAME=createCubiomesModule `
    -s ALLOW_MEMORY_GROWTH=1 `
    -s "EXPORTED_RUNTIME_METHODS=ccall,cwrap,HEAPU8,HEAPU32,HEAP32,HEAPF32" `
    -s "EXPORTED_FUNCTIONS=$exportedFunctions" `
    -o resources/wasm/cubiomes.js

if ($LASTEXITCODE -ne 0) {
    Write-Error "emcc compilation failed (exit code $LASTEXITCODE)"
    exit 1
}

$wasmSize = (Get-Item "resources/wasm/cubiomes.wasm").Length
$jsSize = (Get-Item "resources/wasm/cubiomes.js").Length
Write-Host ""
Write-Host "cubiomes WASM compiled successfully:" -ForegroundColor Green
Write-Host "  resources/wasm/cubiomes.js   ($jsSize bytes)" -ForegroundColor Gray
Write-Host "  resources/wasm/cubiomes.wasm ($wasmSize bytes)" -ForegroundColor Gray
