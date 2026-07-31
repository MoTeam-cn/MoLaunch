/**
 * 联机 store NAT 检测切片
 *
 * 从 stores/online.ts 抽取的 NAT 类型检测 state + actions。
 * 独立于认证切片和房间切片，仅依赖 @/utils/online/nat-type 的 STUN 探测实现。
 */

import { ref } from 'vue'
import type { NatDetectionResult } from '@/types/online'
import { detectNatTypeWithStun } from '@/utils/online/nat-type'

/** 创建联机 store NAT 检测切片 */
export function useOnlineNatSlice() {
  /** NAT 检测结果（null 表示未检测） */
  const natResult = ref<NatDetectionResult | null>(null)
  /** NAT 检测中（避免重复触发） */
  const natDetecting = ref(false)

  /**
   * 执行 NAT 类型检测（写入 natResult，侧边栏切换不丢失）
   *
   * - 已有结果或正在检测时跳过（避免重复请求）
   * - 检测失败时不覆盖已有结果（保留上次成功值）
   * - 供 Online.vue 进入页面时自动调用 + OnlineDevicePanel.vue 手动刷新调用
   */
  async function detectNat(): Promise<void> {
    if (natResult.value || natDetecting.value) return
    if (typeof RTCPeerConnection === 'undefined') return
    natDetecting.value = true
    try {
      const result = await detectNatTypeWithStun()
      natResult.value = result
    } catch (e) {
      console.warn('[Online] NAT detection failed:', e)
    } finally {
      natDetecting.value = false
    }
  }

  /** 强制重新检测 NAT（手动刷新时调用，忽略已有结果） */
  async function forceDetectNat(): Promise<void> {
    if (natDetecting.value) return
    if (typeof RTCPeerConnection === 'undefined') return
    natDetecting.value = true
    try {
      const result = await detectNatTypeWithStun()
      natResult.value = result
    } catch (e) {
      console.warn('[Online] NAT detection failed:', e)
    } finally {
      natDetecting.value = false
    }
  }

  return {
    // NAT 检测
    natResult,
    natDetecting,
    detectNat,
    forceDetectNat,
  }
}
