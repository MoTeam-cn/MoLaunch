/**
 * 设置状态管理
 *
 * - layoutMode / language：纯前端 localStorage
 * - theme：原本的浅色/深色/跟随系统字段，当前版本未实际生效，保留字段以便未来接入
 * - primaryColor：主题主色 HEX（默认 "#165dff" Arco 蓝），通过 applyPrimaryColor() 注入 CSS 变量
 *   全项目所有 `primary-*` Tailwind 类与 `var(--color-primary-*)` 都会跟随此值变化
 *   存储双轨制：前端 localStorage（首屏前同步读取避免闪烁）+ 后端 INI（跨设备同步）
 */

import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Theme, Language } from '@/types/settings'
import { applyPrimaryColor } from '@/utils/color'
import { getConfigMap, applyConfig } from '@/utils/api/config'
import { toastSuccess } from '@/utils/toast'

export type LayoutMode = 'sidebar' | 'topnav'

/** 默认主色（Arco 蓝） */
export const DEFAULT_PRIMARY_COLOR = '#165dff'

export const useSettingsStore = defineStore('settings', () => {
  const layoutMode = ref<LayoutMode>('sidebar')
  const theme = ref<Theme>('system')
  const language = ref<Language>('zh-CN')
  const primaryColor = ref<string>(DEFAULT_PRIMARY_COLOR)
  /** 后端配置是否已加载完成（首次 IPC 拉取后置 true） */
  const backendSynced = ref(false)

  function loadSettings() {
    try {
      const saved = localStorage.getItem('molaunch-settings')
      if (saved) {
        const parsed = JSON.parse(saved)
        if (parsed.layoutMode) layoutMode.value = parsed.layoutMode
        if (parsed.theme) theme.value = parsed.theme
        if (parsed.language) language.value = parsed.language
        if (typeof parsed.primaryColor === 'string' && parsed.primaryColor) {
          primaryColor.value = parsed.primaryColor
        }
      }
    } catch (e) {
      console.error('Failed to load settings:', e)
    }
    // 加载后立即注入 CSS 变量（首屏前调用可避免闪烁）
    applyPrimaryColor(primaryColor.value)
  }

  /**
   * 从后端 INI 同步 primaryColor（启动后异步调用一次）
   * - 后端有值且与前端不同 → 用后端值覆盖前端（以后端为准）
   * - 后端无值 → 把前端默认值同步到后端
   */
  async function syncPrimaryColorFromBackend() {
    try {
      const cfg = await getConfigMap()
      const backend = cfg.primaryColor
      if (backend && backend !== primaryColor.value) {
        primaryColor.value = backend
        applyPrimaryColor(backend)
        saveSettings()
      } else if (!backend) {
        // 后端无值，把前端值同步到后端
        await applyConfig({ primaryColor: primaryColor.value })
      }
      backendSynced.value = true
    } catch (e) {
      console.error('[Settings] Failed to sync primaryColor from backend:', e)
    }
  }

  function saveSettings() {
    try {
      localStorage.setItem('molaunch-settings', JSON.stringify({
        layoutMode: layoutMode.value,
        theme: theme.value,
        language: language.value,
        primaryColor: primaryColor.value,
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

  /**
   * 设置主题主色
   * - 立即注入 CSS 变量（即时生效）
   * - 同步写入前端 localStorage（避免首屏闪烁）
   * - 异步写后端 INI（跨设备同步，失败不阻塞 UI）
   * - 弹 toast 提示用户操作已生效
   */
  async function setPrimaryColor(color: string) {
    primaryColor.value = color
    applyPrimaryColor(color)
    saveSettings()
    toastSuccess(`主题色已更新为 ${color.toUpperCase()}`)
    // 异步写后端，失败仅日志不阻塞
    try {
      await applyConfig({ primaryColor: color })
    } catch (e) {
      console.error('[Settings] Failed to save primaryColor to backend:', e)
    }
  }

  loadSettings()

  return {
    layoutMode,
    theme,
    language,
    primaryColor,
    backendSynced,
    setLayoutMode,
    setTheme,
    setLanguage,
    setPrimaryColor,
    loadSettings,
    saveSettings,
    syncPrimaryColorFromBackend,
  }
})
