/**
 * 界面个性化偏好：读写与 DOM 应用
 *
 * 全部偏好写 localStorage（molaunch.ui），并通过 <html> 的 data-* 属性与 CSS 变量
 * 驱动样式；不涉及后端配置，避免为纯视觉偏好引入配置迁移。
 */

import { DEFAULT_UI_PREFS, type AppearanceMode, type UiPrefs } from '@/types/ui-prefs'

const STORAGE_KEY = 'molaunch.ui'

/** 读取偏好（缺字段用默认值补齐，损坏时回退默认） */
export function loadUiPrefs(): UiPrefs {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return { ...DEFAULT_UI_PREFS, background: { ...DEFAULT_UI_PREFS.background } }
    const parsed = JSON.parse(raw) as Partial<UiPrefs>
    return {
      appearance: parsed.appearance ?? DEFAULT_UI_PREFS.appearance,
      density: parsed.density ?? DEFAULT_UI_PREFS.density,
      reduceMotion: parsed.reduceMotion ?? DEFAULT_UI_PREFS.reduceMotion,
      background: { ...DEFAULT_UI_PREFS.background, ...(parsed.background ?? {}) },
      hiddenPages: Array.isArray(parsed.hiddenPages) ? parsed.hiddenPages : [],
    }
  } catch {
    return { ...DEFAULT_UI_PREFS, background: { ...DEFAULT_UI_PREFS.background } }
  }
}

/** 保存偏好 */
export function saveUiPrefs(prefs: UiPrefs): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs))
  } catch {
    // localStorage 不可用时静默忽略：偏好丢失不影响使用
  }
}

/** 系统是否处于深色模式 */
export function systemPrefersDark(): boolean {
  return typeof window !== 'undefined' && !!window.matchMedia?.('(prefers-color-scheme: dark)').matches
}

/** 把外观模式解析为实际生效的 light / dark */
export function resolveAppearance(mode: AppearanceMode): 'light' | 'dark' {
  if (mode === 'system') return systemPrefersDark() ? 'dark' : 'light'
  return mode
}

/** 系统是否开启了「减少动效」 */
export function systemPrefersReducedMotion(): boolean {
  return typeof window !== 'undefined' && !!window.matchMedia?.('(prefers-reduced-motion: reduce)').matches
}

/**
 * 地址能否直接写进 CSS `url()`
 *
 * 只有 http(s) / data: / blob: 可以；裸本地路径（如 D:\\Pics\\a.png）WebView 加载不了，
 * 需要先由后端读成 data URL。
 */
export function isDirectImageUrl(raw: string): boolean {
  return /^(https?:|data:|blob:)/i.test(raw.trim())
}

/**
 * 把偏好应用到 <html>：data-theme / data-density / data-reduce-motion 与背景 CSS 变量
 *
 * `backgroundImageUrl` 是已经可直接交给 CSS 的地址（http(s) / data: / blob:）；
 * 本地路径需先由后端转成 data URL，未解析完成时传空串，避免写入非法 url() 导致整条声明失效。
 */
export function applyUiPrefs(prefs: UiPrefs, backgroundImageUrl?: string): void {
  const root = document.documentElement
  const appearance = resolveAppearance(prefs.appearance)
  root.dataset.theme = appearance
  root.dataset.appearanceMode = prefs.appearance
  root.dataset.density = prefs.density
  root.dataset.reduceMotion = prefs.reduceMotion || systemPrefersReducedMotion() ? 'true' : 'false'

  const { background } = prefs
  const rawImage = (backgroundImageUrl ?? prefs.background.image).trim()
  // 只有 http(s) / data: / blob: 能直接写进 url()；裸本地路径由 store 异步解析
  const cssImage = isDirectImageUrl(rawImage) ? rawImage : ''
  const hasImage = !!cssImage
  root.dataset.hasBackground = hasImage ? 'true' : 'false'
  root.style.setProperty('--ui-bg-image', hasImage ? `url("${cssImage}")` : 'none')
  root.style.setProperty('--ui-bg-blur', `${background.blur}px`)
  root.style.setProperty('--ui-bg-mask', `${background.mask / 100}`)
  root.style.setProperty('--ui-bg-size', background.fill === 'repeat' ? 'auto' : background.fill)
  root.style.setProperty('--ui-bg-repeat', background.fill === 'repeat' ? 'repeat' : 'no-repeat')
  // 内容区玻璃层：卡片透明度（背景图模式下才生效）
  const panel = Number.isFinite(background.panel) ? background.panel : 65
  root.style.setProperty('--ui-panel-alpha', String(Math.min(100, Math.max(0, panel)) / 100))
  // 背景图轻微提饱和，抵消半透明白面板的灰感
  root.style.setProperty('--ui-bg-saturate', hasImage ? '1.12' : '1')
}

/** 监听系统配色/动效变化，变化时回调（返回取消函数） */
export function watchSystemPreferences(onChange: () => void): () => void {
  if (typeof window === 'undefined' || !window.matchMedia) return () => {}
  const colorQuery = window.matchMedia('(prefers-color-scheme: dark)')
  const motionQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
  const handler = () => onChange()
  colorQuery.addEventListener('change', handler)
  motionQuery.addEventListener('change', handler)
  return () => {
    colorQuery.removeEventListener('change', handler)
    motionQuery.removeEventListener('change', handler)
  }
}
