/**
 * 日志云端分享
 *
 * 纯前端实现（与 Axolotl 一致），上传前脱敏（镜像后端 logger/sanitize.rs 正则模式）。
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
