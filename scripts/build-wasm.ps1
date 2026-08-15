# 构建 cubiomes WASM（种子地图工具）
#
# 用途：将 cubiomes/ 下的 C 源码编译为 WebAssembly，
#       输出到 src/assets/seedmap/cubiomes.{js,wasm}（前端 Vite 资产目录，
#       dev 由 dev server 提供，build 由 Vite 处理为带 hash 的产物）。
#
# 定位：cubiomes 归前端管理后的唯一编译入口。
#   - 产物维护：GitHub Actions 的 update-cubiomes 工作流自动检测上游更新并编译提交，
#     本地开发环境默认不保留 cubiomes 源码目录
#   - 发布流程：tauri.conf.json beforeBuildCommand 先执行本脚本再跑 Vite build
#   - 手动验证：npm run build:wasm
#
# 容错：本地缺 cubiomes 源码（产物由工作流维护）或 emcc 不可用（未装 emsdk）时，
#       若 src/assets/seedmap 产物已存在则跳过，直接使用入库产物，
#       保证无 Emscripten 环境也能构建。
#
# 增量：所有 .c/.h 源文件都不比产物新时跳过编译（工作流环境有源码时生效）。
#
# 前置条件（需要重编译时）：已安装 emsdk 并激活：
#     cd <emsdk 目录>
#     ./emsdk install latest
#     ./emsdk activate latest
#     ./emsdk_env.ps1   (Windows)

$ErrorActionPreference = "Stop"

# emcc 缓存固定到项目内 .cache/emscripten（默认写 emsdk 目录，无写权限时 emcc 会挂起）
if (-not $env:EM_CACHE) {
    $env:EM_CACHE = Join-Path (Resolve-Path (Join-Path $PSScriptRoot "..")) ".cache/emscripten"
}

# 切换到项目根目录（cubiomes 源码在此目录下）
Set-Location "$PSScriptRoot/.."

# 输出目录：前端 src/assets/seedmap（相对项目根）
$outDir = "src/assets/seedmap"
$wasmOut = Join-Path $outDir "cubiomes.wasm"
$jsOut = Join-Path $outDir "cubiomes.js"

# 源文件清单（与 cubiomes_wrapper.c 封装层一致）
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

# 本地开发默认无 cubiomes 源码（产物由 GitHub Actions 的 update-cubiomes 工作流维护）：
# 缺源码时若有入库产物则直接复用，否则报错提示先运行工作流
$missingSources = @($sources | Where-Object { -not (Test-Path $_) })
if ($missingSources.Count -gt 0) {
    if ((Test-Path $wasmOut) -and (Test-Path $jsOut)) {
        Write-Host "缺少 cubiomes 源码（$($missingSources[0])），复用已入库产物（$wasmOut）" -ForegroundColor Yellow
        exit 0
    }
    Write-Error "缺少 cubiomes 源码（$($missingSources -join ', ')）且无入库产物，请先在 GitHub Actions 运行 update-cubiomes 工作流生成产物。"
    exit 1
}

# 增量判断：产物存在且所有 .c/.h 源文件都不比产物新 → 跳过
function Test-IncrementalSkip {
    if (-not (Test-Path $wasmOut) -or -not (Test-Path $jsOut)) {
        return $false
    }
    $outMtime = (Get-Item $wasmOut).LastWriteTime
    foreach ($s in $sources) {
        $srcFiles = Get-ChildItem -Recurse -File (Split-Path $s) -Filter "*.c" -ErrorAction SilentlyContinue
        $srcFiles += Get-ChildItem -Recurse -File (Split-Path $s) -Filter "*.h" -ErrorAction SilentlyContinue
        foreach ($f in $srcFiles) {
            if ($f.LastWriteTime -gt $outMtime) {
                return $false
            }
        }
    }
    return $true
}

if (Test-IncrementalSkip) {
    Write-Host "cubiomes WASM 已是最新，跳过编译（$wasmOut）" -ForegroundColor Gray
    exit 0
}

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
            # emsdk_env.ps1 调用 python 时 stderr 输出会被 PS5.1 当作 NativeCommandError，
            # 在 ErrorActionPreference=Stop 下直接终止脚本，故激活期间临时降级
            $oldEAP = $ErrorActionPreference
            $ErrorActionPreference = "Continue"
            . $envScript | Out-Null
            $ErrorActionPreference = $oldEAP
            if (Get-Command emcc -ErrorAction SilentlyContinue) {
                $emsdkFound = $true
                break
            }
            $emccDir = Join-Path $p "upstream/emscripten"
            if (Test-Path (Join-Path $emccDir "emcc.exe")) {
                # 直连 emcc：emcc.exe 依赖 EMSDK_NODE/EMSDK_PYTHON/EM_CONFIG，
                # 缺少时 emcc 调用会以 9009 失败
                $env:EMSDK = $p
                $env:EM_CONFIG = Join-Path $p ".emscripten"
                $nodeDir = Get-ChildItem (Join-Path $p "node") -Directory -ErrorAction SilentlyContinue | Select-Object -First 1
                if ($nodeDir) { $env:EMSDK_NODE = Join-Path $nodeDir.FullName "bin/node.exe" }
                $pyDir = Get-ChildItem (Join-Path $p "python") -Directory -ErrorAction SilentlyContinue | Select-Object -First 1
                if ($pyDir) { $env:EMSDK_PYTHON = Join-Path $pyDir.FullName "python.exe" }
                $env:PATH = "$emccDir;$env:PATH"
                $emsdkFound = $true
                Write-Host "emsdk 激活被占用，直连 emcc 目录：$emccDir" -ForegroundColor Yellow
                break
            }
        }
    }
    if (-not $emsdkFound) {
        if ((Test-Path $wasmOut) -and (Test-Path $jsOut)) {
            Write-Host "emcc 不可用，使用已入库的 cubiomes WASM 产物（$wasmOut）" -ForegroundColor Yellow
            exit 0
        }
        Write-Error "emcc not found in PATH and no emsdk installation detected. Please install emsdk."
        exit 1
    }
}

# 导出的 C 函数（前端 worker 通过 _xxx 调用）
# _malloc/_free 用于 JS 端分配/释放内存传递 buffer
$exportedFunctions = "_cubiomes_gen_biomes," +
                     "_cubiomes_gen_biomes_with_height," +
                     "_cubiomes_gen_biomes_static," +
                     "_cubiomes_gen_biomes_with_height_static," +
                     "_cubiomes_gen_biomes_at_y," +
                     "_cubiomes_gen_biomes_at_y_with_height," +
                     "_cubiomes_get_biome_data_pointer," +
                     "_cubiomes_get_biome_data_size," +
                     "_cubiomes_get_height_data_pointer," +
                     "_cubiomes_get_height_data_size," +
                     "_cubiomes_get_height_grid_dims," +
                     "_cubiomes_init_biome_colors," +
                     "_cubiomes_get_all_biome_colors," +
                     "_cubiomes_get_image_dimensions," +
                     "_cubiomes_free_static_buffers," +
                     "_cubiomes_get_structure_pos," +
                     "_cubiomes_is_viable," +
                     "_cubiomes_get_region_size," +
                     "_cubiomes_estimate_spawn," +
                     "_cubiomes_first_stronghold," +
                     "_cubiomes_find_strongholds," +
                     "_cubiomes_is_slime_chunk," +
                     "_cubiomes_find_ravines," +
                     "_cubiomes_find_nether_fossils," +
                     "_cubiomes_find_fossils," +
                     "_cubiomes_get_biome_at_point," +
                     "_malloc,_free"

# 确保输出目录存在
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

Write-Host "Compiling cubiomes to WebAssembly via emcc..." -ForegroundColor Cyan

# 调用 emcc 编译
# 参数说明（与原 build.rs compile_cubiomes_wasm 一致）：
#   -I cubiomes            头文件搜索路径
#   -O2 -fwrapv            优化级别 + 整数溢出回绕（cubiomes 依赖有符号溢出行为）
#   -s WASM=1              输出 WASM 而非 asm.js
#   -s MODULARIZE=1        包装为工厂函数（createCubiomesModule）
#   -s EXPORT_NAME=...     工厂函数名
#   -s ALLOW_MEMORY_GROWTH=1  允许内存自动增长（cubiomes 大范围生成需要）
#   -s MAXIMUM_MEMORY=512MB   显式内存上限：不设时不同 emcc 版本默认不一致，
#                              1.16 layer stack 的 allocCache 拖动累积会 calloc 失败
#   -s EXPORTED_RUNTIME_METHODS=...  导出 JS 辅助方法（ccall/cwrap/HEAP 视图）
#   -s EXPORTED_FUNCTIONS=...        导出 C 函数
emcc @sources `
    -I cubiomes `
    -O2 -fwrapv `
    -s WASM=1 `
    -s MODULARIZE=1 `
    -s EXPORT_NAME=createCubiomesModule `
    -s ALLOW_MEMORY_GROWTH=1 `
    -s MAXIMUM_MEMORY=512MB `
    -s "EXPORTED_RUNTIME_METHODS=ccall,cwrap,HEAPU8,HEAPU32,HEAP32,HEAPF32" `
    -s "EXPORTED_FUNCTIONS=$exportedFunctions" `
    -o $jsOut

if ($LASTEXITCODE -ne 0) {
    Write-Error "emcc compilation failed (exit code $LASTEXITCODE)"
    exit 1
}

$wasmSize = (Get-Item $wasmOut).Length
$jsSize = (Get-Item $jsOut).Length
Write-Host ""
Write-Host "cubiomes WASM compiled successfully:" -ForegroundColor Green
Write-Host "  src/assets/seedmap/cubiomes.js   ($jsSize bytes)" -ForegroundColor Gray
Write-Host "  src/assets/seedmap/cubiomes.wasm ($wasmSize bytes)" -ForegroundColor Gray
