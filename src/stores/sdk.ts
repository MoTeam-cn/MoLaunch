/**
 * SDK 状态管理
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { SdkStatus } from '@/types/auth'
import * as tauri from '@/utils/tauri'
import { safeCall } from '@/utils/async'

export const useSdkStore = defineStore('sdk', () => {
  // 状态
  const status = ref<SdkStatus | null>(null)
  const initialized = ref(false)
  const version = ref<string | null>(null)
  const deviceId = ref<string | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 计算属性
  const isReady = computed(() => initialized.value)

  // 方法
  async function fetchPlatformInfo() {
    // 已初始化则跳过，避免 App.vue 与 Home.vue 重复发起 IPC
    if (initialized.value) return
    await safeCall(async () => {
      status.value = await tauri.getPlatformInfo()
      version.value = await tauri.getSdkVersion()
      initialized.value = true
    }, 'get platform info')
  }

  async function fetchDeviceId() {
    try {
      deviceId.value = await tauri.getDeviceId()
    } catch (e) {
      console.error('Failed to get device ID:', e)
      deviceId.value = null
    }
  }

  return {
    status,
    initialized,
    version,
    deviceId,
    loading,
    error,
    isReady,
    fetchPlatformInfo,
    fetchDeviceId,
  }
})
