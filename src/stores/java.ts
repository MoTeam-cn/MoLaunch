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

  // 从 storage 读取保存的 Java 路径
  async function loadSavedJavaPath(): Promise<string | null> {
    try {
      return await tauri.getConfigValue('Java', 'path')
    } catch (e) {
      return null
    }
  }

  // 保存 Java 路径到 storage
  async function saveJavaPath(path: string) {
    try {
      await tauri.setConfigValue('Java', 'path', path)
    } catch (e) {
      console.error('Failed to save Java path:', e)
    }
  }

  async function detectJava() {
    if (javaLoaded.value) return
    try {
      javaList.value = await tauri.listJava()
      
      // 读取保存的 Java 路径
      const savedPath = await loadSavedJavaPath()
      
      if (savedPath) {
        // 用户手动选择的路径
        const found = javaList.value.find(j => j.executable === savedPath)
        if (found) {
          javaPath.value = found.executable
        } else {
          // 保存的路径无效，清空（自动模式）
          console.warn('Saved Java path not found, using auto mode')
          javaPath.value = ''
        }
      } else {
        // 自动选择模式，javaPath 为空表示自动
        javaPath.value = ''
      }
      
      javaLoaded.value = true
    } catch (e) {
      console.error('Failed to load Java:', e)
      javaList.value = []
      javaLoaded.value = true
    }
  }

  // 获取实际使用的 Java 路径：自动模式由后端启动流水线统一处理（select_best_java_with_loader）
  // 前端不再独立计算，避免与后端规则不一致

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
    // 保存到 storage
    saveJavaPath(path)
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
