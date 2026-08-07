/**
 * MoLaunch 开屏动画逻辑（供 public/splash.html 使用）
 *
 * 说明：脚本抽成外部文件以满足全局 CSP（script-src 'self'，不允许内联脚本）。
 * 与 docs/Run-html/run.html 设计稿保持一致。
 */

document.addEventListener('DOMContentLoaded', () => {
  const icon = document.getElementById('icon')
  const content = document.getElementById('content')
  const text = document.getElementById('text')
  const cursor = document.getElementById('cursor')
  const fill = document.getElementById('fill')
  const status = document.getElementById('status')
  const sub = document.getElementById('sub')

  const NAME = 'MoLaunch'
  const STATUS = ['初始化...', '加载资源...', '检查更新...', '就绪']

  // 图标出现 → 旋转左移
  requestAnimationFrame(() => icon.classList.add('show'))
  setTimeout(() => {
    icon.classList.remove('show')
    icon.classList.add('move')
  }, 700)

  // 打字机
  setTimeout(() => {
    content.classList.add('show')
    let i = 0
    const type = () => {
      if (i < NAME.length) {
        text.textContent += NAME[i++]
        setTimeout(type, 90)
      } else cursor.classList.add('done')
    }
    type()
  }, 1700)

  // 进度条
  setTimeout(() => {
    const total = 2400, step = 30
    let p = 0
    const tick = () => {
      p += step
      const prog = Math.min((p / total) * 100, 100)
      fill.style.width = prog + '%'
      const idx = Math.min(Math.floor((prog / 100) * STATUS.length), STATUS.length - 1)
      status.textContent = STATUS[idx]
      if (p < total) setTimeout(tick, step)
      else {
        status.textContent = '就绪'
        status.classList.add('done')
      }
    }
    tick()
  }, 1900)

  // 副标语
  setTimeout(() => sub.classList.add('show'), 3200)

  // Tauri 兜底：动画播完仍未收到前端就绪信号时，通知后端切换到主窗口
  setTimeout(async () => {
    try {
      const { invoke } = window.__TAURI?.core
      if (invoke) await invoke('frontend_ready')
    } catch (_) { /* 浏览器预览 */ }
  }, 4600)
})
