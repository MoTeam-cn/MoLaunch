/// <reference types="vitest/config" />
import { defineConfig, type Plugin } from 'vite'
import vue from '@vitejs/plugin-vue'
import { execFileSync } from 'node:child_process'
import { resolve } from 'path'
import pkg from './package.json'

/**
 * 基于 git 生成「本次更新」日志：取「上一 tag → 最新 tag」间的 commit（剥离 !c 标记），
 * `note:` 前缀的 commit 作为作者寄语单独提取；无 tag 时回退最近 20 条。git 不可用返回空。
 */
function gitUpdateLog(): { version: string; content: string; notes: string[] } {
  const run = (args: string[]): string[] => {
    try {
      return execFileSync('git', args, { encoding: 'utf8' })
        .split('\n')
        .map((s) => s.trim())
        .filter(Boolean)
    } catch {
      return []
    }
  }
  const tags = run(['tag', '--sort=-v:refname'])
  let logs: string[] = []
  let version = pkg.version
  if (tags.length === 0) {
    logs = run(['log', '-20', '--pretty=format:%s'])
  } else {
    const latest = tags[0]
    const prev = tags[1] ?? null
    logs = run(['log', prev ? `${prev}..${latest}` : latest, '--pretty=format:%s'])
    version = latest.replace(/^v/, '')
  }
  const subjects = logs.map((s) => s.replace(/\s*!c\s*$/i, '').trim()).filter(Boolean)
  const noteRe = /^note:\s*/i
  const notes = subjects
    .filter((s) => noteRe.test(s))
    .map((s) => s.replace(noteRe, '').trim())
  const commits = subjects.filter((s) => !noteRe.test(s))
  if (commits.length === 0 && notes.length === 0) return { version, content: '', notes: [] }
  return {
    version,
    notes,
    content: [`## MoLaunch ${version}`, ...commits.map((s) => `- ${s}`)].join('\n'),
  }
}

/** 虚拟模块 `virtual:update-log`：构建时内联上一版本到当前版本的 commit 列表，避免引入 CHANGELOG 依赖 */
function updateLogPlugin(): Plugin {
  return {
    name: 'molaunch:update-log',
    resolveId(id) {
      if (id === 'virtual:update-log') return '\0molaunch:update-log'
    },
    load(id) {
      if (id !== '\0molaunch:update-log') return
      const { version, content, notes } = gitUpdateLog()
      return `export const version = ${JSON.stringify(
        version,
      )}\nexport const notes = ${JSON.stringify(notes)}\nexport default ${JSON.stringify(content)}`
    },
  }
}

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue(), updateLogPlugin()],
  test: {
    // 仅扫描 src 下的测试，排除工作区内的第三方源码目录
    include: ['src/**/*.test.ts'],
  },
  define: {
    // 注入应用版本号（来自 package.json），供「其他」页展示与开发者模式解锁
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  optimizeDeps: {
    // 限定依赖预构建扫描入口，避免误扫工作区内未安装依赖的目录
    entries: ['index.html', 'src/main.ts'],
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  // Tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
  },
  // Env variables starting with the item of `envPrefix` will be exposed in tauri's source code through `import.meta.env`.
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  esbuild: {
    // 生产构建剥离 console.* 和 debugger，防止 PII 泄露（CWE-532）
    drop: process.env.NODE_ENV === 'production' ? ['console', 'debugger'] : [],
  },
  build: {
    // Tauri uses Chromium on Windows and Webkit on macOS and Linux
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    // don't minify for debug builds
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    // 资源不内联 base64：Tauri 走本地文件加载，独立文件利于缓存与调试
    assetsInlineLimit: 0,
    rollupOptions: {
      output: {
        entryFileNames: 'assets/js/[name]-[hash].js',
        // 共享 chunk 输出路径，清理 Vue 组件/TS 的 Rollup 自动后缀
        chunkFileNames: (chunkInfo) => {
          const name = chunkInfo.name
            .replace(/\.vue_.*$/, '')
            .replace(/\.ts$/, '')
            .replace(/[^\w-]/g, '_')
          return `assets/js/${name}-[hash].js`
        },
        // 静态资源按扩展名分类输出；@lobehub 品牌图标统一放 assets/@lobehub/ 便于识别
        assetFileNames: (assetInfo) => {
          const fileName = assetInfo.name ?? ''
          const ext = fileName.split('.').pop()?.toLowerCase() ?? ''
          if (ext === 'css') return 'assets/css/[name]-[hash].[ext]'
          if (['js', 'mjs', 'cjs'].includes(ext)) return 'assets/js/[name]-[hash].[ext]'
          if (assetInfo.originalFileNames?.some((f) => f.includes('@lobehub/icons-static-svg'))) {
            return 'assets/@lobehub/[name]-[hash].[ext]'
          }
          return 'assets/[name]-[hash].[ext]'
        },
        manualChunks(id) {
          // 仅处理 node_modules 中的依赖，业务代码走默认拆分
          if (!id.includes('node_modules')) return undefined

          // vendor-vue：Vue 框架核心，独立 chunk 利于长期缓存
          if (
            id.includes('node_modules/vue/') ||
            id.includes('node_modules/@vue/') ||
            id.includes('node_modules/vue-router/') ||
            id.includes('node_modules/pinia/') ||
            id.includes('node_modules/vue-demi/')
          ) {
            return 'vendor-vue'
          }

          // vendor-tauri：Tauri JS 桥接层，更新 Tauri 时只更新此 chunk
          if (id.includes('node_modules/@tauri-apps/')) {
            return 'vendor-tauri'
          }

          // vendor-ol：OpenLayers 地图库，仅 Tools 种子地图使用
          if (
            id.includes('node_modules/ol/') ||
            id.includes('node_modules/rbush/') ||
            id.includes('node_modules/quickselect/')
          ) {
            return 'vendor-ol'
          }

          // vendor-skinview3d：3D 皮肤预览库，仅皮肤管理使用
          if (
            id.includes('node_modules/skinview3d/') ||
            id.includes('node_modules/three/') ||
            id.includes('node_modules/skinview-utils/')
          ) {
            return 'vendor-skinview3d'
          }

          // vendor-heroicons：Heroicons 图标库（按需导入，650+ 图标文件）
          if (id.includes('node_modules/@heroicons/')) {
            return 'vendor-heroicons'
          }

          // vendor-markdown：Markdown 渲染栈（marked + dompurify），AI 聊天与更新日志共用
          if (
            id.includes('node_modules/marked/') ||
            id.includes('node_modules/dompurify/')
          ) {
            return 'vendor-markdown'
          }

          // vendor-misc：其他第三方依赖兜底
          return 'vendor-misc'
        },
      },
    },
  },
})
