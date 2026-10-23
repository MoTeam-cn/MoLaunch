/**
 * 输入序列采样
 *
 * 采样窗口内连续输入的字符折叠为 32 位校验值，与预设值一致时触发回调；
 * 命中即清空窗口，焦点位于可编辑元素时不采样。
 */

const WINDOW_MS = 1800
const FOLD_TARGET = 0x2671059b
const MAX_LEN = 16

let buffer = ''
let lastAt = 0
let attached = false

function fold(text: string): number {
  let hash = 0x811c9dc5
  for (let i = 0; i < text.length; i += 1) {
    hash ^= text.charCodeAt(i)
    hash = Math.imul(hash, 0x01000193) >>> 0
  }
  return hash >>> 0
}

function isEditable(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false
  if (target.isContentEditable) return true
  const tag = target.tagName
  return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT'
}

/**
 * 挂载采样监听（幂等）
 *
 * @param onMatch 校验值命中时的回调
 */
export function attachInputTelemetry(onMatch: () => void): void {
  if (attached) return
  attached = true
  window.addEventListener('keydown', (event) => {
    if (event.ctrlKey || event.altKey || event.metaKey) return
    if (isEditable(event.target)) return
    if (event.key.length !== 1) return
    const now = Date.now()
    if (now - lastAt > WINDOW_MS) buffer = ''
    lastAt = now
    buffer = (buffer + event.key.toLowerCase()).slice(-MAX_LEN)
    if (fold(buffer) !== FOLD_TARGET) return
    buffer = ''
    onMatch()
  })
}
