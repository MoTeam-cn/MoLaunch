import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import { useSettingsStore } from './stores/settings'
import { usePluginStore } from './stores/plugins'
import { renderNonTauriWarning } from './utils/checkTauriEnv'
import './assets/styles/main.css'
import 'ol/ol.css'

// 前端 JS 入口最早可执行点：此时 WebView2 已完成 HTML/JS bundle 加载
// 与后端 setup() hook 的时间差 = WebView2 加载 localhost:1420 + JS bundle 解析耗时
console.log('[Startup][Frontend] main.ts entered (JS bundle parsed) @', new Date().toISOString())

// ===== Tauri 环境检测 =====
// 非 Tauri 环境（浏览器直接打开 dev server）时渲染友好提示并阻止 Vue app 挂载
if (renderNonTauriWarning()) {
  // 已渲染提示，阻止后续挂载
} else {
  const app = createApp(App)
  const pinia = createPinia()
  app.use(pinia)
  app.use(router)

  // 在 mount 前触发 settingsStore.loadSettings()，把 primaryColor 注入 CSS 变量
  // 避免首屏使用默认蓝色然后闪烁到用户自定义色
  useSettingsStore(pinia)

  // 初始化插件 store（从 localStorage 同步加载启用状态与 homePanelMode）
  // 后端 INI 同步在 mount 后异步进行，不阻塞首屏
  const pluginStore = usePluginStore(pinia)

  console.log('[Startup][Frontend] Vue app created, Pinia/router installed @', new Date().toISOString())

  app.mount('#app')
  console.log('[Startup][Frontend] app.mount("#app") called @', new Date().toISOString())

  // mount 后异步从后端 INI 同步插件配置（[Plugin] 节）
  // 失败时静默回退到 localStorage，不影响首屏
  pluginStore.syncFromBackend().catch((e) => {
    console.warn('[Startup][Frontend] Plugin backend sync failed:', e)
  })
}
