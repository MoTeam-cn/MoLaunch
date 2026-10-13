/**
 * 公告 API
 *
 * 公开接口（服务端已豁免 JWT），无需设备凭证。由 Rust 侧发起——WebView 的
 * origin 不在服务端 CORS 白名单内，前端直接 fetch 会被浏览器拦掉。
 * 失败一律返回空列表：公告拉不到不该影响启动。
 */
import { onlineManager } from '@/utils/api/online-manager'
import type { PublicAnnouncement } from '@/types/announcement'

/** 后端 announcement_list 响应 */
interface AnnouncementListResponse {
  success: boolean
  announcements: PublicAnnouncement[]
  error: string
}

/** 拉取当前生效公告（通道与版本由后端推导；失败返回空列表） */
export async function fetchAnnouncements(): Promise<PublicAnnouncement[]> {
  try {
    const resp = await onlineManager<AnnouncementListResponse>('announcement_list')
    return resp?.announcements ?? []
  } catch {
    return []
  }
}
