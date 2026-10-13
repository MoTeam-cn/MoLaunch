/**
 * 公告类型（对应 api-server 的 PublicAnnouncement）
 */

/** 公告级别 */
export type AnnouncementLevel = 'info' | 'warning' | 'important'

/** 公开公告（启动器读取视图） */
export interface PublicAnnouncement {
  id: string
  title: string
  content: string
  level: AnnouncementLevel
  priority: number
  starts_at: number
  ends_at: number
  link_url: string
  dismissible: boolean
}
