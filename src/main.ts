import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import './assets/styles/main.css'

// 前端 JS 入口最早可执行点：此时 WebView2 已完成 HTML/JS bundle 加载
// 与后端 setup() hook 的时间差 = WebView2 加载 localhost:1420 + JS bundle 解析耗时
console.log('[Startup][Frontend] main.ts entered (JS bundle parsed) @', new Date().toISOString())

const app = createApp(App)

app.use(createPinia())
app.use(router)
console.log('[Startup][Frontend] Vue app created, Pinia/router installed @', new Date().toISOString())

app.mount('#app')
console.log('[Startup][Frontend] app.mount("#app") called @', new Date().toISOString())
