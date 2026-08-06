/**
 * Token 估算工具
 *
 * 与后端 `ai_core::client::estimate_tokens` 保持一致的本地估算：
 * - CJK 字符（统一表意文字 / 扩展 A / 扩展 B / CJK 标点 / 全角）≈ 1 token/字符
 * - 其余字符 ≈ 1 token/4 字符（向上取整）
 * 仅用于前端展示上下文窗口占用情况；真实用量以后端返回的 usage 为准。
 */
const CJK_RANGES: Array<[number, number]> = [
  [0x4e00, 0x9fff], // CJK 统一表意文字
  [0x3400, 0x4dbf], // 扩展 A
  [0x20000, 0x2a6df], // 扩展 B
  [0x3000, 0x303f], // CJK 标点
  [0xff00, 0xffef], // 全角
]

function isCjk(code: number): boolean {
  return CJK_RANGES.some(([start, end]) => code >= start && code <= end)
}

/** 估算一段文本的 token 数（与后端 estimate_tokens 对齐） */
export function estimateTokens(text: string | null | undefined): number {
  if (!text) return 0
  let cjk = 0
  let other = 0
  for (const ch of text) {
    const code = ch.codePointAt(0) ?? 0
    if (isCjk(code)) cjk += 1
    else other += 1
  }
  return cjk + Math.ceil(other / 4)
}

/** 格式化 token 数量为易读文本（如 12.4k；整数千位显示 184k 而非 184.0k） */
export function formatTokens(count: number): string {
  if (count >= 1000) {
    const k = count / 1000
    return `${Number.isInteger(k) ? k : k.toFixed(1)}k`
  }
  return String(count)
}
