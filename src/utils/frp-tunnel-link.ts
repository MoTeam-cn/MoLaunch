/** 从 frpc 日志提取隧道访问链接，未命中时按隧道字段组装默认链接。 */

/**
 * 从 frpc 日志中提取访问链接（首个匹配），失败返回 null。
 *
 * 命中规则（按优先级）：
 * 1. `您可通过（或等价）xxx 访问您的服务`（中文厂商，如 Lolia）
 * 2. `start proxy success, [name] tcp://host:port`
 * 3. 形如 `tcp://host:port` / `http://host:port` / `url = host:port` 的行
 */
export function extractTunnelLink(lines: string[]): string | null {
  const text = lines.join('\n')

  // 1. 中文厂商："您可通过 host:port 访问您的服务"
  const zhRegex = /(?:您可以通过|您可通过|可通过)[:：]?\s*([a-zA-Z0-9][\w.-]*:\d+(?:\/[\w./-]*)?)/i
  const zhMatch = text.match(zhRegex)
  if (zhMatch) return zhMatch[1]

  // 2. "start proxy success" 附近的 tcp://host:port
  const successRegex = /start proxy success[^\n]*?tcp?:\/\/[^\s]*/i
  const successMatch = text.match(successRegex)
  if (successMatch) {
    const url = successMatch[0].match(/https?:\/\/[^\s]*/i)
    if (url) return url[0]
  }

  // 3. 通用 url 提取：
  //    - tcp://host:port / http://host:port
  //    - 或裸 host:port（形如 `url = host:port`）
  const urlRegex = /(https?:\/\/|tcp:\/\/)[^\s,.;，。；]*/gi
  const urlMatch = text.match(urlRegex)
  if (urlMatch) {
    // 优先选带端口/路径最完整的
    return urlMatch[urlMatch.length - 1]
  }

  return null
}

/** 按隧道字段组装默认访问链接（serverAddr:remotePort） */
export function buildTunnelLink(
  serverAddr: string,
  remotePort: number,
  tunnelType?: string,
): string {
  const hostport = `${serverAddr}:${remotePort}`
  // http/https 类型可带协议前缀，tcp/udp 默认不带协议
  if (tunnelType === 'http' || tunnelType === 'https') {
    return `${tunnelType}://${hostport}`
  }
  return hostport
}

/**
 * 获取隧道访问链接：优先从日志截取，未命中则按字段组装。
 * 返回可用于复制的最终链接。
 */
export function resolveTunnelLink(
  lines: string[],
  serverAddr: string,
  remotePort: number,
  tunnelType?: string,
): string {
  const extracted = extractTunnelLink(lines)
  if (extracted) return extracted
  return buildTunnelLink(serverAddr, remotePort, tunnelType)
}