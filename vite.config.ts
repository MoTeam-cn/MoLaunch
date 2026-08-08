import { defineConfig, type Plugin } from 'vite'
import vue from '@vitejs/plugin-vue'
import { execFileSync } from 'node:child_process'
import { resolve } from 'path'
import pkg from './package.json'

/**
 * 基于 git 生成「本次更新」日志
 *
 * 仓库用 tag 管理版本（tag 名即版本号，如 v0.3.4）：内容取「上一 tag → 最新 tag」
 * 之间的全部 commit message（剥离 CI 跳过标记 !c），生成 Markdown 供 ReleaseTimeline 渲染；
 * 无 tag 时回退到最近 20 条 commit。git 不可用时返回空内容。
 */
function gitUpdateLog(): { version: string; content: string } {
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
  if (subjects.length === 0) return { version, content: '' }
  return {
    version,
    content: [`## MoLaunch ${version}`, ...subjects.map((s) => `- ${s}`)].join('\n'),
  }
}

/**
 * 虚拟模块 `virtual:update-log`：构建时读取 git 提交历史生成「本次更新」日志。
 * 避免引入 CHANGELOG 依赖，也不把完整历史打进前端包，仅内联上一版本到当前版本的 commit 列表。
 */
function updateLogPlugin(): Plugin {
  return {
    name: 'molaunch:update-log',
    resolveId(id) {
      if (id === 'virtual:update-log') return '\0molaunch:update-log'
    },
    load(id) {
      if (id !== '\0molaunch:update-log') return
      const { version, content } = gitUpdateLog()
      return `export const version = ${JSON.stringify(version)}\nexport default ${JSON.stringify(content)}`
    },
  }
}

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue(), updateLogPlugin()],
  define: {
    // 注入应用版本号（来自 package.json），供「其他」页版本号展示与开发者模式解锁使用
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  optimizeDeps: {
    // 显式指定依赖预构建扫描入口，避免 vite 默认扫描整个项目根目录
    // code-libs/arco-design-vue-main 是 Arco Design Vue 源码副本（仅查阅用，已被 .gitignore 排除），
    // 其中 packages/arco-vue-docs 引用了 vue-i18n / @web-vue/* / @arco-design/arco-vue-docs-navbar
    // 等未安装的依赖，不限定扫描范围会导致 dev 启动时依赖预构建报错
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
    // 禁用资源内联（base64）：所有资源（图片/字体等）一律输出为独立文件
    // 默认 4096 字节以下的资源会被 Vite 内联为 base64，污染 JS chunk 且无法被浏览器缓存
    // Tauri 应用走本地文件系统加载，无 HTTP 请求开销，独立文件更利于缓存与调试
    assetsInlineLimit: 0,
    // 手动 chunk 分离策略：把第三方依赖按「稳定性 + 用途」拆分，
    // 避免单个业务 chunk 堆积多个大依赖（如 ol/skinview3d 跑到 Home.js）
    rollupOptions: {
      output: {
        // 入口 JS 文件输出路径
        entryFileNames: 'assets/js/[name]-[hash].js',
        // 共享 chunk 输出路径
        // 清理 Vue 组件的丑陋后缀：AlertV2.vue_vue_type_script_setup_true_lang → AlertV2
        chunkFileNames: (chunkInfo) => {
          let name = chunkInfo.name
          // 移除 Vue 单文件组件的 lang 后缀（Rollup 自动生成的标记）
          name = name.replace(/\.vue_.*$/, '')
          // 移除 .ts 后缀
          name = name.replace(/\.ts$/, '')
          // 移除其他非法字符（统一替换为下划线）
          name = name.replace(/[^\w-]/g, '_')
          return `assets/js/${name}-[hash].js`
        },
        // 静态资源输出路径：按扩展名分类
        // .css → assets/css/，.js → assets/js/，其他（图片/字体/媒体）→ assets/ 根目录
        // 注意：Vite 5.4 中 webp 等资源通过 import.meta.glob 的 ?url 模式导入时会触发双输出
        // （assetFileNames 指定路径 + 默认 assets/ 路径各一份），这是 Vite asset plugin 的已知行为。
        // 因此图片等普通资源直接输出到 assets/ 根目录，与默认输出位置一致，避免双输出。
        assetFileNames: (assetInfo) => {
          const fileName = assetInfo.name ?? ''
          const ext = fileName.split('.').pop()?.toLowerCase() ?? ''
          if (ext === 'css') return 'assets/css/[name]-[hash].[ext]'
          if (['js', 'mjs', 'cjs'].includes(ext)) return 'assets/js/[name]-[hash].[ext]'
          // @lobehub/icons-static-svg 品牌图标统一输出到 assets/@lobehub/ 目录，便于识别与管理
          if (assetInfo.originalFileNames?.some((f) => f.includes('@lobehub/icons-static-svg'))) {
            return 'assets/@lobehub/[name]-[hash].[ext]'
          }
          return 'assets/[name]-[hash].[ext]'
        },
        manualChunks(id) {
          // 仅处理 node_modules 中的依赖，业务代码走默认拆分
          if (!id.includes('node_modules')) return undefined

          // vendor-vue：Vue 框架核心（vue / vue-router / pinia / vue-demi）
          // 这部分几乎不变，独立 chunk 利于长期缓存
          if (
            id.includes('node_modules/vue/') ||
            id.includes('node_modules/@vue/') ||
            id.includes('node_modules/vue-router/') ||
            id.includes('node_modules/pinia/') ||
            id.includes('node_modules/vue-demi/')
          ) {
            return 'vendor-vue'
          }

          // vendor-tauri：Tauri JS 桥接层
          // 独立 chunk 避免与业务逻辑混在一起，更新 Tauri 版本时只更新这个 chunk
          if (id.includes('node_modules/@tauri-apps/')) {
            return 'vendor-tauri'
          }

          // vendor-ol：OpenLayers 地图库及其依赖（rbush / quickselect）
          // 仅 Tools 种子地图使用，独立 chunk 后只有进入 Tools 页才加载
          if (
            id.includes('node_modules/ol/') ||
            id.includes('node_modules/rbush/') ||
            id.includes('node_modules/quickselect/')
          ) {
            return 'vendor-ol'
          }

          // vendor-skinview3d：3D 皮肤预览库及其依赖（three / skinview-utils）
          // 仅 SkinModel3D 组件使用，独立 chunk 后只有打开皮肤管理时才加载
          if (
            id.includes('node_modules/skinview3d/') ||
            id.includes('node_modules/three/') ||
            id.includes('node_modules/skinview-utils/')
          ) {
            return 'vendor-skinview3d'
          }

          // vendor-heroicons：Heroicons 图标库（按需导入，650+ 图标文件）
          // 独立 chunk 避免图标代码堆积到业务 chunk，利于浏览器缓存
          if (id.includes('node_modules/@heroicons/')) {
            return 'vendor-heroicons'
          }

          // vendor-misc：其他第三方依赖兜底
          return 'vendor-misc'
        },
      },
    },
  },
})
