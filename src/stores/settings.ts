/**
 * 设置状态管理
 *
 * layoutMode/language 纯前端 localStorage；theme 字段保留但未生效；
 * primaryColor 经 applyPrimaryColor() 注入 CSS 变量（影响全部 primary-* 类），localStorage + 后端 INI 双轨存储。
 */

import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Theme, Language } from '@/types/settings'
import { applyPrimaryColor } from '@/utils/color'
import { getConfigMap, applyConfig } from '@/utils/api/config'
import { toastSuccess } from '@/utils/toast'
import { safeCall, safeCallSync } from '@/utils/async'

export type LayoutMode = 'sidebar' | 'topnav'

/** 默认主色（Arco 蓝） */
export const DEFAULT_PRIMARY_COLOR = '#165dff'

export const useSettingsStore = defineStore('settings', () => {
  const layoutMode = ref<LayoutMode>('sidebar')
  const theme = ref<Theme>('system')
  const language = ref<Language>('zh-CN')
  const primaryColor = ref<string>(DEFAULT_PRIMARY_COLOR)
  /** 记住上次打开的页面（默认关闭，设置 → 个性化开启后普通重启恢复上次页面） */
  const rememberLastPage = ref(false)
  /** 后端配置是否已加载完成（首次 IPC 拉取后置 true） */
  const backendSynced = ref(false)

  function loadSettings() {
    safeCallSync(() => {
      const saved = localStorage.getItem('molaunch-settings')
      if (saved) {
        const parsed = JSON.parse(saved)
        if (parsed.layoutMode) layoutMode.value = parsed.layoutMode
        if (parsed.theme) theme.value = parsed.theme
        if (parsed.language) language.value = parsed.language
        if (typeof parsed.primaryColor === 'string' && parsed.primaryColor) {
          primaryColor.value = parsed.primaryColor
        }
        if (typeof parsed.rememberLastPage === 'boolean') {
          rememberLastPage.value = parsed.rememberLastPage
        }
      }
    }, 'load settings')
    // 加载后立即注入 CSS 变量（首屏前调用可避免闪烁）
    applyPrimaryColor(primaryColor.value)
  }

  /**
   * 从后端 INI 同步 primaryColor（启动后异步调用一次）
   * - 后端有值且与前端不同 → 用后端值覆盖前端（以后端为准）
   * - 后端无值 → 把前端默认值同步到后端
   */
  async function syncPrimaryColorFromBackend() {
    await safeCall(async () => {
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
    }, '[Settings] sync primaryColor from backend')
  }

  function saveSettings() {
    safeCallSync(() => {
      localStorage.setItem('molaunch-settings', JSON.stringify({
        layoutMode: layoutMode.value,
        theme: theme.value,
        language: language.value,
        primaryColor: primaryColor.value,
        rememberLastPage: rememberLastPage.value,
      }))
    }, 'save settings')
  }

  function setLayoutMode(mode: LayoutMode) {
    layoutMode.value = mode
    saveSettings()
  }

  function setRememberLastPage(enabled: boolean) {
    rememberLastPage.value = enabled
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
    await safeCall(() => applyConfig({ primaryColor: color }), '[Settings] save primaryColor to backend')
  }

  loadSettings()

  return {
    layoutMode,
    theme,
    language,
    primaryColor,
    rememberLastPage,
    backendSynced,
    setLayoutMode,
    setTheme,
    setLanguage,
    setPrimaryColor,
    setRememberLastPage,
    loadSettings,
    saveSettings,
    syncPrimaryColorFromBackend,
  }
})
