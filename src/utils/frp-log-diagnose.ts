/**
 * frpc 启动日志诊断
 *
 * 基于关键词模式匹配分析最近一次运行日志，推断退出原因并输出中文诊断（供 FrpLogs.vue 展示），
 * 覆盖网络/鉴权/配置/服务端四类失败场景，纯前端实现。
 */

/** 诊断结果 */
export interface DiagnoseResult {
  /** 退出类别：normal / network / auth / config / server / unknown */
  category: 'normal' | 'network' | 'auth' | 'config' | 'server' | 'unknown'
  /** 标题（简短中文描述） */
  title: string
  /** 详细说明 */
  detail: string
  /** 建议操作 */
  suggestion: string
  /** 匹配到的关键日志行（用于佐证诊断） */
  evidence: string[]
}

/** 单条诊断规则 */
interface DiagnoseRule {
  category: DiagnoseResult['category']
  /** 关键词数组（全部包含才命中，全部小写） */
  keywords: string[]
  title: string
  detail: string
  suggestion: string
}

/**
 * 诊断规则表（按优先级从高到低排序，更具体的规则在前）
 */
const RULES: DiagnoseRule[] = [
  // 正常启动（运行中）——优先级最高：日志含"成功启动隧道"等标志且无错误
  {
    category: 'normal',
    keywords: ['成功启动隧道'],
    title: '正常运行',
    detail: 'frpc 已成功启动隧道并连接到 Frp 服务器，当前运行正常。',
    suggestion: '无需处理。可通过日志中提示的访问地址访问你的服务。',
  },
  {
    category: 'normal',
    keywords: ['start proxy success'],
    title: '正常运行',
    detail: 'frpc 已成功启动隧道并连接到 Frp 服务器，当前运行正常。',
    suggestion: '无需处理。可通过日志中提示的访问地址访问你的服务。',
  },
  // 网络层
  {
    category: 'network',
    keywords: ['i/o timeout', 'dial tcp'],
    title: '连接服务器超时',
    detail: 'frpc 在尝试连接 Frp 服务器时超时，通常是网络不通或被防火墙拦截。',
    suggestion: '检查服务器地址是否正确、网络是否畅通、防火墙是否放行 serverPort 端口。',
  },
  {
    category: 'network',
    keywords: ['connection refused'],
    title: '连接被拒绝',
    detail: '目标服务器端口未开放或 Frp 服务未启动。',
    suggestion: '确认服务器端 frps 已运行，且 serverPort 配置正确。',
  },
  {
    category: 'network',
    keywords: ['no route to host'],
    title: '网络不可达',
    detail: '到达目标服务器的网络路由不存在。',
    suggestion: '检查服务器 IP 是否正确、本地网络是否正常、VPN/代理是否影响路由。',
  },
  {
    category: 'network',
    keywords: ['network is unreachable'],
    title: '网络不可达',
    detail: '本地网络无法访问目标服务器。',
    suggestion: '检查本地网络连接，尝试切换网络后重试。',
  },
  {
    category: 'network',
    keywords: ['dns', 'no such host'],
    title: '域名解析失败',
    detail: '无法解析服务器域名。',
    suggestion: '检查服务器地址是否正确，或尝试直接使用 IP 地址。',
  },
  // 鉴权层
  {
    category: 'auth',
    keywords: ['authorization failed'],
    title: '鉴权失败',
    detail: 'frpc 提供的 token 与服务器不匹配。',
    suggestion: '检查隧道配置中的 token 是否与服务器端一致。',
  },
  {
    category: 'auth',
    keywords: ['authorization timeout'],
    title: '鉴权超时',
    detail: '鉴权流程超时，可能是网络延迟过高或服务器负载过重。',
    suggestion: '检查网络质量，或联系服务器管理员。',
  },
  {
    category: 'auth',
    keywords: ['login to the server failed'],
    title: '登录服务器失败',
    detail: 'frpc 登录 Frp 服务器失败，可能是网络问题或 token 错误。',
    suggestion: '查看日志中的具体错误（如 i/o timeout 或 authorization failed），检查网络连接与 token 配置。',
  },
  // 配置层
  {
    category: 'config',
    keywords: ['address already in use'],
    title: '端口已被占用',
    detail: 'frpc 尝试监听的本地端口已被其他程序占用。',
    suggestion: '更换本地端口，或关闭占用该端口的程序。',
  },
  {
    category: 'config',
    keywords: ['parse config error'],
    title: '配置文件解析失败',
    detail: 'frpc 配置文件格式错误。',
    suggestion: '删除隧道重建，或检查配置文件 TOML 语法。',
  },
  // 服务端
  {
    category: 'server',
    keywords: ['protocol error'],
    title: '协议错误',
    detail: 'frpc 与 frps 协议版本不匹配或通信异常。',
    suggestion: '确认 frpc 版本与 frps 服务端版本兼容（建议同版本）。',
  },
  {
    category: 'server',
    keywords: ['custom config handler', 'not support'],
    title: '服务端不支持该配置',
    detail: 'frps 服务端不支持当前隧道类型或配置。',
    suggestion: '联系服务器管理员，或更换支持该隧道类型的服务器。',
  },
]

/**
 * 诊断日志
 *
 * 输入：日志行数组（已包含时间戳与级别前缀）
 * 输出：诊断结果，无匹配规则时 category=unknown
 *
 * 规则：从尾部向前扫描（最近一次运行最关键），命中即返回。
 * 同时收集 evidence 用于佐证。
 */
export function diagnoseLogs(lines: string[]): DiagnoseResult {
  if (lines.length === 0) {
    return {
      category: 'unknown',
      title: '暂无日志',
      detail: '未捕获到任何日志输出，隧道可能尚未启动。',
      suggestion: '启动隧道后再查看诊断结果。',
      evidence: [],
    }
  }

  // 从尾部扫描最近 maxScan 行（覆盖一次完整启动）
  const maxScan = 200
  const start = Math.max(0, lines.length - maxScan)
  const recent = lines.slice(start)

  // 检查是否正常退出（含 "stopped" 且无 error 关键词）
  const lowerJoined = recent.join('\n').toLowerCase()
  const hasError = /error|failed|timeout|refused/.test(lowerJoined)
  if (!hasError && lowerJoined.includes('stopped')) {
    return {
      category: 'normal',
      title: '正常退出',
      detail: '隧道已正常停止，未检测到异常。',
      suggestion: '无需处理。',
      evidence: recent.slice(-3),
    }
  }

  // 按规则匹配
  for (const rule of RULES) {
    const evidence: string[] = []
    let allKeywordsFound = true
    for (const kw of rule.keywords) {
      const matched = recent.find(l => l.toLowerCase().includes(kw))
      if (matched) {
        if (!evidence.includes(matched)) evidence.push(matched)
      } else {
        allKeywordsFound = false
        break
      }
    }
    if (allKeywordsFound) {
      return {
        category: rule.category,
        title: rule.title,
        detail: rule.detail,
        suggestion: rule.suggestion,
        evidence: evidence.slice(-3),
      }
    }
  }

  // 兜底：未匹配规则但有 error/failed 关键词
  if (hasError) {
    const errorLines = recent
      .filter(l => /error|failed|timeout|refused/i.test(l))
      .slice(-3)
    return {
      category: 'unknown',
      title: '检测到错误，但未能识别具体原因',
      detail: '日志中包含错误关键词，但未匹配到已知诊断规则。',
      suggestion: '查看下方日志详情，或反馈给开发者补充诊断规则。',
      evidence: errorLines,
    }
  }

  return {
    category: 'unknown',
    title: '未检测到异常',
    detail: '日志中未发现明显错误关键词。',
    suggestion: '若隧道仍无法工作，请查看完整日志或反馈给开发者。',
    evidence: recent.slice(-3),
  }
}

/**
 * 诊断结果类别对应的徽章颜色 class
 */
export function diagnoseBadgeClass(category: DiagnoseResult['category']): string {
  switch (category) {
    case 'normal': return 'bg-green-50 text-green-700 border-green-200'
    case 'network': return 'bg-amber-50 text-amber-700 border-amber-200'
    case 'auth': return 'bg-red-50 text-red-700 border-red-200'
    case 'config': return 'bg-orange-50 text-orange-700 border-orange-200'
    case 'server': return 'bg-purple-50 text-purple-700 border-purple-200'
    default: return 'bg-gray-50 text-gray-700 border-gray-200'
  }
}

/**
 * 诊断结果类别对应的图标名（heroicons outline）
 */
export function diagnoseIcon(category: DiagnoseResult['category']): string {
  switch (category) {
    case 'normal': return 'CheckCircleIcon'
    case 'network': return 'WifiIcon'
    case 'auth': return 'LockClosedIcon'
    case 'config': return 'Cog6ToothIcon'
    case 'server': return 'ServerIcon'
    default: return 'QuestionMarkCircleIcon'
  }
}
