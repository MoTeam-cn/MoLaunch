/**
 * frpc 日志关键词翻译
 *
 * 将常见 frpc 英文日志关键词翻译为中文，帮助用户理解输出含义。
 * 纯前端实现：按关键词匹配替换，不调用任何翻译 API，零延迟。
 *
 * 翻译策略：
 * - 优先匹配完整短语（如 "i/o timeout" → "I/O 超时"）
 * - 其次匹配关键词（如 "login to the server failed" → "登录服务器失败"）
 * - 保留原始日志结构（时间戳 / 级别 / 源文件:行号 不翻译）
 * - 翻译后追加中文释义，原英文保留在前，便于对照
 */

/** 单条翻译规则 */
interface TranslateRule {
  /** 英文关键词或短语（小写，用于匹配） */
  pattern: string
  /** 中文翻译 */
  zh: string
}

/**
 * 翻译规则表（按优先级从高到低排序，长短语优先）
 *
 * 参考 frpc 源码常见日志输出：
 * - sub/root.go: start frpc service / frpc service ... stopped
 * - client/service.go: try to connect to server / connect to server error / login to the server failed
 * - pkg/util/log: login to the server failed
 */
const RULES: TranslateRule[] = [
  // 服务生命周期
  { pattern: 'start frpc service for config file', zh: '为配置文件启动 frpc 服务' },
  { pattern: 'with aggregated configuration', zh: '（聚合配置模式）' },
  { pattern: 'frpc service for config file', zh: 'frpc 服务（配置文件：' },
  { pattern: 'stopped', zh: '已停止' },
  // 连接服务器
  { pattern: 'try to connect to server', zh: '正在尝试连接服务器...' },
  { pattern: 'connect to server error', zh: '连接服务器失败' },
  { pattern: 'login to the server failed', zh: '登录服务器失败' },
  { pattern: 'with loginfailexit enabled, no additional retries will be attempted', zh: '（已启用登录失败即退出，不再重试）' },
  { pattern: 'with loginfailexit enabled', zh: '（已启用登录失败即退出）' },
  { pattern: 'no additional retries will be attempted', zh: '不再重试' },
  // 网络错误
  { pattern: 'i/o timeout', zh: 'I/O 超时（网络不通或被防火墙拦截）' },
  { pattern: 'connection refused', zh: '连接被拒绝（目标端口未开放或服务未启动）' },
  { pattern: 'connection reset by peer', zh: '连接被对端重置' },
  { pattern: 'no route to host', zh: '无路由到目标主机（网络不可达）' },
  { pattern: 'network is unreachable', zh: '网络不可达' },
  { pattern: 'dial tcp', zh: '建立 TCP 连接' },
  { pattern: 'dial udp', zh: '建立 UDP 连接' },
  // 鉴权
  { pattern: 'authorization timeout', zh: '鉴权超时' },
  { pattern: 'authorization failed', zh: '鉴权失败（token 错误）' },
  { pattern: 'auth failed', zh: '鉴权失败' },
  // 端口相关
  { pattern: 'listen tcp', zh: '监听 TCP 端口' },
  { pattern: 'bind: address already in use', zh: '端口已被占用' },
  { pattern: 'address already in use', zh: '地址已被占用' },
  // 通用
  { pattern: 'proxy', zh: '代理' },
  { pattern: 'visitor', zh: '访问者' },
  { pattern: 'xlhttp', zh: 'XHTTP 协议' },
  { pattern: 'xtcp', zh: 'XTCP 协议' },
  { pattern: 'stcp', zh: 'STCP 协议' },
  { pattern: 'https', zh: 'HTTPS' },
  { pattern: 'http', zh: 'HTTP' },
  { pattern: 'tcp', zh: 'TCP' },
  { pattern: 'udp', zh: 'UDP' },
  { pattern: 'tls', zh: 'TLS' },
]

/**
 * 翻译单行日志
 *
 * 输出格式：`<原始行> ｜ <中文释义>`
 * - 原始行保留完整结构（时间戳/级别/源文件/原始消息）
 * - 中文释义追加在行尾，用 ` ｜ ` 分隔，便于对照
 * - 无匹配规则时返回原始行
 *
 * 性能：单行匹配 O(N×M)，N=规则数（~30），M=行长度，对单行日志可忽略不计。
 */
export function translateLogLine(line: string): string {
  if (!line) return line
  const lower = line.toLowerCase()
  const translations: string[] = []
  const matchedPatterns = new Set<string>()

  for (const rule of RULES) {
    // 跳过已被更长短语包含的关键词，避免重复翻译
    let alreadyCovered = false
    for (const matched of matchedPatterns) {
      if (matched.includes(rule.pattern) && matched !== rule.pattern) {
        alreadyCovered = true
        break
      }
    }
    if (alreadyCovered) continue

    if (lower.includes(rule.pattern)) {
      translations.push(rule.zh)
      matchedPatterns.add(rule.pattern)
    }
  }

  if (translations.length === 0) return line
  return `${line} ｜ ${translations.join('；')}`
}

/**
 * 批量翻译日志行
 *
 * 保留原始行号与级别信息，仅追加翻译到 text 字段。
 * 用于 FrpLogs.vue 显示。
 */
export function translateLogLines(lines: string[]): string[] {
  return lines.map(translateLogLine)
}
