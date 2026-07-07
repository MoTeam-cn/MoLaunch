/**
 * 设置状态管理
 */

import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Theme, Language } from '@/types/settings'

export type LayoutMode = 'sidebar' | 'topnav'

export const useSettingsStore = defineStore('settings', () => {
  const layoutMode = ref<LayoutMode>('sidebar')
  const theme = ref<Theme>('system')
  const language = ref<Language>('zh-CN')

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

  function setLayoutMode(mode: LayoutMode) {
    layoutMode.value = mode
    saveSettings()
  }

  function setTheme(newTheme: Theme) {
    theme.value = newTheme
    saveSettings()
  }

  function setLanguage(newLanguage: string) {
    language.value = newLanguage as Language
    saveSettings()
  }

  loadSettings()

  return {
    layoutMode,
    theme,
    language,
    setLayoutMode,
    setTheme,
    setLanguage,
    loadSettings,
    saveSettings,
  }
})
