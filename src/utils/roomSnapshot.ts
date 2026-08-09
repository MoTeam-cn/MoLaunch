/**
 * 管理员提权重启的房间会话快照
 *
 * 提权重启前把 roomState（房间码/虚拟 IP/ICE/密钥等）写入 localStorage，
 * 新实例启动后恢复 roomState 并自动重连，保证创建/加入的房间不因重启丢失。
 */

import type { RoomState } from '@/stores/online/types'

const SNAPSHOT_KEY = 'molaunch-room-snapshot'

interface RoomSnapshotPayload {
  state: RoomState
  /** guest 房间密码（重新加入用），host 为空串 */
  password: string
  savedAt: number
}

/** guest 加入房间时表单输入的密码（内存暂存，快照保存时写入） */
let pendingJoinPassword = ''
/** guest 待重连密码（App.vue 消费快照后转存，RoomGuestPanel 重连时读取） */
let reconnectPassword: string | null = null

/** 记住 guest 加入房间的密码（joinRoom 成功后由 RoomManager 调用） */
export function rememberJoinPassword(password: string): void {
  pendingJoinPassword = password
}

/** 保存房间快照（提权重启前调用；未在房间时不调用） */
export function saveRoomSnapshot(state: RoomState): void {
  try {
    const payload: RoomSnapshotPayload = {
      state,
      password: pendingJoinPassword,
      savedAt: Date.now(),
    }
    localStorage.setItem(SNAPSHOT_KEY, JSON.stringify(payload))
  } catch (e) {
    console.warn('[Online] 保存房间快照失败:', e)
  }
}

/** 读取并清除房间快照（一次性消费，App.vue 启动恢复用） */
export function consumeRoomSnapshot(): RoomSnapshotPayload | null {
  try {
    const raw = localStorage.getItem(SNAPSHOT_KEY)
    if (!raw) return null
    localStorage.removeItem(SNAPSHOT_KEY)
    return JSON.parse(raw) as RoomSnapshotPayload
  } catch (e) {
    console.warn('[Online] 读取房间快照失败:', e)
    localStorage.removeItem(SNAPSHOT_KEY)
    return null
  }
}

/** 清除房间快照（UAC 被用户拒绝时调用，避免下次启动误恢复） */
export function clearRoomSnapshot(): void {
  localStorage.removeItem(SNAPSHOT_KEY)
  reconnectPassword = null
}

/** 记录 guest 待重连密码（App.vue 消费快照后调用） */
export function setReconnectPassword(password: string): void {
  reconnectPassword = password
}

/** 读取 guest 重连密码（RoomGuestPanel 重连时一次性消费） */
export function consumeReconnectPassword(): string | null {
  const pw = reconnectPassword
  reconnectPassword = null
  return pw
}
