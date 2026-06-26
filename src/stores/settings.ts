/**
 * 设置状态管理
 */

import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export type LayoutMode = 'sidebar' | 'topnav'
export type Theme = 'light' | 'dark' | 'system'

export const useSettingsStore = defineStore('settings', () => {
  // 状态
  const layoutMode = ref<LayoutMode>('sidebar')
  const theme = ref<Theme>('system')
  const language = ref<'zh-CN' | 'en-US'>('zh-CN')
  
  // 从本地存储加载设置
  function loadSettings() {
    try {
      const saved = localStorage.getItem('molaunch-settings')
      if (saved) {
        const parsed = JSON.parse(saved)
        if (parsed.layoutMode) layoutMode.value = parsed.layoutMode
        if (parsed.theme) theme.value = parsed.theme
        if (parsed.language) language.value = parsed.language
      }
    } catch (e) {
      console.error('Failed to load settings:', e)
    }
  }
  
  // 保存设置到本地存储
  function saveSettings() {
    try {
      localStorage.setItem('molaunch-settings', JSON.stringify({
        layoutMode: layoutMode.value,
        theme: theme.value,
        language: language.value,
      }))
    } catch (e) {
      console.error('Failed to save settings:', e)
    }
  }
  
  // 设置方法
  function setLayoutMode(mode: LayoutMode) {
    layoutMode.value = mode
    saveSettings()
  }
  
  function setTheme(newTheme: Theme) {
    theme.value = newTheme
    saveSettings()
    applyTheme()
  }
  
  function applyTheme() {
    const root = document.documentElement
    if (theme.value === 'dark' || 
        (theme.value === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
      root.classList.add('dark')
    } else {
      root.classList.remove('dark')
    }
  }
  
  // 初始化
  loadSettings()
  applyTheme()
  
  // 监听系统主题变化
  if (typeof window !== 'undefined') {
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      if (theme.value === 'system') {
        applyTheme()
      }
    })
  }
  
  return {
    layoutMode,
    theme,
    language,
    setLayoutMode,
    setTheme,
    loadSettings,
    saveSettings,
  }
})
