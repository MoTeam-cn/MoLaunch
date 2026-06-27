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
  const deviceId = ref<string | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Java 状态
  const javaPath = ref('')
  const javaList = ref<{ executable: string; version: string; major_version: number }[]>([])
  const javaLoaded = ref(false)

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
      
      // 初始化成功后获取设备 ID
      await fetchDeviceId()
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
        await fetchDeviceId()
      }
    } catch (e) {
      console.error('Failed to check SDK status:', e)
    }
  }

  async function fetchDeviceId() {
    try {
      deviceId.value = await tauri.getDeviceId()
    } catch (e) {
      console.error('Failed to get device ID:', e)
      deviceId.value = null
    }
  }

  async function loadJava() {
    if (javaLoaded.value) return
    try {
      const detected = await tauri.detectJava()
      if (detected && detected.executable) {
        javaPath.value = detected.executable
      }
      javaList.value = await tauri.listJava()
      javaLoaded.value = true
    } catch (e) {
      console.error('Failed to load Java:', e)
      javaList.value = []
      javaLoaded.value = true
    }
  }

  async function refreshJava() {
    javaLoaded.value = false
    javaPath.value = ''
    javaList.value = []
    await loadJava()
  }

  return {
    status,
    initialized,
    version,
    deviceId,
    loading,
    error,
    isReady,
    javaPath,
    javaList,
    javaLoaded,
    fetchPlatformInfo,
    initialize,
    checkStatus,
    fetchDeviceId,
    loadJava,
    refreshJava,
  }
})
