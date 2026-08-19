/**
 * GitHub 镜像源筛选工具
 *
 * 读取 src/assets/Common/githubProxy.json，对每个源构造 easytier release 下载测速 URL，
 * 并发 HEAD + Range: bytes=0-1 请求，筛选可用（2xx/206）且响应快的 30 个，
 * 经 setGithubProxies 传给后端（下载时竞速选源）。启动时执行一次，失败静默。
 */
import proxyData from '@/assets/Common/githubProxy.json'
import type { GithubProxy } from '@/types/online'
import { setGithubProxies } from '@/utils/api/online-manager/easytier'

/** 测速用固定版本（已知存在的 release，仅验证可用性与速度） */
const PROBE_VERSION = '2.6.4'
const EASYTIER_REPO = 'EasyTier/EasyTier'
/** 筛选数量上限 */
const PROXY_LIMIT = 30
/** 单源测速超时（ms） */
const PROBE_TIMEOUT = 5000

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
    : `${base}https://github.com/${EASYTIER_REPO}/releases/download/v${PROBE_VERSION}/${asset}`
}

/** 单源测速：HEAD + Range 0-1，返回耗时（ms）；失败返回 null */
async function probeSource(proxy: GithubProxy): Promise<number | null> {
  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), PROBE_TIMEOUT)
  try {
    const start = performance.now()
    const resp = await fetch(buildProbeUrl(proxy), {
      method: 'HEAD',
      headers: { Range: 'bytes=0-1' },
      signal: controller.signal,
    })
    if (resp.ok || resp.status === 206) {
      return performance.now() - start
    }
    return null
  } catch {
    return null
  } finally {
    clearTimeout(timer)
  }
}

/** 读取镜像源清单并筛选可用且快的 30 个，传给后端（失败静默） */
export async function initGithubProxies(): Promise<void> {
  try {
    const sources = (proxyData.sources ?? []) as GithubProxy[]
    const results = await Promise.all(sources.map((s) => probeSource(s).then((ms) => ({ s, ms }))))
    const picked = results
      .filter((r): r is { s: GithubProxy; ms: number } => r.ms !== null)
      .sort((a, b) => a.ms - b.ms)
      .slice(0, PROXY_LIMIT)
      .map((r) => r.s)
    if (picked.length > 0) {
      await setGithubProxies(picked)
    }
  } catch {
    // 镜像筛选失败静默，不影响启动（后端下载时回退官方源）
  }
}