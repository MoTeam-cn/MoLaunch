/**
 * 界面个性化偏好类型
 *
 * 纯前端偏好（外观模式 / 背景 / 密度 / 动效 / 隐藏项），存 localStorage，不走后端配置。
 */

/** 外观模式：浅色 / 深色 / 跟随系统 */
export type AppearanceMode = 'light' | 'dark' | 'system'

/** 界面密度：标准 / 紧凑 */
export type UiDensity = 'normal' | 'compact'

/** 背景图填充方式 */
export type BackgroundFill = 'cover' | 'contain' | 'repeat'

/** 背景图设置 */
export interface BackgroundPrefs {
  /** 本地图片路径或 http(s) URL，空串表示不使用背景图 */
  image: string
  /** 模糊半径（px，0-32） */
  blur: number
  /** 遮罩不透明度（0-100，越大背景越淡、内容越清晰） */
  mask: number
  /** 内容区不透明度（0-100，越小卡片越透、背景越明显；100 为完全不透明） */
  panel: number
  /** 填充方式 */
  fill: BackgroundFill
}

/** 完整界面偏好 */
export interface UiPrefs {
  appearance: AppearanceMode
  density: UiDensity
  /** 关闭全部界面动画与过渡 */
  reduceMotion: boolean
  background: BackgroundPrefs
  /** 被隐藏的页面（路由 name 列表） */
  hiddenPages: string[]

}

/** 可隐藏的页面清单（按路由路径；首页与设置页不可隐藏） */
export const HIDEABLE_PAGES: Array<{ path: string; label: string; hint: string }> = [
  { path: '/apps/versions', label: '下载与版本', hint: '版本列表、版本选择与安装入口' },
  { path: '/apps/downloads', label: '下载管理', hint: '下载任务列表（双击导航栏下载图标进入）' },
  { path: '/apps/tools', label: '工具箱', hint: '崩溃分析、资源包、NBT、截图等工具' },
  { path: '/apps/online', label: '联机大厅', hint: '多人联机与房间列表' },
  { path: '/apps/experimental', label: '实验性功能', hint: '需在进阶设置开启实验性开关后才出现' },
]

/** 默认界面偏好 */
export const DEFAULT_UI_PREFS: UiPrefs = {
  appearance: 'system',
  density: 'normal',
  reduceMotion: false,
  background: { image: '', blur: 0, mask: 0, panel: 65, fill: 'cover' },
  hiddenPages: [],
}
