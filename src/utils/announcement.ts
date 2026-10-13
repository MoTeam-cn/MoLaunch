/**
 * 公告展示与已读记录
 *
 * 启动时拉取当前生效公告，只展示本机未读过的；用户关闭可关闭的公告后写入已读 id，
 * 之后不再弹出。dismissible=false 的公告不写已读，每次启动都提示，直到服务端下架。
 * 已读记录只存本机 localStorage，不上报任何标识。
 */
import { ref } from 'vue'
import { fetchAnnouncements } from '@/utils/api/announcement'
import type { PublicAnnouncement } from '@/types/announcement'

/** localStorage 键：已读公告 id 列表 */
const STORAGE_KEY = 'molaunch.readAnnouncements'

/** 弹窗可见状态（供 AnnouncementDialog.vue 绑定） */
export const announcementVisible = ref(false)

/** 本次要展示的公告 */
export const pendingAnnouncements = ref<PublicAnnouncement[]>([])

/** 读取已读 id 集合（localStorage 不可用或内容损坏时返回空集合） */
function readIds(): Set<string> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return new Set()
    const parsed: unknown = JSON.parse(raw)
    if (!Array.isArray(parsed)) return new Set()
    return new Set(parsed.filter((v): v is string => typeof v === 'string'))
  } catch {
    return new Set()
  }
}

/** 写入已读 id 集合（失败忽略：下次启动会再提示一次） */
function writeIds(ids: Set<string>): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify([...ids]))
  } catch {
    // 静默忽略
  }
}

/** 直接展示指定公告（dev-api 与样式预览用） */
export function showAnnouncements(list: PublicAnnouncement[]): void {
  if (list.length === 0) return
  pendingAnnouncements.value = list
  announcementVisible.value = true
}

/** 启动时拉取并展示未读公告；无未读则不弹 */
export async function maybeShowAnnouncements(): Promise<void> {
  const list = await fetchAnnouncements()
  if (list.length === 0) return
  const read = readIds()
  showAnnouncements(list.filter((a) => !read.has(a.id)))
}

/** 关闭弹窗：可关闭的公告写入已读，不可关闭的只在本次会话内隐藏 */
export function closeAnnouncements(): void {
  const read = readIds()
  let changed = false
  for (const a of pendingAnnouncements.value) {
    if (a.dismissible && !read.has(a.id)) {
      read.add(a.id)
      changed = true
    }
  }
  if (changed) writeIds(read)
  announcementVisible.value = false
  pendingAnnouncements.value = []
}

/** 清空已读记录（dev-api 测试用：下次启动将重新提示） */
export function resetAnnouncementRead(): void {
  try {
    localStorage.removeItem(STORAGE_KEY)
  } catch {
    // 静默忽略
  }
}
