/**
 * 界面个性化偏好 store
 *
 * 统一管理外观模式 / 界面密度 / 减少动效 / 自定义背景 / 隐藏项；
 * 每次修改立即写 localStorage 并应用到 <html>，无需重启即生效。
 */

import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import {
  DEFAULT_UI_PREFS,
  type AppearanceMode,
  type BackgroundFill,
  type BackgroundPrefs,
  type UiDensity,
  type UiPrefs,
} from '@/types/ui-prefs'
import { applyUiPrefs, loadUiPrefs, saveUiPrefs, watchSystemPreferences } from '@/utils/ui-prefs'

export const useUiPrefsStore = defineStore('uiPrefs', () => {
  const prefs = ref<UiPrefs>(loadUiPrefs())

  /** 把当前偏好同步到 DOM 与 localStorage */
  function persist() {
    saveUiPrefs(prefs.value)
    applyUiPrefs(prefs.value)
  }

  const appearance = computed(() => prefs.value.appearance)
  const density = computed(() => prefs.value.density)
  const reduceMotion = computed(() => prefs.value.reduceMotion)
  const background = computed(() => prefs.value.background)
  const hiddenPages = computed(() => prefs.value.hiddenPages)
  const hasBackground = computed(() => !!prefs.value.background.image)

  function setAppearance(mode: AppearanceMode) {
    prefs.value.appearance = mode
    persist()
  }

  function setDensity(value: UiDensity) {
    prefs.value.density = value
    persist()
  }

  function setReduceMotion(enabled: boolean) {
    prefs.value.reduceMotion = enabled
    persist()
  }

  function setBackground(patch: Partial<BackgroundPrefs>) {
    prefs.value.background = { ...prefs.value.background, ...patch }
    persist()
  }

  function setBackgroundFill(fill: BackgroundFill) {
    setBackground({ fill })
  }

  function clearBackground() {
    prefs.value.background = { ...DEFAULT_UI_PREFS.background }
    persist()
  }

  /** 页面是否被隐藏 */
  function isPageHidden(name: string): boolean {
    return prefs.value.hiddenPages.includes(name)
  }

  function toggleHiddenPage(name: string) {
    const list = prefs.value.hiddenPages
    prefs.value.hiddenPages = list.includes(name) ? list.filter((n) => n !== name) : [...list, name]
    persist()
  }


  /** 恢复全部界面个性化设置 */
  function resetAll() {
    prefs.value = { ...DEFAULT_UI_PREFS, background: { ...DEFAULT_UI_PREFS.background } }
    persist()
  }

  // 首屏立即应用（避免浅色闪烁）
  applyUiPrefs(prefs.value)
  // 系统配色/动效变化时重新解析（跟随系统模式下即时切换）
  watchSystemPreferences(() => applyUiPrefs(prefs.value))

  return {
    prefs,
    appearance,
    density,
    reduceMotion,
    background,
    hiddenPages,
    hasBackground,
    setAppearance,
    setDensity,
    setReduceMotion,
    setBackground,
    setBackgroundFill,
    clearBackground,
    isPageHidden,
    toggleHiddenPage,

    resetAll,
  }
})
