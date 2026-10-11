/**
 * 运行平台判断（前端侧）
 *
 * 仅依赖 WebView 暴露的 navigator 信息，不调用后端；用于对特定平台做能力提示
 * （如官方已停止提供 macOS 红石内核）。
 */

/** 当前是否运行在 macOS */
export function isMacOS(): boolean {
  const platform = navigator.platform ?? ''
  if (/^Mac/i.test(platform)) return true
  // 新版本 WebView 可能不再提供 platform，退回 userAgent
  return /Macintosh|Mac OS X/i.test(navigator.userAgent)
}
