/**
 * GitHub 镜像源筛选工具
 *
 * 读取 src/assets/Common/githubProxy.json，对每个源构造 easytier release 下载测速 URL，
 * 并发探测可用（HEAD 优先，部分镜像不支持 HEAD 时回退 GET 短 Range），
 * 按响应耗时排序取最快的前 10 个作为默认源。
 * 启动时执行一次：已有用户自定义镜像源（配置持久化）则保留，否则注入默认源。
 * 失败静默（后端下载时回退官方源）。
 *
 * 注：项目 assetsInclude 将 *.json 视为静态资源（不编译进 chunk），故经 ?url 导入后 fetch 加载。
 */
import proxyJsonUrl from '@/assets/Common/githubProxy.json?url'
import type { GithubProxy } from '@/types/online'
import { getConfigMap } from '@/utils/api/config'
import { setGithubProxies } from '@/utils/api/online-manager/easytier'

/** 测速用固定版本（已知存在的 release，仅验证可用性与速度） */
const PROBE_VERSION = '2.6.4'
const EASYTIER_REPO = 'EasyTier/EasyTier'
/** 筛选数量上限（按响应耗时取最快 N 个） */
const PROXY_LIMIT = 10
/** 单源测速超时（ms）：部分镜像响应较慢，5s 过严会误杀可用源 */
const PROBE_TIMEOUT = 8000

/** 启动时筛选出的默认源缓存（设置页"恢复默认"复用） */
let defaultProxies: GithubProxy[] = []

/** 当前平台资产名（与后端 asset_name 一致） */
function probeAssetName(): string {
  const ua = navigator.userAgent
  const os = ua.includes('Windows') ? 'windows' : ua.includes('Mac') ? 'macos' : 'linux'
  const arch = ua.includes('x86_64') || ua.includes('Win64') ? 'x86_64' : 'aarch64'
  return `easytier-${os}-${arch}-v${PROBE_VERSION}.zip`
}

/** 构造镜像源测速 URL（type: path 追加路径 / type: full 追加完整 GitHub URL） */
function buildProbeUrl(proxy: GithubProxy): string {
  const base = proxy.base.replace(/\/+$/, '')
  const asset = probeAssetName()
  return proxy.type === 'path'
    ? `${base}/${EASYTIER_REPO}/releases/download/v${PROBE_VERSION}/${asset}`
    : `${base}/https://github.com/${EASYTIER_REPO}/releases/download/v${PROBE_VERSION}/${asset}`
}

/**
 * 单源测速：HEAD + Range 0-1 优先；部分镜像不支持 HEAD（405/501/403）时
 * 回退 GET + Range 0-1023（读取响应头即完成并释放连接）。
 * 返回耗时（ms），失败返回 null。
 */
async function probeSource(proxy: GithubProxy): Promise<number | null> {
  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), PROBE_TIMEOUT)
  const start = performance.now()
  const url = buildProbeUrl(proxy)
  try {
    let resp = await fetch(url, {
      method: 'HEAD',
      headers: { Range: 'bytes=0-1' },
      signal: controller.signal,
    })
    if (resp.status === 405 || resp.status === 501 || resp.status === 403) {
      resp = await fetch(url, {
        headers: { Range: 'bytes=0-1023' },
        signal: controller.signal,
      })
      if (resp.ok || resp.status === 206) await resp.body?.cancel()
    }
    if (resp.ok || resp.status === 206) return performance.now() - start
    return null
  } catch {
    return null
  } finally {
    clearTimeout(timer)
  }
}

/** 读取镜像源清单并筛选可用且快的前 PROXY_LIMIT 个 */
async function probeDefaults(): Promise<GithubProxy[]> {
  const proxyData = (await (await fetch(proxyJsonUrl)).json()) as { sources: GithubProxy[] }
  const sources = (proxyData.sources ?? []) as GithubProxy[]
  const results = await Promise.all(sources.map((s) => probeSource(s).then((ms) => ({ s, ms }))))
  return results
    .filter((r): r is { s: GithubProxy; ms: number } => r.ms !== null)
    .sort((a, b) => a.ms - b.ms)
    .slice(0, PROXY_LIMIT)
    .map((r) => r.s)
}

/** 启动时初始化：已有自定义源（配置持久化）则保留，否则注入默认源（失败静默） */
export async function initGithubProxies(): Promise<void> {
  try {
    const cfg = await getConfigMap()
    const custom = (cfg.onlineGithubProxies ?? []) as GithubProxy[]
    if (custom.length > 0) return // AppState 启动时已从配置加载，直接保留
    defaultProxies = await probeDefaults()
    if (defaultProxies.length > 0) await setGithubProxies(defaultProxies)
  } catch {
    // 镜像筛选失败静默，不影响启动（后端下载时回退官方源）
  }
}

/** 恢复默认镜像源（设置页按钮）：重新测速筛选并注入，返回新列表 */
export async function restoreDefaultProxies(): Promise<GithubProxy[]> {
  defaultProxies = await probeDefaults()
  if (defaultProxies.length > 0) await setGithubProxies(defaultProxies)
  return defaultProxies
}
