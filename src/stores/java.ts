/**
 * Java 运行时状态管理
 */

import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as tauri from '@/utils/tauri'

export const useJavaStore = defineStore('java', () => {
  const javaPath = ref('')
  const javaList = ref<{ executable: string; version: string; major_version: number }[]>([])
  const javaLoaded = ref(false)

  async function detectJava() {
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

  async function listJava() {
    try {
      javaList.value = await tauri.listJava()
    } catch (e) {
      console.error('Failed to list Java:', e)
      javaList.value = []
    }
  }

  function setJavaPath(path: string) {
    javaPath.value = path
  }

  async function refreshJava() {
    javaLoaded.value = false
    javaPath.value = ''
    javaList.value = []
    await detectJava()
  }

  return {
    javaPath,
    javaList,
    javaLoaded,
    detectJava,
    listJava,
    setJavaPath,
    refreshJava,
  }
})
