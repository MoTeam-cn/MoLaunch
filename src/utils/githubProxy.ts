/**
 * GitHub 镜像源筛选工具
 *
 * 读取 src/assets/Common/githubProxy.json 的全部源，交给后端 `probe_github_proxies`
 * 并发测速（reqwest HEAD+Range 0-1，禁止重定向，不受前端 CSP 限制），
 * 后端随机抽 30 个测速并返回最快的前 10 个作为默认源；后端下载时再竞速选择。
 * 启动时执行一次：已有用户自定义镜像源（配置持久化）则保留，否则注入默认源。
 * 失败静默（后端下载时回退官方源）。
 *
 * 注：项目 assetsInclude 将 *.json 视为静态资源（不编译进 chunk），故经 ?url 导入后 fetch 加载。
 */
import proxyJsonUrl from '@/assets/Common/githubProxy.json?url'
import type { GithubProxy } from '@/types/online'
import { getConfigMap } from '@/utils/api/config'
import { probeGithubProxies, setGithubProxies } from '@/utils/api/online-manager/easytier'
import { getJson } from '@/utils/request'

/** 启动时筛选出的默认源缓存（设置页"恢复默认"复用） */
let defaultProxies: GithubProxy[] = []

/** 读取镜像源清单，交给后端测速筛选（随机抽 30 个，返回最快 10 个） */
async function probeDefaults(): Promise<GithubProxy[]> {
  const proxyData = await getJson<{ sources: GithubProxy[] }>(proxyJsonUrl)
  const sources = (proxyData.sources ?? []) as GithubProxy[]
  if (sources.length === 0) return []
  return probeGithubProxies(sources)
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