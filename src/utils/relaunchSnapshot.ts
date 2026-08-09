/**
 * 程序重启会话快照：重启前加密保存"当前页面 + 在线房间会话"，
 * 新实例启动后解密恢复（页面跳回 + 房间自动重连）。
 *
 * 明文经后端 SDK（AES-256-CBC）加密后写入 localStorage，
 * 避免敏感信息（房间密码 / roomKey）明文落盘。
 */

import type { RoomState } from '@/stores/online/types'
import { invoke } from '@tauri-apps/api/core'

const SNAPSHOT_KEY = 'molaunch-relaunch-snapshot'
// 旧版明文快照键（升级前遗留，读取时清理，防明文密码残留）
const LEGACY_KEYS = ['molaunch-room-snapshot', 'molaunch-relaunch-restore']
// 普通重启"记住上次页面"：明文路径（非敏感信息），每次导航后更新，启动时恢复
const LAST_PAGE_KEY = 'molaunch-last-page'

/** 快照载荷：页面路径 + 在线房间会话（可选）+ guest 房间密码 */
export interface RelaunchSnapshotPayload {
  /** 快照保存时所在页面（pathname + search） */
  path: string
  /** 在线房间会话（在房间时为 roomState；不在房间为 null） */
  room: RoomState | null
  /** guest 房间密码（重新加入用），host 为空串 */
  password: string
  /** 快照保存时间戳 */
  savedAt: number
}

let pendingJoinPassword = ''
let reconnectPassword: string | null = null

/** guest 加入房间成功时记录房间密码（供重启后自动重连） */
export function rememberJoinPassword(password: string): void {
  pendingJoinPassword = password
}

/** 恢复快照时转存 guest 重连密码（供 RoomGuestPanel 挂载后自动重连） */
export function setReconnectPassword(password: string): void {
  reconnectPassword = password
}

/** 重启后 guest 自动重连用密码；一次性消费，读取后清空 */
export function consumeReconnectPassword(): string | null {
  const pw = reconnectPassword
  reconnectPassword = null
  return pw
}

/** 加密保存重启快照（SDK AES 加密后写入 localStorage），失败仅告警不中断 */
export async function saveRelaunchSnapshot(payload: {
  path: string
  room: RoomState | null
}): Promise<void> {
  const full: RelaunchSnapshotPayload = {
    path: payload.path,
    room: payload.room,
    password: pendingJoinPassword,
    savedAt: Date.now(),
  }
  try {
    const encrypted = await invoke<string>('relaunch_snapshot', {
      req: { action: 'encrypt', params: { data: JSON.stringify(full) } },
    })
    localStorage.setItem(SNAPSHOT_KEY, encrypted)
  } catch (e) {
    console.warn('[Relaunch] 保存重启快照失败:', e)
  }
}

/** 读取并清除重启快照（一次性消费；解密失败返回 null），并顺手清理旧版明文快照 */
export async function consumeRelaunchSnapshot(): Promise<RelaunchSnapshotPayload | null> {
  LEGACY_KEYS.forEach((key) => localStorage.removeItem(key))
  const raw = localStorage.getItem(SNAPSHOT_KEY)
  if (!raw) {
    return null
  }
  localStorage.removeItem(SNAPSHOT_KEY)
  try {
    const plain = await invoke<string>('relaunch_snapshot', {
      req: { action: 'decrypt', params: { data: raw } },
    })
    return JSON.parse(plain) as RelaunchSnapshotPayload
  } catch (e) {
    console.warn('[Relaunch] 读取重启快照失败:', e)
    return null
  }
}

/** 清除重启快照（UAC 被拒绝时调用，避免下次启动误恢复） */
export function clearRelaunchSnapshot(): void {
  localStorage.removeItem(SNAPSHOT_KEY)
  reconnectPassword = null
}

/** 记录当前页面路径（供普通重启后回到上次打开的页面；忽略入口/登录页） */
export function saveLastPage(path: string): void {
  if (!path || path === '/' || path === '/login') return
  try {
    localStorage.setItem(LAST_PAGE_KEY, path)
  } catch (e) {
    console.warn('[Relaunch] 保存上次页面失败:', e)
  }
}

/** 读取上次打开的页面路径（启动恢复用；不消费，后续导航持续覆盖） */
export function readLastPage(): string | null {
  try {
    return localStorage.getItem(LAST_PAGE_KEY)
  } catch (e) {
    console.warn('[Relaunch] 读取上次页面失败:', e)
    return null
  }
}
