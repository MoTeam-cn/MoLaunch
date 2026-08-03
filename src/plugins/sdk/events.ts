/**
 * SDK 事件与日志域
 *
 * 提供插件自定义事件广播与日志输出。
 */

/** 发送自定义事件（强制 plugin: 前缀，避免与内置事件冲突） */
export function emitEvent(event: string, payload?: unknown): void {
  if (!event.startsWith('plugin:')) {
    console.warn(`[PluginSdk] 事件名必须以 "plugin:" 开头: ${event}`)
    return
  }
  window.dispatchEvent(new CustomEvent(event, { detail: payload }))
}

/** 记录日志（写入启动器日志文件） */
export function logMessage(level: 'info' | 'warn' | 'error', message: string): void {
  const prefix = '[Plugin]'
  switch (level) {
    case 'info':
      console.log(prefix, message)
      break
    case 'warn':
      console.warn(prefix, message)
      break
    case 'error':
      console.error(prefix, message)
      break
  }
}