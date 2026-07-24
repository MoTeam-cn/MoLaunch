/**
 * Minecraft MOTD § 格式化代码解析
 *
 * Minecraft 多人联机服务器 MOTD 使用 § + 字符 表示颜色/格式：
 * 颜色代码：0-9, a-f, g（1.16+ 新增）
 * 格式代码：k(混淆) l(粗体) m(删除线) n(下划线) o(斜体) r(重置)
 *
 * 解析为 HTML span，保留颜色和格式，前端用 v-html 渲染。
 * 输出已做 HTML 转义，防止 XSS。
 */

/** MC 颜色代码 → CSS 颜色（对应 Java 版正颜色调色板） */
const COLOR_MAP: Record<string, string> = {
  '0': '#000000', // black
  '1': '#0000AA', // dark_blue
  '2': '#00AA00', // dark_green
  '3': '#00AAAA', // dark_aqua
  '4': '#AA0000', // dark_red
  '5': '#AA00AA', // dark_purple
  '6': '#FFAA00', // gold
  '7': '#AAAAAA', // gray
  '8': '#555555', // dark_gray
  '9': '#5555FF', // blue
  a: '#55FF55', // green
  b: '#55FFFF', // aqua
  c: '#FF5555', // red
  d: '#FF55FF', // light_purple
  e: '#FFFF55', // yellow
  g: '#DDD605', // minecoin_gold (1.16+)
}

/** 格式代码 → CSS 属性 */
const FORMAT_MAP: Record<string, string> = {
  k: 'oblique', // 混淆（MC 是随机字符抖动，CSS 无等价，用斜体近似）
  l: 'bold',
  m: 'line-through',
  n: 'underline',
  o: 'italic',
}

/** r 重置代码：清空当前所有样式 */

/**
 * 解析 MC MOTD § 格式化代码为 HTML span 字符串
 *
 * @param raw 原始 MOTD 字符串（含 § 代码）
 * @returns HTML 字符串，可直接 v-html 渲染
 *
 * @example
 * parseMcMotd('§a在线 §r§7服务器') // '<span style="color:#55FF55">在线 </span><span style="color:#AAAAAA">服务器</span>'
 * parseMcMotd('§l§c粗体红字') // '<span style="color:#FF5555;font-weight:bold">粗体红字</span>'
 */
export function parseMcMotd(raw: string): string {
  if (!raw) return ''

  const result: string[] = []
  let currentColor = ''
  const currentFormats = new Set<string>()
  let buffer = ''

  // 先 HTML 转义，避免 § 切分后片段中含 < > & 导致 XSS
  const escapeHtml = (s: string): string =>
    s
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')

  /** 将 buffer 中累积的文本用当前样式输出为 span */
  const flushBuffer = () => {
    if (!buffer) return
    const styles: string[] = []
    if (currentColor) styles.push(`color:${currentColor}`)
    for (const fmt of currentFormats) {
      if (fmt === 'bold') styles.push('font-weight:bold')
      else if (fmt === 'italic') styles.push('font-style:italic')
      else if (fmt === 'underline') styles.push('text-decoration:underline')
      else if (fmt === 'line-through') styles.push('text-decoration:line-through')
      else if (fmt === 'oblique') styles.push('font-style:italic') // 混淆用斜体近似
    }

    const escaped = escapeHtml(buffer)
    if (styles.length === 0) {
      result.push(escaped)
    } else {
      result.push(`<span style="${styles.join(';')}">${escaped}</span>`)
    }
    buffer = ''
  }

  const chars = Array.from(raw)
  let i = 0
  while (i < chars.length) {
    const c = chars[i]
    if (c === '§' && i + 1 < chars.length) {
      // 遇到格式化代码，先冲刷当前缓冲
      flushBuffer()
      const code = chars[i + 1].toLowerCase()
      if (code === 'r') {
        // 重置所有样式
        currentColor = ''
        currentFormats.clear()
      } else if (code in COLOR_MAP) {
        currentColor = COLOR_MAP[code]
        // 新颜色会重置格式（MC 行为：颜色代码隐式重置格式）
        currentFormats.clear()
      } else if (code in FORMAT_MAP) {
        currentFormats.add(FORMAT_MAP[code])
      }
      // 未知代码（如 x y z）忽略样式，但仍冲刷缓冲
      i += 2
    } else {
      buffer += c
      i += 1
    }
  }
  flushBuffer()

  return result.join('')
}
