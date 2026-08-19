/**
 * GitHub 镜像源筛选工具
 *
 * 读取 src/assets/Common/githubProxy.json，把全部源交后端并发测速
 * （`github_probe` IPC，Rust reqwest 无浏览器 CORS 限制，禁止重定向），
 * 按响应耗时升序取最快的前 10 个作为默认源。
 * 启动时执行一次：已有用户自定义镜像源（配置持久化）则保留，否则注入默认源。
 * 失败静默（后端下载时回退官方源）。
 *
 * 注：项目 assetsInclude 将 *.json 视为静态资源（不编译进 chunk），故经 ?url 导入后 fetch 加载。
 */
import proxyJsonUrl from '@/assets/Common/githubProxy.json?url'
import type { GithubProxy } from '@/types/online'
import { getConfigMap } from '@/utils/api/config'
import { probeGithubProxies, setGithubProxies } from '@/utils/api/online-manager/easytier'

/** 筛选数量上限（按响应耗时取最快 N 个） */
const PROXY_LIMIT = 10

/** 启动时筛选出的默认源缓存（设置页"重新测速"复用） */
let defaultProxies: GithubProxy[] = []

/** 读取内置镜像源清单，交后端测速并筛选可用且快的前 PROXY_LIMIT 个 */
async function probeDefaults(): Promise<GithubProxy[]> {
  const proxyData = (await (await fetch(proxyJsonUrl)).json()) as { sources: GithubProxy[] }
  const sources = (proxyData.sources ?? []) as GithubProxy[]
  const results = await probeGithubProxies(sources)
  return results.slice(0, PROXY_LIMIT).map((r) => ({
    name: r.name || undefined,
    type: r.proxyType === 'full' ? 'full' : 'path',
    base: r.base,
  }))
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

/** 重新测速（设置页按钮）：交后端测速筛选并注入，返回新列表 */
export async function restoreDefaultProxies(): Promise<GithubProxy[]> {
  defaultProxies = await probeDefaults()
  if (defaultProxies.length > 0) await setGithubProxies(defaultProxies)
  return defaultProxies
}
