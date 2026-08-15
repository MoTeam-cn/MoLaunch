// 构建前置：先编译 cubiomes WASM，再执行 Vite build。
// - Windows：走 scripts/build-wasm.ps1（含增量判断；本地无 cubiomes 源码或 emcc 缺失时复用入库产物）
// - 其他平台：无 emsdk 环境，直接复用仓库已提交的 src/assets/seedmap 产物
import { spawnSync } from 'node:child_process'
import { existsSync } from 'node:fs'

const jsOut = 'src/assets/seedmap/cubiomes.js'
const wasmOut = 'src/assets/seedmap/cubiomes.wasm'

if (process.platform === 'win32') {
  // npm 脚本（含 .cmd 批处理）必须经 cmd 运行，Node 直接 spawn 会 ENOENT
  const r = spawnSync(process.env.ComSpec || 'cmd.exe', ['/c', 'npm run build:wasm'], { stdio: 'inherit' })
  if (r.status !== 0) process.exit(r.status ?? 1)
} else if (!existsSync(jsOut) || !existsSync(wasmOut)) {
  console.error(
    '[prebuild] 缺少 src/assets/seedmap/cubiomes.{js,wasm}，且非 Windows 平台无法编译，请先在 Windows 上执行 npm run build:wasm',
  )
  process.exit(1)
}
