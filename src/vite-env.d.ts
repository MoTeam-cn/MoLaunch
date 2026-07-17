/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

// 应用版本号（由 vite.config.ts define 注入，来自 package.json version）
declare const __APP_VERSION__: string
