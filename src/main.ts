import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import { useSettingsStore } from './stores/settings'
import './assets/styles/main.css'

// 前端 JS 入口最早可执行点：此时 WebView2 已完成 HTML/JS bundle 加载
// 与后端 setup() hook 的时间差 = WebView2 加载 localhost:1420 + JS bundle 解析耗时
console.log('[Startup][Frontend] main.ts entered (JS bundle parsed) @', new Date().toISOString())

// ===== Tauri 环境检测 =====
// 浏器直接打开 dev server 时，@tauri-apps/api 的 getCurrentWindow() 会抛
// "Cannot read properties of undefined (reading 'metadata')"，导致 TopNavLayout
// setup 阶段崩溃。此处最早拦截，渲染友好提示并阻止 Vue app 挂载。
//
// Tauri 2 在 WebView 中会注入 window.__TAURI_INTERNALS__，浏览器环境无此对象。
if (!(window as any).__TAURI_INTERNALS__) {
  const root = document.getElementById('app')
  if (root) {
    root.innerHTML = `
      <div style="position:fixed;inset:0;display:flex;align-items:center;justify-content:center;
                  background:linear-gradient(135deg,#f0f5ff 0%,#e0ecff 100%);
                  font-family:-apple-system,BlinkMacSystemFont,'Segoe UI','Microsoft YaHei',sans-serif;
                  color:#1f2937;text-align:center;padding:24px;">
        <div style="max-width:480px;">
          <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none"
               stroke="#165dff" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"
               style="margin:0 auto 16px;display:block;">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
          </svg>
          <h1 style="font-size:22px;font-weight:600;margin:0 0 12px;color:#165dff;">
            小朋友，此页面默认给 Tauri 客户端使用
          </h1>
          <p style="font-size:14px;line-height:1.7;margin:0;color:#4b5563;">
            请勿使用浏览器直接打开呦？！<br/>
            请启动 MoLaunch 桌面客户端以正常使用本应用。
          </p>
        </div>
      </div>
    `
  }
  // 阻止后续 Vue app 挂载（避免 Tauri API 调用抛错刷屏）
  // eslint-disable-next-line no-console
  console.warn('[Startup][Frontend] Non-Tauri environment detected, Vue app mount skipped.')
} else {
  const app = createApp(App)
  const pinia = createPinia()
  app.use(pinia)
  app.use(router)

  // 在 mount 前触发 settingsStore.loadSettings()，把 primaryColor 注入 CSS 变量
  // 避免首屏使用默认蓝色然后闪烁到用户自定义色
  useSettingsStore(pinia)

  console.log('[Startup][Frontend] Vue app created, Pinia/router installed @', new Date().toISOString())

  app.mount('#app')
  console.log('[Startup][Frontend] app.mount("#app") called @', new Date().toISOString())
}
