/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

// 应用版本号（由 vite.config.ts define 注入，来自 package.json version）
declare const __APP_VERSION__: string

// 启动更新日志内容（由 vite.config.ts updateLogPlugin 虚拟模块注入，
// 仅内联 CHANGELOG.md 中当前版本对应的段落，避免整份 Markdown 打进前端包）
declare module 'virtual:update-log' {
  export const version: string
  /** 作者的话列表（vite 构建时从版本区间内 `note:` 前缀的 commit 提取，可为空数组） */
  export const notes: string[]
  const content: string
  export default content
}

// Markdown 文件原始内容导入（Vite ?raw 后缀）
declare module '*.md?raw' {
  const content: string
  export default content
}
