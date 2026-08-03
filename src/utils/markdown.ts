/**
 * Markdown 渲染工具
 *
 * 统一封装 `marked`（Markdown → HTML）与 `dompurify`（HTML 消毒）。
 * - 更新日志等来自云端的内容必须经 DOMPurify 消毒，防 XSS。
 * - 复用点统一走此模块，避免各组件重复引入两个库、各自配置。
 *
 * 链接策略：禁止在 webview 内跳转到外部网页（会脱离 SPA 页面且无法返回），
 * 所有 Markdown 连接一律通过 Tauri shell 插件调用系统默认浏览器打开。
 */
import { marked } from 'marked'
import DOMPurify from 'dompurify'
import { open } from '@tauri-apps/plugin-shell'
import { toastError, toastInfo } from '@/utils/toast'

// 自定义渲染器：为 Markdown 渲染出的所有链接统一注入
// `target="_blank" rel="noopener noreferrer"`，并对交互做事件委托兜底
const renderer = new marked.Renderer()
const defaultLink = renderer.link.bind(renderer)
renderer.link = (token) => {
  const html = String(defaultLink(token))
  if (/^https?:\/\//i.test(token.href)) {
    return html.replace(/^<a /, '<a target="_blank" rel="noopener noreferrer" ')
  }
  return html
}

marked.setOptions({
  gfm: true,
  breaks: false,
  renderer,
})

/** 将 Markdown 文本渲染为已消毒的 HTML 字符串（供 `v-html` 使用） */
export function renderMarkdown(source: string): string {
  const html = marked.parse(source ?? '', { async: false }) as string
  return DOMPurify.sanitize(html)
}

/**
 * Markdown 内容区的链接点击策略（事件委托，绑定在渲染容器的 `@click` 上）
 *
 * 拦截所有外部链接在 webview 内的导航（否则会跳出 SPA 页面、无法返回），
 * 通过 Tauri shell 插件调用系统默认浏览器打开。
 */
export function handleMarkdownLinkClick(e: MouseEvent): void {
  if (e.defaultPrevented) return
  const anchor = (e.target as HTMLElement).closest<HTMLAnchorElement>('a[href]')
  if (!anchor) return
  const href = anchor.getAttribute('href')
  if (!href || !/^https?:\/\//i.test(href)) return
  e.preventDefault()
  e.stopPropagation()
  open(href)
    .then(() => toastInfo('已在系统浏览器中打开'))
    .catch((err) => {
      console.error('[markdown] 系统浏览器打开失败:', err)
      toastError('打开外部链接失败')
    })
}