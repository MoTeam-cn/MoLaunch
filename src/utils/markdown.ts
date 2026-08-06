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
import { showConfirmAsync } from '@/utils/modal'
import { mdIconComponent } from '@/utils/md-icons'

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

// 行内扩展：`[::名称]` / `[:icon:名称]` / `[:名称]` / `[名称]` → 图标占位符。
// 兼容模型输出的各种占位符变体：双冒号 `[::game]`、`[:icon:game]`、单冒号 `[:game]`、
// 无冒号 `[game]`；**闭合括号可缺失**（模型经常漏写 `]`，如 `[::game 你好...` 也会命中），
// 名称提取成功后仅替换占位符本身、剩余文本继续正常渲染。
// 仅当名称命中 utils/md-icons.ts 的已知图标表时才替换；负向前瞻 `(?!\()` 排除
// markdown 链接（`[text](url)`），普通文本（`[注]`）不受影响。
// 渲染为 `<span class="md-icon md-icon-名称">` 占位，由消费方在 v-html 挂载后
// 调用 mountMdIcons() 替换为 heroicons Vue 组件（名称经 class 携带，避免被
// DOMPurify 剥离自定义属性）。代码块内不生效（代码块由 marked 优先解析）。
marked.use({
  extensions: [
    {
      name: 'inlineIcon',
      level: 'inline',
      start(src: string) {
        return src.search(/\[/)
      },
      tokenizer(src: string) {
        // 闭合括号 `]` 可选：模型常输出 `[::game`（漏写 ]），需兜底识别；
        // 前缀 `[:icon:` / `[::` / `[:` 可选，`[名称]` 无冒号形式也兼容。
        // 负向前瞻放在 `\]?` 之前且用 `\]?\(` 断言，防止回溯破坏链接 `[game](url)`
        const match = /^\[(?::icon:|::|:)?([a-zA-Z0-9_-]+)(?!\]?\()\]?/.exec(src)
        if (!match) return undefined
        const iconName = match[1]
        if (!mdIconComponent(iconName)) return undefined
        return {
          type: 'inlineIcon',
          raw: match[0],
          iconName,
        }
      },
      renderer(token) {
        const iconName = (token as unknown as { iconName: string }).iconName
        return `<span class="md-icon md-icon-${iconName}"></span>`
      },
    },
  ],
})

marked.setOptions({
  gfm: true,
  breaks: false,
  renderer,
})

// ===== 模型输出容错预处理 =====
// AI 模型偶尔输出不合法的 Markdown：表格整行挤在一起（GFM 表格要求每行独立）、
// 加粗星号内侧带多余空格（CommonMark 规定 `**` 后不能跟空白，否则不生效）。
// 在交给 marked 解析前做防御性修正；围栏代码块内容原样保留。

/** GFM 表格分隔单元格（`:---` / `---:` / `---`） */
function isTableSepCell(cell: string): boolean {
  return /^:?-{3,}:?$/.test(cell.trim())
}

/**
 * 修复单行折叠表格：模型把整张表格挤在一行时（`| 表头 | | --- | --- | | 数据 | ... |`），
 * 按分隔单元格拆分，重建为多行 GFM 表格。正常表格的各行不受影响。
 */
function fixSingleLineTable(line: string): string {
  if (!/^\s*\|/.test(line) || !line.includes('|')) return line
  let cells = line
    .split('|')
    .map((c) => c.trim())
  if (/^\s*\|/.test(line)) cells = cells.slice(1)
  if (/\|\s*$/.test(line)) cells = cells.slice(0, -1)

  const sepIdx = cells.findIndex(isTableSepCell)
  if (sepIdx < 0) return line
  const seps = cells.slice(sepIdx)
  const colCount = seps.findIndex((c) => !isTableSepCell(c))
  const cols = colCount < 0 ? seps.length : colCount
  if (cols <= 0) return line
  // 整行全是分隔单元格（如正常的 `| --- | --- |` 分隔行）不处理
  if (cells.every(isTableSepCell)) return line

  const header = cells.slice(0, sepIdx)
  while (header.length < cols) header.push('')
  const data = cells.slice(sepIdx + cols).filter((c) => c.length > 0)
  const rows: string[] = []
  for (let i = 0; i < data.length; i += cols) {
    const row = data.slice(i, i + cols)
    while (row.length < cols) row.push('')
    rows.push(`| ${row.join(' | ')} |`)
  }
  return [
    `| ${header.join(' | ')} |`,
    `| ${Array(cols).fill('---').join(' | ')} |`,
    ...rows,
  ].join('\n')
}

/** 修复加粗星号内侧的多余空格：`** xxx**` / `**xxx **` → `**xxx**` */
function fixBoldSpacing(text: string): string {
  return text
    .replace(/(^|\s)\*\*\s+/g, '$1**')
    .replace(/\s+\*\*(?!\S)/g, '**')
}

/** 预处理非代码块部分（围栏代码块 / ``` 与 ~~~ 原样保留） */
function preprocessMarkdown(src: string): string {
  const fenceRe = /^\s*(`{3,}|~{3,})[^\n]*\n[\s\S]*?^\s*\1\s*$/gm
  let last = 0
  const parts: string[] = []
  let m: RegExpExecArray | null
  while ((m = fenceRe.exec(src)) !== null) {
    if (m.index > last) parts.push(fixBoldSpacing(src.slice(last, m.index)))
    parts.push(m[0])
    last = m.index + m[0].length
  }
  if (last < src.length) parts.push(fixBoldSpacing(src.slice(last)))
  return parts
    .join('')
    .split('\n')
    .map((line) => fixSingleLineTable(line))
    .join('\n')
}

/** 将 Markdown 文本渲染为已消毒的 HTML 字符串（供 `v-html` 使用） */
export function renderMarkdown(source: string): string {
  const prepared = preprocessMarkdown(source ?? '')
  const html = marked.parse(prepared, { async: false }) as string
  return DOMPurify.sanitize(html)
}

/** 将 Markdown 文本转换为纯文本（复制"渲染后文本"用，去除 markdown 语法） */
export function markdownToPlainText(source: string): string {
  const el = document.createElement('div')
  el.innerHTML = renderMarkdown(source)
  return (el.textContent ?? '').replace(/\n{3,}/g, '\n\n').trim()
}

/**
 * Markdown 内容区的链接点击策略（事件委托，绑定在渲染容器的 `@click` 上）
 *
 * 拦截所有外部链接在 webview 内的导航（否则会跳出 SPA 页面、无法返回），
 * 先弹二次确认（防止 AI 输出中夹带的外部链接被误点），确认后
 * 通过 Tauri shell 插件调用系统默认浏览器打开。
 */
export async function handleMarkdownLinkClick(e: MouseEvent): Promise<void> {
  if (e.defaultPrevented) return
  const anchor = (e.target as HTMLElement).closest<HTMLAnchorElement>('a[href]')
  if (!anchor) return
  const href = anchor.getAttribute('href')
  if (!href || !/^https?:\/\//i.test(href)) return
  e.preventDefault()
  e.stopPropagation()
  const confirmed = await showConfirmAsync('打开外部链接', `是否在系统浏览器中打开以下链接？\n\n${href}`)
  if (!confirmed) return
  open(href)
    .then(() => toastInfo('已在系统浏览器中打开'))
    .catch((err) => {
      console.error('[markdown] 系统浏览器打开失败:', err)
      toastError('打开外部链接失败')
    })
}