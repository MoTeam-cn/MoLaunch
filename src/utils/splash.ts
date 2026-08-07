/**
 * 开屏动画（splashscreen）就绪通知
 *
 * 启动时序：
 * 1. Tauri 先创建开屏窗口（splash.html 动画），main 窗口 visible:false 后台加载
 * 2. Vue app 挂载后调用 notifyFrontendReady()
 * 3. 为保证动画至少展示一段时间（避免一闪而过），不足最短时长则延迟到最短时长再通知
 * 4. 后端 frontend_ready 命令关闭开屏窗口、显示主窗口
 *
 * 开屏页与动画兜底见 public/splash.html（4.6s 未收到前端信号时自兜底调用同一命令）。
 */

import { invoke } from '@tauri-apps/api/core'

/** 开屏动画完整展示时长（ms）：与 splash.js 动画总时长（约 4.6s）对齐，避免动画播一半被切走 */
const SPLASH_MIN_MS = 4600

/** 模块加载时刻（约等于 WebView 加载 JS bundle 的时刻），作为最短展示计时起点 */
const startedAt = Date.now()

/**
 * 通知后端开屏动画就绪：至少展示 SPLASH_MIN_MS 后关闭开屏窗口、显示主窗口
 *
 * 浏览器预览环境（非 Tauri）invoke 会失败，静默忽略。
 */
export function notifyFrontendReady(): void {
  const elapsed = Date.now() - startedAt
  const wait = Math.max(0, SPLASH_MIN_MS - elapsed)
  window.setTimeout(() => {
    invoke('frontend_ready').catch(() => {
      // 浏览器预览环境无该命令，忽略即可
    })
  }, wait)
}
