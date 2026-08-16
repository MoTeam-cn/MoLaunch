// 构建 cubiomes WASM（种子地图工具）
//
// 用途：将 cubiomes/ 下的 C 源码编译为 WebAssembly，输出到 src/assets/seedmap/cubiomes.{js,wasm}
// （前端 Vite 资产目录）。行为：容错（缺源码/emcc 时复用入库产物）、增量跳过、emsdk 自动激活。
// 由 npm run build:wasm 调用；仅依赖 Node 18+ 内置 API，无第三方包。

const { spawnSync } = require('node:child_process')
const fs = require('node:fs')
const path = require('node:path')

process.chdir(path.resolve(__dirname, '..'))
const projectRoot = process.cwd()

// emcc 缓存固定到项目内 .cache/emscripten（默认写 emsdk 目录，无写权限时 emcc 会挂起）
const env = { ...process.env }
if (!env.EM_CACHE) env.EM_CACHE = path.join(projectRoot, '.cache', 'emscripten')

const outDir = 'src/assets/seedmap'
const wasmOut = path.join(outDir, 'cubiomes.wasm')
const jsOut = path.join(outDir, 'cubiomes.js')

// 源文件清单（与 cubiomes_wrapper.c 封装层一致）
const sources = [
  'cubiomes/biomenoise.c',
  'cubiomes/biomes.c',
  'cubiomes/finders.c',
  'cubiomes/generator.c',
  'cubiomes/layers.c',
  'cubiomes/noise.c',
  'cubiomes/terrainnoise.c',
  'cubiomes/quadbase.c',
  'cubiomes/util.c',
  'cubiomes/xradv.c',
  'cubiomes/cubiomes_wrapper.c',
]

// 本地开发默认无 cubiomes 源码（产物由 GitHub Actions 的 update-cubiomes 工作流维护）：
// 缺源码时若有入库产物则直接复用，否则报错提示先运行工作流
const missingSources = sources.filter((s) => !fs.existsSync(s))
if (missingSources.length > 0) {
  if (fs.existsSync(wasmOut) && fs.existsSync(jsOut)) {
    console.log(`缺少 cubiomes 源码（${missingSources[0]}），复用已入库产物（${wasmOut}）`)
    process.exit(0)
  }
  console.error(`缺少 cubiomes 源码（${missingSources.join(', ')}）且无入库产物，请先在 GitHub Actions 运行 update-cubiomes 工作流生成产物。`)
  process.exit(1)
}

// 增量判断：产物存在且所有 .c/.h 源文件都不比产物新 → 跳过
function isIncrementalSkip() {
  if (!fs.existsSync(wasmOut) || !fs.existsSync(jsOut)) return false
  const outMtime = fs.statSync(wasmOut).mtimeMs
  for (const s of sources) {
    const base = path.dirname(s)
    const files = []
    ;(function collect(dir) {
      for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name)
        if (entry.isDirectory()) collect(full)
        else if (/\.(c|h)$/.test(entry.name)) files.push(full)
      }
    })(base)
    for (const f of files) {
      if (fs.statSync(f).mtimeMs > outMtime) return false
    }
  }
  return true
}

if (isIncrementalSkip()) {
  console.log(`cubiomes WASM 已是最新，跳过编译（${wasmOut}）`)
  process.exit(0)
}

// 自动检测并激活 emsdk 环境（若 emcc 不在 PATH）：找到后返回 emcc 可执行文件路径或命令名
function resolveEmcc() {
  const checkPath = (cmd) => {
    const which = spawnSync(process.platform === 'win32' ? 'where' : 'which', [cmd], { encoding: 'utf8' })
    if (which.status === 0) return true
    return false
  }
  if (checkPath('emcc')) return 'emcc'

  const home = process.env.USERPROFILE || process.env.HOME || ''
  const candidates = [
    path.join(home, 'Desktop', 'emsdk'),
    path.join(home, 'emsdk'),
    'C:/emsdk',
    'D:/emsdk',
    '/usr/local/emsdk',
  ]
  for (const p of candidates) {
    const emccDir = path.join(p, 'upstream', 'emscripten')
    if (!fs.existsSync(emccDir)) continue
    // 直连 emcc：emcc 依赖 EMSDK_NODE/EMSDK_PYTHON/EM_CONFIG，缺失时调用会失败
    env.EMSDK = p
    env.EM_CONFIG = path.join(p, '.emscripten')
    const binName = process.platform === 'win32' ? 'node.exe' : 'node'
    const pyName = process.platform === 'win32' ? 'python.exe' : 'python'
    const nodeDir = fs.existsSync(path.join(p, 'node'))
      ? fs.readdirSync(path.join(p, 'node'), { withFileTypes: true }).find((d) => d.isDirectory())
      : null
    const pyDir = fs.existsSync(path.join(p, 'python'))
      ? fs.readdirSync(path.join(p, 'python'), { withFileTypes: true }).find((d) => d.isDirectory())
      : null
    if (nodeDir) env.EMSDK_NODE = path.join(p, 'node', nodeDir.name, 'bin', binName)
    if (pyDir) env.EMSDK_PYTHON = path.join(p, 'python', pyDir.name, 'bin', pyName)
    env.PATH = emccDir + (process.platform === 'win32' ? ';' : ':') + env.PATH

    const emccNames = process.platform === 'win32'
      ? ['emcc.exe', 'emcc.bat', 'emcc.cmd']
      : ['emcc', 'emcc.py']
    const emcc = emccNames.find((n) => fs.existsSync(path.join(emccDir, n)))
    if (emcc) {
      console.log(`emsdk 激活被占用，直连 emcc 目录：${emccDir}`)
      return process.platform === 'win32' ? path.join(emccDir, emcc) : path.join(emccDir, emcc)
    }
  }
  return null
}

const emccBin = resolveEmcc()
if (!emccBin) {
  if (fs.existsSync(wasmOut) && fs.existsSync(jsOut)) {
    console.log(`emcc 不可用，使用已入库的 cubiomes WASM 产物（${wasmOut}）`)
    process.exit(0)
  }
  console.error('emcc not found in PATH and no emsdk installation detected. Please install emsdk.')
  process.exit(1)
}

// 导出的 C 函数（前端 worker 通过 _xxx 调用）；_malloc/_free 用于 JS 端分配/释放内存传递 buffer
const exportedFunctions = [
  '_cubiomes_gen_biomes',
  '_cubiomes_gen_biomes_with_height',
  '_cubiomes_gen_biomes_static',
  '_cubiomes_gen_biomes_with_height_static',
  '_cubiomes_gen_biomes_at_y',
  '_cubiomes_gen_biomes_at_y_with_height',
  '_cubiomes_get_biome_data_pointer',
  '_cubiomes_get_biome_data_size',
  '_cubiomes_get_height_data_pointer',
  '_cubiomes_get_height_data_size',
  '_cubiomes_get_height_grid_dims',
  '_cubiomes_init_biome_colors',
  '_cubiomes_get_all_biome_colors',
  '_cubiomes_get_image_dimensions',
  '_cubiomes_free_static_buffers',
  '_cubiomes_get_structure_pos',
  '_cubiomes_is_viable',
  '_cubiomes_get_region_size',
  '_cubiomes_estimate_spawn',
  '_cubiomes_first_stronghold',
  '_cubiomes_find_strongholds',
  '_cubiomes_is_slime_chunk',
  '_cubiomes_find_ravines',
  '_cubiomes_find_nether_fossils',
  '_cubiomes_find_fossils',
  '_cubiomes_get_biome_at_point',
  '_malloc,_free',
].join(',')

// 确保输出目录存在
fs.mkdirSync(outDir, { recursive: true })

console.log('Compiling cubiomes to WebAssembly via emcc...')

// 调用 emcc 编译：
//   -I cubiomes            头文件搜索路径
//   -O2 -fwrapv            优化级别 + 整数溢出回绕（cubiomes 依赖有符号溢出行为）
//   -s WASM=1              输出 WASM 而非 asm.js
//   -s MODULARIZE=1        包装为工厂函数（createCubiomesModule）
//   -s EXPORT_NAME=...     工厂函数名
//   -s ALLOW_MEMORY_GROWTH=1  允许内存自动增长（cubiomes 大范围生成需要）
//   -s MAXIMUM_MEMORY=512MB   显式内存上限：不设时不同 emcc 版本默认不一致，
//                              1.16 layer stack 的 allocCache 拖动累积会 calloc 失败
//   -s EXPORTED_RUNTIME_METHODS=...  导出 JS 辅助方法（ccall/cwrap/HEAP 视图）
//   -s EXPORTED_FUNCTIONS=...        导出 C 函数
const args = [
  ...sources,
  '-I', 'cubiomes',
  '-O2', '-fwrapv',
  '-s', 'WASM=1',
  '-s', 'MODULARIZE=1',
  '-s', 'EXPORT_NAME=createCubiomesModule',
  '-s', 'ALLOW_MEMORY_GROWTH=1',
  '-s', 'MAXIMUM_MEMORY=512MB',
  '-s', 'EXPORTED_RUNTIME_METHODS=ccall,cwrap,HEAPU8,HEAPU32,HEAP32,HEAPF32',
  '-s', `EXPORTED_FUNCTIONS=${exportedFunctions}`,
  '-o', jsOut,
]

// Windows 下 emcc 为 .bat/.cmd，必须经 cmd.exe 运行；Unix 直接执行
const r = process.platform === 'win32'
  ? spawnSync('cmd.exe', ['/c', emccBin, ...args], { stdio: 'inherit', env, cwd: projectRoot })
  : spawnSync(emccBin, args, { stdio: 'inherit', env, cwd: projectRoot })

if (r.status !== 0) {
  console.error(`emcc compilation failed (exit code ${r.status})`)
  process.exit(1)
}

const wasmSize = fs.statSync(wasmOut).size
const jsSize = fs.statSync(jsOut).size
console.log('')
console.log('cubiomes WASM compiled successfully:')
console.log(`  src/assets/seedmap/cubiomes.js   (${jsSize} bytes)`)
console.log(`  src/assets/seedmap/cubiomes.wasm (${wasmSize} bytes)`)
