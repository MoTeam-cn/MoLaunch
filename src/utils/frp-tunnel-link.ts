/** 从 frpc 日志提取隧道访问链接，未命中时按隧道字段组装默认链接。 */

/** ANSI 颜色码（日志落盘后仍可能带转义序列；ESC 用 fromCharCode 构造，避免源码出现控制字符） */
const ANSI_ESCAPE = String.fromCharCode(27)
const ANSI_RE = new RegExp(`${ANSI_ESCAPE}\\[[0-9;?]*[ -/]*[@-~]`, 'g')
/** 源码位置（Go/frpc 日志的 `<client/control.go:198>`，避免被当成 host:port） */
const SOURCE_LOCATION_RE = /<[^>]*>/g
/** 中文厂商的提示语（Lolia 等：`您可通过 http://host 访问您的服务`） */
const ZH_HINT_RE = /(?:您可以通过|您可通过|可通过)[:：]?\s*/i
/** 链接尾部的句读标点，需从结果中剥掉（域名内部的点不受影响） */
const TRAILING_PUNCT_RE = /[.,;:!?，。；：！？、)】」』"'》〉]+$/

/**
 * 构造「取第一个匹配」与「取全部匹配」两种正则
 *
 * 分开构造是为了避免 `g` 标志的 lastIndex 在多次调用间互相污染。
 */
function patterns(source: string): { first: RegExp; all: RegExp } {
  return { first: new RegExp(source, 'i'), all: new RegExp(source, 'gi') }
}

/** 带协议的链接：http(s):// / tcp:// / udp://（域名、端口、路径完整保留） */
const SCHEME_URL = patterns(String.raw`(?:https?|tcp|udp):\/\/[^\s,;，。；、]+`)
/** 裸 host:port（主机名必须含点，避免把日志时间戳 20:36 误判为地址） */
const HOST_PORT = patterns(
  String.raw`[a-z0-9](?:[a-z0-9-]*[a-z0-9])?(?:\.[a-z0-9](?:[a-z0-9-]*[a-z0-9])?)+:\d{1,5}(?:\/[\w./?=&%#-]*)?`,
)

/** 去掉链接末尾的句读标点 */
function trimTrailingPunct(raw: string): string {
  return raw.replace(TRAILING_PUNCT_RE, '')
}

/** 在文本片段里取第一个链接：优先带协议的，其次裸 host:port */
function pickLink(segment: string): string | null {
  const scheme = SCHEME_URL.first.exec(segment)
  if (scheme) return trimTrailingPunct(scheme[0])
  const hostPort = HOST_PORT.first.exec(segment)
  if (hostPort) return trimTrailingPunct(hostPort[0])
  return null
}

/**
 * 从 frpc 日志中提取访问链接（首个匹配），失败返回 null。
 *
 * 命中规则（按优先级）：
 * 1. 中文厂商提示语之后的链接，如 `您可通过 http://host 访问您的服务`（Lolia 等）
 * 2. `start proxy success` 所在行内的 `tcp://host:port` / `http://host`
 * 3. 全文最后一个带协议链接，其次最后一个裸 `host:port`
 *
 * 链接内部允许出现 `.`（域名必需），只在结果尾部剥离句读标点。
 */
export function extractTunnelLink(lines: string[]): string | null {
  const text = lines
    .join('\n')
    .replace(ANSI_RE, '')
    .replace(SOURCE_LOCATION_RE, ' ')

  // 1. 中文厂商提示语之后的部分（提示语本身不含链接）
  const hint = ZH_HINT_RE.exec(text)
  if (hint) {
    const link = pickLink(text.slice(hint.index + hint[0].length))
    if (link) return link
  }

  // 2. frpc 官方输出的 "start proxy success" 行
  const successLine = text.match(/start proxy success[^\n]*/i)
  if (successLine) {
    const link = pickLink(successLine[0])
    if (link) return link
  }

  // 3. 兜底：全文最后一个链接（frpc 启动日志的访问地址通常在末尾）
  const schemeMatches = text.match(SCHEME_URL.all)
  if (schemeMatches && schemeMatches.length > 0) {
    return trimTrailingPunct(schemeMatches[schemeMatches.length - 1])
  }
  const hostPortMatches = text.match(HOST_PORT.all)
  if (hostPortMatches && hostPortMatches.length > 0) {
    return trimTrailingPunct(hostPortMatches[hostPortMatches.length - 1])
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
