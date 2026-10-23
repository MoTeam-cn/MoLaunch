/**
 * 公告展示、已读记录与轮询
 *
 * 启动时拉一次，之后每 10 分钟拉一次；拉到未读公告时，只有窗口在前台才弹，
 * 否则先暂存，等用户把焦点切回启动器再弹（避免公告弹在后台被错过）。
 * 已读记录只存本机 localStorage，不上报任何标识。
 */
import { ref } from 'vue'
import { fetchAnnouncements } from '@/utils/api/announcement'
import type { PublicAnnouncement } from '@/types/announcement'

/** localStorage 键：已读公告 id 列表 */
const STORAGE_KEY = 'molaunch.readAnnouncements'

/** 定时拉取间隔：10 分钟 */
const POLL_INTERVAL_MS = 10 * 60 * 1000

/** 窗口重新获得焦点时的最小刷新间隔：避免频繁切换窗口反复请求云端 */
const FOCUS_REFRESH_MS = 60 * 1000

/** 弹窗可见状态（供 AnnouncementDialog.vue 绑定） */
export const announcementVisible = ref(false)

/** 本次要展示的公告 */
export const pendingAnnouncements = ref<PublicAnnouncement[]>([])

/** 已拉到但窗口不在前台、暂缓弹出的公告 */
let deferredAnnouncements: PublicAnnouncement[] = []

/** 上次成功发起拉取的时间戳（焦点补拉的去抖基准） */
let lastFetchAt = 0

/** 轮询定时器与初始化标记（模块级单例） */
let pollTimer: ReturnType<typeof setInterval> | null = null
let started = false

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

/** 过滤出本机未读的公告 */
function unreadOnly(list: PublicAnnouncement[]): PublicAnnouncement[] {
  const read = readIds()
  return list.filter((a) => !read.has(a.id))
}

/** 窗口当前是否在前台（拿不到焦点信息时按前台处理，宁可弹出也不吞掉） */
function isWindowFocused(): boolean {
  try {
    return document.hasFocus() && document.visibilityState !== 'hidden'
  } catch {
    return true
  }
}

/** 暂存待弹公告（按 id 去重，避免多轮轮询重复堆积） */
function defer(list: PublicAnnouncement[]): void {
  const seen = new Set(deferredAnnouncements.map((a) => a.id))
  for (const item of list) {
    if (seen.has(item.id)) continue
    seen.add(item.id)
    deferredAnnouncements.push(item)
  }
}

/** 直接展示指定公告（dev-api 与样式预览用） */
export function showAnnouncements(list: PublicAnnouncement[]): void {
  if (list.length === 0) return
  pendingAnnouncements.value = list
  announcementVisible.value = true
}

/** 有未读就弹；窗口不在前台或已有弹窗时先暂存，等焦点回来再弹 */
function showOrDefer(list: PublicAnnouncement[]): void {
  const unread = unreadOnly(list)
  if (unread.length === 0) return
  if (announcementVisible.value || !isWindowFocused()) {
    defer(unread)
    return
  }
  showAnnouncements(unread)
}

/** 把暂存的公告在窗口回到前台时补弹 */
function flushDeferred(): void {
  if (deferredAnnouncements.length === 0) return
  if (announcementVisible.value || !isWindowFocused()) return
  const list = deferredAnnouncements
  deferredAnnouncements = []
  showAnnouncements(list)
}

/** 拉取一次并展示未读公告（窗口不在前台则暂存） */
export async function maybeShowAnnouncements(): Promise<void> {
  lastFetchAt = Date.now()
  const list = await fetchAnnouncements()
  if (list.length === 0) return
  showOrDefer(list)
}

/** 窗口重新获得焦点：补弹暂存公告；距上次拉取超过阈值时顺手再拉一次 */
function onWindowFocus(): void {
  flushDeferred()
  if (Date.now() - lastFetchAt > FOCUS_REFRESH_MS) {
    void maybeShowAnnouncements()
  }
}

/** 从后台切回前台时同样处理（部分平台 focus 事件不触发，靠 visibilitychange 兜底） */
function onVisibilityChange(): void {
  if (document.visibilityState === 'visible') onWindowFocus()
}

/**
 * 启动公告轮询（App.vue onMounted 调用一次）
 *
 * - 立即拉一次；之后每 10 分钟拉一次
 * - 窗口重新获得焦点时补弹暂存公告，并视情况补拉一次
 */
export function initAnnouncements(): void {
  if (started) return
  started = true
  void maybeShowAnnouncements()
  pollTimer = setInterval(() => {
    void maybeShowAnnouncements()
  }, POLL_INTERVAL_MS)
  window.addEventListener('focus', onWindowFocus)
  document.addEventListener('visibilitychange', onVisibilityChange)
}

/** 停止轮询（仅测试/热重载使用） */
export function stopAnnouncements(): void {
  if (pollTimer !== null) {
    clearInterval(pollTimer)
    pollTimer = null
  }
  window.removeEventListener('focus', onWindowFocus)
  document.removeEventListener('visibilitychange', onVisibilityChange)
  started = false
}

/** 关闭弹窗：可关闭的公告写入已读，不可关闭的只在本次会话内隐藏；随后补弹暂存的公告 */
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
  flushDeferred()
}

/** 清空已读记录（dev-api 测试用：下次轮询将重新提示） */
export function resetAnnouncementRead(): void {
  try {
    localStorage.removeItem(STORAGE_KEY)
  } catch {
    // 静默忽略
  }
}
