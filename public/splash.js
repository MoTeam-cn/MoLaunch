/**
 * MoLaunch 开屏动画逻辑（供 public/splash.html 使用）
 *
 * 说明：脚本抽成外部文件以满足全局 CSP（script-src 'self'，不允许内联脚本）。
 * 与 docs/Run-html/run.html 设计稿保持一致。
 */

document.addEventListener('DOMContentLoaded', () => {
  // 禁止右键菜单、快捷键与拖拽：splash 页仅作展示，不应响应任何交互。
  // 拦截 F1~F12 与所有 Ctrl/Cmd/Alt 组合键（刷新、DevTools、关窗、复制粘贴等），
  // 与主窗口 useDevToolsGuard 同一防护思路；splash 无输入框，无需保留编辑键。
  const guard = (e) => {
    e.preventDefault()
    e.stopPropagation()
  }
  document.addEventListener('contextmenu', guard, true)
  document.addEventListener('dragstart', guard, true)
  document.addEventListener('keydown', (e) => {
    const key = e.key.toLowerCase()
    if (/^f([1-9]|1[0-2])$/.test(key) || e.ctrlKey || e.metaKey || e.altKey) guard(e)
  }, true)

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

  // 进度条：动画最多走到 92%（留一小块余量），真正就绪时才补满
  setTimeout(() => {
    const total = 2400, step = 30, MAX = 92
    let p = 0
    const tick = () => {
      p += step
      const prog = Math.min((p / total) * MAX, MAX)
      fill.style.width = prog + '%'
      const idx = Math.min(Math.floor((prog / MAX) * (STATUS.length - 1)), STATUS.length - 2)
      status.textContent = STATUS[idx]
      if (prog < MAX) setTimeout(tick, step)
    }
    tick()
  }, 1900)

  // 副标语
  setTimeout(() => sub.classList.add('show'), 3200)

  // 就绪补满：仅在真正切换窗口前调用，避免进度虚满后仍等待
  const finish = () => {
    fill.style.width = '100%'
    status.textContent = '就绪'
    status.classList.add('done')
  }

  // Tauri 兜底：动画播完仍未收到前端就绪信号时，补满进度并通知后端切换到主窗口
  setTimeout(async () => {
    finish()
    try {
      const invoke = window.__TAURI?.core?.invoke
      if (invoke) await invoke('frontend_ready')
    } catch (_) { /* 浏览器预览 */ }
  }, 4600)

  // 窗口拖拽：整个页面即拖拽区。捕获阶段拦截（先于 Tauri 注入脚本执行并阻止其重复触发），
  // 调用失败时把原因写入状态栏，便于定位（无边框 + transparent 窗口拖拽）
  document.addEventListener('mousedown', (e) => {
    if (e.button !== 0) return
    const tauri = window.__TAURI_INTERNALS__
    if (!tauri?.invoke) {
      if (status) status.textContent = '拖拽不可用: 无 Tauri 通道'
      return
    }
    e.preventDefault()
    e.stopImmediatePropagation()
    tauri.invoke('plugin:window|start_dragging').catch((err) => {
      if (status) status.textContent = `拖拽错误: ${err}`
    })
  }, true)
})
