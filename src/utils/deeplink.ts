/**
 * 深度链接（molaunch:// 协议）前端封装
 *
 * 后端 deeplink 模块按 host 路由后 emit `deeplink://new` 事件；
 * onDeeplink 实时监听，getCurrent 用于应用启动被唤醒的场景。
 */
import { listen } from '@tauri-apps/api/event'
import { getCurrent } from '@tauri-apps/plugin-deep-link'

/** 后端 deeplink 模块 emit 的结构化请求（与 Rust DeeplinkRequest 字段一致） */
export interface DeeplinkRequest {
  /** 原始 URL 字符串 */
  raw: string
  /** 协议名（固定 molaunch） */
  scheme: string
  /** 路由键（molaunch://run 的 run） */
  host: string
  /** host 之后的路径段 */
  path: string
  /** 查询参数（已 URL 解码） */
  query: Record<string, string>
}

/** 后端 emit 的事件名（与 Rust dispatch 的 emit 一致） */
export const DEEPLINK_EVENT = 'deeplink://new'

/**
 * 注册一个 deeplink 后缀路由的前端回调
 *
 * 返回取消监听函数。注意：后端 handler 才是真正的逻辑入口，
 * 前端回调主要用于 UI 跳转/提示（如打开对应页面）。
 */
export async function onDeeplink(
  handler: (req: DeeplinkRequest) => void,
): Promise<() => void> {
  return listen<DeeplinkRequest>(DEEPLINK_EVENT, (e) => {
    handler(e.payload)
  })
}

/**
 * 获取应用启动时由 deeplink 唤醒的 URL 列表
 *
 * 在应用 onMounted 时调用：若应用是通过 molaunch:// 链接启动的，
 * 返回触发链接；否则返回 null。
 */
export async function getStartupDeeplink(): Promise<string[] | null> {
  try {
    return await getCurrent()
  } catch {
    return null
  }
}

/** 解析 molaunch:// URL 为结构化请求（前端侧镜像，供测试/预览用） */
export function parseMolaunchUrl(raw: string): DeeplinkRequest | null {
  try {
    const url = new URL(raw)
    if (url.protocol !== 'molaunch:') return null
    const query: Record<string, string> = {}
    url.searchParams.forEach((v, k) => {
      query[k] = v
    })
    return {
      raw,
      scheme: 'molaunch',
      host: url.host,
      path: url.pathname,
      query,
    }
  } catch {
    return null
  }
}
