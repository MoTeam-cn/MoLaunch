/**
 * 界面个性化偏好 store
 *
 * 统一管理外观模式 / 界面密度 / 减少动效 / 自定义背景 / 隐藏项；
 * 每次修改立即写 localStorage 并应用到 <html>，无需重启即生效。
 * 背景图若为本地路径，会经后端转成 data URL 后再写入 CSS 变量。
 */

import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'
import {
  DEFAULT_UI_PREFS,
  type AppearanceMode,
  type BackgroundFill,
  type BackgroundPrefs,
  type UiDensity,
  type UiPrefs,
} from '@/types/ui-prefs'
import { applyUiPrefs, loadUiPrefs, saveUiPrefs, watchSystemPreferences } from '@/utils/ui-prefs'
import { readImageDataUrl } from '@/utils/api/system'
import { toastError } from '@/utils/toast'

/** 直接可用的图片地址（远程 / 内联） */
const DIRECT_IMAGE_RE = /^(https?:|data:|blob:)/i
/** 输入停顿多久后才去解析本地路径，避免逐字符输入时刷 IPC */
const IMAGE_RESOLVE_DEBOUNCE_MS = 400

export const useUiPrefsStore = defineStore('uiPrefs', () => {
  const prefs = ref<UiPrefs>(loadUiPrefs())
  /** 已解析、可直接写进 CSS url() 的背景图地址 */
  const backgroundImageUrl = ref('')
  let imageToken = 0
  let debounceTimer: ReturnType<typeof setTimeout> | null = null

  /** 把当前偏好同步到 DOM 与 localStorage */
  function persist() {
    saveUiPrefs(prefs.value)
    applyUiPrefs(prefs.value, backgroundImageUrl.value)
  }

  /**
   * 解析背景图地址
   *
   * - 空：清空背景
   * - http(s) / data: / blob:：直接使用
   * - 其它（本地路径）：交后端读成 data URL（WebView 不能直接加载本地路径）
   */
  async function resolveBackgroundImage() {
    const raw = prefs.value.background.image.trim()
    const token = ++imageToken
    if (!raw) {
      backgroundImageUrl.value = ''
      applyUiPrefs(prefs.value, '')
      return
    }
    if (DIRECT_IMAGE_RE.test(raw)) {
      backgroundImageUrl.value = raw
      applyUiPrefs(prefs.value, raw)
      return
    }
    try {
      const dataUrl = await readImageDataUrl(raw)
      if (token !== imageToken) return
      backgroundImageUrl.value = dataUrl
      applyUiPrefs(prefs.value, dataUrl)
    } catch (err) {
      if (token !== imageToken) return
      backgroundImageUrl.value = ''
      applyUiPrefs(prefs.value, '')
      toastError(err instanceof Error ? err.message : String(err))
    }
  }

  /** 延迟解析（输入框连续输入时只在停顿后执行一次） */
  function scheduleResolveBackgroundImage() {
    if (debounceTimer) clearTimeout(debounceTimer)
    debounceTimer = setTimeout(() => {
      debounceTimer = null
      void resolveBackgroundImage()
    }, IMAGE_RESOLVE_DEBOUNCE_MS)
  }

  const appearance = computed(() => prefs.value.appearance)
  const density = computed(() => prefs.value.density)
  const reduceMotion = computed(() => prefs.value.reduceMotion)
  const background = computed(() => prefs.value.background)
  const hiddenPages = computed(() => prefs.value.hiddenPages)
  /** 用户填了背景图（可能仍在解析中，用于设置项展示） */
  const hasBackground = computed(() => !!prefs.value.background.image)
  /** 背景图已生效（用于决定是否渲染背景层） */
  const hasVisibleBackground = computed(() => !!backgroundImageUrl.value)

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
    backgroundImageUrl.value = ''
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
    backgroundImageUrl.value = ''
    persist()
  }

  // 首屏立即应用（避免浅色闪烁），背景图异步解析后再补上
  applyUiPrefs(prefs.value, '')
  void resolveBackgroundImage()
  // 系统配色/动效变化时重新解析（跟随系统模式下即时切换）
  watchSystemPreferences(() => applyUiPrefs(prefs.value, backgroundImageUrl.value))
  // 背景图地址变化时重新解析（本地路径需要读文件）
  watch(() => prefs.value.background.image, scheduleResolveBackgroundImage)

  return {
    prefs,
    appearance,
    density,
    reduceMotion,
    background,
    backgroundImageUrl,
    hiddenPages,
    hasBackground,
    hasVisibleBackground,
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
