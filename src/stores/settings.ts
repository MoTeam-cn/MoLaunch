/**
 * 设置状态管理
 */

import { defineStore } from 'pinia'
import { ref } from 'vue'

export type LayoutMode = 'sidebar' | 'topnav'
export type Theme = 'light' | 'system'

export const useSettingsStore = defineStore('settings', () => {
  const layoutMode = ref<LayoutMode>('sidebar')
  const theme = ref<Theme>('system')
  const language = ref<'zh-CN' | 'en-US'>('zh-CN')

  function loadSettings() {
    try {
      const saved = localStorage.getItem('molaunch-settings')
      if (saved) {
        const parsed = JSON.parse(saved)
        if (parsed.layoutMode) layoutMode.value = parsed.layoutMode
        if (parsed.language) language.value = parsed.language
      }
    } catch (e) {
      console.error('Failed to load settings:', e)
    }
  }

  function saveSettings() {
    try {
      localStorage.setItem('molaunch-settings', JSON.stringify({
        layoutMode: layoutMode.value,
        language: language.value,
      }))
    } catch (e) {
      console.error('Failed to save settings:', e)
    }
  }

  function setLayoutMode(mode: LayoutMode) {
    layoutMode.value = mode
    saveSettings()
  }

  function setTheme(newTheme: Theme) {
    theme.value = newTheme
    saveSettings()
  }

  loadSettings()

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
