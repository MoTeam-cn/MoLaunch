/**
 * 日志云端分享
 *
 * 分享为纯前端实现，上传前脱敏（镜像后端 logger/sanitize.rs 正则模式，
 * 后端日志读取路径已复用 sanitize.rs，此处为分享路径兜底）。
 * 支持两个服务：
 * - mclo.gs：国际主流，POST https://api.mclo.gs/1/log（form: content=）
 * - logshare.cn：国内访问快，POST https://api.logshare.cn/v1/log（同构接口）
 */

export type LogShareProvider = 'mclogs' | 'logshare'

interface ProviderConfig {
  label: string
  endpoint: string
  /** 响应缺少 url 时的兜底分享链接 */
  fallbackUrl: (id: string) => string
}

const PROVIDERS: Record<LogShareProvider, ProviderConfig> = {
  mclogs: {
    label: 'mclo.gs',
    endpoint: 'https://api.mclo.gs/1/log',
    fallbackUrl: (id) => `https://mclo.gs/${id}`,
  },
  logshare: {
    label: 'logshare.cn',
    endpoint: 'https://api.logshare.cn/v1/log',
    fallbackUrl: (id) => `https://logshare.cn/log/${id}`,
  },
}

/** 分享服务选项（供崩溃弹窗 / 实例日志页分享浮层复用） */
export const LOG_SHARE_PROVIDERS: { value: LogShareProvider; label: string; desc: string }[] = [
  { value: 'mclogs', label: 'mclo.gs', desc: '国际主流日志分享，自带分析' },
  { value: 'logshare', label: 'logshare.cn', desc: '国内访问快，支持 AI 分析' },
]

/** 上传日志前脱敏：JWT / JSON 敏感字段 / Bearer 头 / URL query 参数 / 本机用户名路径 */
export function sanitizeShareLog(content: string): string {
  let s = content
  // JWT 格式 token（eyJ 开头，三段点分隔）
  s = s.replace(/eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}/g, '***')
  // JSON 敏感字段值
  s = s.replace(
    /"(access_token|accesstoken|refresh_token|refreshtoken|client_token|clienttoken|token|password|passwd|secret|api_key|apikey|client_secret|authorization|session)"\s*:\s*"[^"]{8,}"/gi,
    '"$1":"***"',
  )
  // Authorization: Bearer 头
  s = s.replace(/Authorization:\s*Bearer\s+[^\s,]+/gi, 'Authorization: Bearer ***')
  // URL query 敏感参数
  s = s.replace(/([?&](token|key|api_key|apikey|signature|sig)=)[^&\s"'<>]+/gi, '$1***')
  // 本机用户路径（Windows: C:\Users\xxx；Unix: /home/xxx）
  s = s.replace(/[A-Za-z]:[\\/]Users[\\/][^\\/]+/g, '{USERNAME}')
  s = s.replace(/\/(home|Users)\/[^/\\\s]+/g, '/{USERNAME}')
  return s
}

/**
 * 上传日志到云端分享服务，返回分享链接
 */
export async function uploadLogShare(
  content: string,
  provider: LogShareProvider,
): Promise<string> {
  const conf = PROVIDERS[provider]
  const body = new URLSearchParams({ content })
  const res = await fetch(conf.endpoint, {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body,
  })
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  const data: { id?: string; url?: string } | null = await res.json().catch(() => null)
  const id = data?.id
  const url = data?.url || (id ? conf.fallbackUrl(id) : '')
  if (!url) throw new Error('服务响应缺少分享链接')
  return url
}

/** mclo.gs Insights 分析结果（先 POST /1/log 上传，再 GET /1/log/{id}?insights 拉取） */
export interface MclogsAnalysis {
  success?: boolean
  content?: {
    insights?: {
      /** 日志类型识别信息（type 为 "Unknown Log" 时无具体分析） */
      type?: string
      analysis?: {
        problems?: Array<{ title?: string; description?: string; solution?: string; type?: string }>
        /** 版本识别等信息条目（label/value/message） */
        information?: Array<{ label?: string; value?: string; message?: string }>
      }
    }
  }
}

/**
 * 云端日志分析（mclo.gs Insights）：崩溃后自动调用，有 problems 才展示。
 * 新版接口流程：先上传获取 id，再按 id 拉取 insights。
 */
export async function analyseLogShare(content: string): Promise<MclogsAnalysis | null> {
  // 1) 先上传，拿到日志 id
  const upRes = await fetch('https://api.mclo.gs/1/log', {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({ content }),
  })
  if (!upRes.ok) throw new Error(`HTTP ${upRes.status}`)
  const upData: { id?: string } | null = await upRes.json().catch(() => null)
  if (!upData?.id) throw new Error('上传响应缺少日志 id')
  // 2) 再按 id 拉取 Insights 分析
  const res = await fetch(`https://api.mclo.gs/1/log/${upData.id}?insights=1`)
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  const data: MclogsAnalysis | null = await res.json().catch(() => null)
  return data
}
