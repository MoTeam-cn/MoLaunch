/**
 * SDK 状态管理
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { SdkStatus } from '@/types/auth'
import * as tauri from '@/utils/tauri'

export const useSdkStore = defineStore('sdk', () => {
  // 状态
  const status = ref<SdkStatus | null>(null)
  const initialized = ref(false)
  const version = ref<string | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 计算属性
  const isReady = computed(() => initialized.value && version.value !== null)

  // 方法
  async function fetchPlatformInfo() {
    try {
      status.value = await tauri.getPlatformInfo()
    } catch (e) {
      console.error('Failed to get platform info:', e)
    }
  }

  async function initialize(gameDir?: string) {
    loading.value = true
    error.value = null
    
    try {
      version.value = await tauri.initializeSdk(gameDir)
      initialized.value = true
    } catch (e) {
      error.value = String(e)
      initialized.value = false
      throw e
    } finally {
      loading.value = false
    }
  }

  async function checkStatus() {
    try {
      initialized.value = await tauri.isSdkInitialized()
      if (initialized.value) {
        version.value = await tauri.getSdkVersion()
      }
    } catch (e) {
      console.error('Failed to check SDK status:', e)
    }
  }

  return {
    status,
    initialized,
    version,
    loading,
    error,
    isReady,
    fetchPlatformInfo,
    initialize,
    checkStatus,
  }
})
