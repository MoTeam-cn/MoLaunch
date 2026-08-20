/**
 * HTML section shadow DOM 渲染器（从 CustomLayoutPanel.vue 提取）
 *
 * CSS 隔离（shadow root 内样式互不影响）；JS 经 new Function 在主窗口上下文执行（可直接调 window.molaunch SDK）；
 * 无 iframe 消除 sandbox 安全警告；molaunch API 桥接 toast/modal，懒加载且仅初始化一次。
 */
import { toastInfo, toastSuccess, toastError, toastWarning } from '@/utils/toast'
import { showInfo, showConfirm, showPrompt } from '@/utils/modal'
import { safeCallSync } from '@/utils/async'
import type { LayoutSection } from './types'

/**
 * 内置设计系统 CSS（注入到 html section 的 shadow root 中）
 *
 * 提供与启动器主界面一致的视觉风格，开发者可直接使用 .btn / .card / .stat 等类名。
 */
const DESIGN_SYSTEM_CSS = `
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;font-size:13px;color:#1f2937;background:transparent;padding:12px}
h1{font-size:18px;font-weight:600;margin-bottom:8px}
h2{font-size:16px;font-weight:600;margin-bottom:6px}
h3{font-size:14px;font-weight:600;margin-bottom:4px}
p{font-size:12px;color:#6b7280;margin-bottom:4px}
/* 按钮 */
.btn{display:inline-flex;align-items:center;gap:4px;padding:6px 12px;border:1px solid #d1d5db;border-radius:4px;background:#fff;color:#374151;font-size:12px;cursor:pointer;transition:background .15s}
.btn:hover{background:#f3f4f6}
.btn-primary{background:#6366f1;border-color:#6366f1;color:#fff}
.btn-primary:hover{background:#4f46e5}
.btn-sm{padding:4px 8px;font-size:11px}
/* 卡片 */
.card{border:1px solid #e5e7eb;border-radius:6px;padding:12px;background:#fff}
.card-title{font-size:12px;font-weight:600;color:#6b7280;margin-bottom:8px}
/* 统计卡片 */
.stat{display:flex;flex-direction:column;gap:2px}
.stat-label{font-size:11px;color:#9ca3af}
.stat-value{font-size:20px;font-weight:700;color:#111827}
.stat-suffix{font-size:12px;color:#6b7280}
/* 网格 */
.grid{display:grid;gap:12px}
.grid-2{grid-template-columns:repeat(2,1fr)}
.grid-3{grid-template-columns:repeat(3,1fr)}
/* 进度条 */
.progress-bar{height:8px;border-radius:4px;background:#e5e7eb;overflow:hidden}
.progress-fill{height:100%;border-radius:4px;background:#6366f1;transition:width .3s}
.progress-fill.green{background:#10b981}
.progress-fill.yellow{background:#f59e0b}
.progress-fill.red{background:#ef4444}
/* 徽章 */
.badge{display:inline-flex;align-items:center;padding:2px 6px;border-radius:3px;font-size:10px;font-weight:500}
.badge-primary{background:#eef2ff;color:#4338ca}
.badge-green{background:#ecfdf5;color:#047857}
.badge-red{background:#fef2f2;color:#b91c1c}
.badge-gray{background:#f3f4f6;color:#6b7280}
/* 文本工具类 */
.text-muted{color:#9ca3af}
.text-sm{font-size:11px}
.text-lg{font-size:15px}
.text-bold{font-weight:600}
.text-center{text-align:center}
.flex{display:flex}
.items-center{align-items:center}
.justify-between{justify-content:space-between}
.gap-2{gap:8px}
.gap-4{gap:16px}
.mt-2{margin-top:8px}
.mt-4{margin-top:16px}
.mb-2{margin-bottom:8px}
.mb-4{margin-bottom:16px}
`

/**
 * 确保 window.molaunch API 已定义
 *
 * shadow DOM 方案中无 iframe，用户脚本直接在主窗口上下文执行，
 * 因此 window.molaunch 直接调用前端组件，无需 postMessage 桥接。
 */
let molaunchApiReady = false
function setupMolaunchApi() {
  if (molaunchApiReady) return
  molaunchApiReady = true

  const molaunch = {
    toast(type: string, text: string) {
      if (type === 'success') toastSuccess(text)
      else if (type === 'error') toastError(text)
      else if (type === 'warning') toastWarning(text)
      else toastInfo(text)
    },
    alert(title: string, message: string) {
      showInfo(title, message)
    },
    confirm(title: string, message: string): Promise<boolean> {
      return new Promise((resolve) => {
        showConfirm(title, message, () => resolve(true), () => resolve(false))
      })
    },
    prompt(title: string, message: string, defaultValue = ''): Promise<string | null> {
      return new Promise((resolve) => {
        showPrompt(title, message, (value: string) => resolve(value), { defaultValue, onCancel: () => resolve(null) })
      })
    },
  }

  ;(window as unknown as Record<string, unknown>).molaunch = molaunch
}

/**
 * 危险脚本模式（静态扫描，命中则拒绝执行）
 *
 * html section 脚本经 new Function 在主窗口上下文执行，可访问全局 DOM 与 Tauri IPC，
 * 因此拦截以下危险调用：
 * - window.molaunch 敏感方法（spawnProcess / createWindow）
 * - Tauri IPC 直接调用（__TAURI_INTERNALS__ / __TAURI__ 全局对象）
 * - 主窗口 DOM 篡改（document.body / documentElement 内容替换、document.write）
 */
const DANGEROUS_SCRIPT_PATTERNS: RegExp[] = [
  /molaunch\s*[[.]\s*['"]?(spawnProcess|createWindow)['"]?/i,
  /__TAURI_INTERNALS__/,
  /__TAURI__/,
  /document\.(body|documentElement)\.(innerHTML|outerHTML)\s*=/,
  /document\.write\s*\(/,
]

/** 静态扫描脚本内容，命中危险模式返回 true */
function isDangerousScript(code: string): boolean {
  return DANGEROUS_SCRIPT_PATTERNS.some((re) => re.test(code))
}

/**
 * 用 shadow DOM 渲染 html section
 *
 * 通过内容指纹避免相同内容重复渲染，每次渲染重建 shadow root 内容并执行用户脚本。
 */
export function renderHtmlShadow(
  container: HTMLElement,
  section: Extract<LayoutSection, { type: 'html' }>,
): void {
  // 内容指纹：避免相同内容重复渲染
  const key = section.content + '\0' + (section.script || '') + '\0' + (section.style || '')
  if (container.dataset.renderedKey === key) return
  container.dataset.renderedKey = key

  // 获取或创建 shadow root
  let shadow = container.shadowRoot
  if (!shadow) {
    shadow = container.attachShadow({ mode: 'open' })
  }
  shadow.innerHTML = ''

  // 注入设计系统 CSS
  const styleEl = document.createElement('style')
  styleEl.textContent = DESIGN_SYSTEM_CSS
  shadow.appendChild(styleEl)

  // 注入用户自定义样式
  if (section.style) {
    const userStyle = document.createElement('style')
    userStyle.textContent = section.style
    shadow.appendChild(userStyle)
  }

  // 注入用户 HTML
  const wrapper = document.createElement('div')
  wrapper.innerHTML = section.content
  shadow.appendChild(wrapper)

  // 确保 window.molaunch API 可用
  setupMolaunchApi()

  // 执行用户脚本（先静态扫描危险调用，命中则拒绝执行）
  // 注：script 已通过 if 守卫，但 TS 无法在闭包内窄化，提取到局部变量
  const script = section.script
  if (script) {
    if (isDangerousScript(script)) {
      console.warn('[CustomLayout] html section 脚本包含危险调用，已拒绝执行')
    } else {
      safeCallSync(() => new Function(script)(), '[CustomLayout] run html section script')
    }
  }
}
