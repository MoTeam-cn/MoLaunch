/**
 * 联机 store 白名单切片（阶段三子任务 8 安全加强）
 *
 * 从 stores/online.ts 房间切片进一步抽取的白名单 state + actions。
 * 4 个方法对应 4 个后端 action：list / add / remove / setEnabled。
 * 仅房主可调用，加入方无权管理白名单。
 * 启用白名单且条目为空 = 拒绝所有人加入（仅房主可进入）。
 *
 * 依赖房间切片的 roomState 引用（检查 role/roomCode，同步 whitelistEnabled）。
 */

import type { Ref } from 'vue'
import { ref } from 'vue'
import type {
  WhitelistEntry,
  WhitelistResponse,
} from '@/types/online'
import {
  listWhitelist,
  addWhitelist,
  removeWhitelist,
  setWhitelistEnabled,
} from '@/utils/api/online-manager'
import { toastSuccess, toastError } from '@/utils/toast'
import type { RoomState } from './types'

/**
 * 创建联机 store 白名单切片
 *
 * @param roomState 房间状态引用（检查房主身份 + 房间码，同步 whitelistEnabled）
 */
export function useOnlineWhitelistSlice(roomState: Ref<RoomState>) {
  /** 白名单条目列表（房主管理用） */
  const whitelistEntries = ref<WhitelistEntry[]>([])
  /** 白名单是否正在加载/操作中 */
  const whitelistLoading = ref(false)

  /**
   * 拉取当前房间的白名单列表（房主）
   *
   * 同步 `roomState.whitelistEnabled` 与 `whitelistEntries`。
   * @returns 白名单响应；失败时返回 null
   */
  async function refreshWhitelist(): Promise<WhitelistResponse | null> {
    if (roomState.value.role !== 'host' || !roomState.value.roomCode) return null
    whitelistLoading.value = true
    try {
      const result = await listWhitelist(roomState.value.roomCode)
      if (result.code !== 1 || !result.data) return null
      const data = result.data
      roomState.value.whitelistEnabled = data.enabled
      whitelistEntries.value = data.entries ?? []
      return data
    } finally {
      whitelistLoading.value = false
    }
  }

  /**
   * 添加白名单条目（房主）
   *
   * @param deviceId 设备友好标识（`mcsdk-xxxx-xxxx-xxxx-xxxx`）
   * @returns 是否添加成功
   */
  async function addWhitelistEntry(deviceId: string): Promise<boolean> {
    if (roomState.value.role !== 'host' || !roomState.value.roomCode) return false
    const trimmed = deviceId.trim()
    if (!trimmed) {
      toastError('设备 ID 不能为空')
      return false
    }
    whitelistLoading.value = true
    try {
      const result = await addWhitelist(roomState.value.roomCode, trimmed)
      if (result.code !== 1) {
        toastError(result.msg || '添加白名单失败')
        return false
      }
      toastSuccess(`已添加白名单：${trimmed}`)
      await refreshWhitelist()
      return true
    } finally {
      whitelistLoading.value = false
    }
  }

  /**
   * 移除白名单条目（房主）
   *
   * @param deviceId 设备友好标识
   * @returns 是否移除成功
   */
  async function removeWhitelistEntry(deviceId: string): Promise<boolean> {
    if (roomState.value.role !== 'host' || !roomState.value.roomCode) return false
    whitelistLoading.value = true
    try {
      const result = await removeWhitelist(roomState.value.roomCode, deviceId)
      if (result.code !== 1) {
        toastError(result.msg || '移除白名单失败')
        return false
      }
      toastSuccess(`已移除白名单：${deviceId}`)
      await refreshWhitelist()
      return true
    } finally {
      whitelistLoading.value = false
    }
  }

  /**
   * 修改白名单启用状态（房主）
   *
   * 关闭白名单不影响已落库的条目，再次启用时无需重新添加。
   * 同步更新 `roomState.whitelistEnabled`。
   * @param enabled 是否启用白名单
   * @returns 是否修改成功
   */
  async function updateWhitelistEnabled(enabled: boolean): Promise<boolean> {
    if (roomState.value.role !== 'host' || !roomState.value.roomCode) return false
    whitelistLoading.value = true
    try {
      const result = await setWhitelistEnabled(roomState.value.roomCode, enabled)
      if (result.code !== 1) {
        toastError(result.msg || '修改白名单状态失败')
        return false
      }
      roomState.value.whitelistEnabled = enabled
      toastSuccess(enabled ? '已启用白名单' : '已关闭白名单')
      return true
    } finally {
      whitelistLoading.value = false
    }
  }

  /** 清空白名单条目（仅前端缓存，由主 store 的 resetRoomState 调用） */
  function resetWhitelistEntries(): void {
    whitelistEntries.value = []
  }

  return {
    // 白名单状态
    whitelistEntries,
    whitelistLoading,
    // 白名单方法
    refreshWhitelist,
    addWhitelistEntry,
    removeWhitelistEntry,
    updateWhitelistEnabled,
    resetWhitelistEntries,
  }
}
