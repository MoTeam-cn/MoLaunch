/**
 * Element Plus Icons - SVG path 数据
 *
 * Copyright (C) 2026 MoTeam
 *
 * SVG path data derived from Element Plus Icons
 * (https://github.com/element-plus/element-plus-icons).
 * Original icons licensed under the MIT License.
 *
 * MIT License full text will be added here
 *
 * 使用方式：
 *   import { elementIcons } from '@/utils/element-icons'
 *   const icon = elementIcons.info
 *   // <svg :viewBox="icon.viewBox" fill="currentColor"><path :d="icon.path" /></svg>
 */

export interface ElementIcon {
  /** SVG viewBox */
  viewBox: string
  /** path 的 d 属性 */
  path: string
}

/** Element Plus Icons 图标集（viewBox 统一为 0 0 1024 1024） */
export const elementIcons = {
  /** 信息提示（实心圆圈 i） */
  info: {
    viewBox: '0 0 1024 1024',
    path: 'M512 64a448 448 0 1 1 0 896 448 448 0 0 1 0-896m67.2 275c33.3 0 60.3-23 60.3-57.3s-27-57.3-60.3-57.3-60.2 23-60.2 57.3 27 57.4 60.2 57.4m11.7 360.1c0-6.8 2.4-24.6 1-34.8L539.3 725c-10.9 11.4-24.5 19.4-30.9 17.3a13 13 0 0 1-8.2-14.7l87.6-277c7.2-35.2-12.5-67.2-54.3-71.3-44 0-109 44.7-148.5 101.5 0 6.8-1.3 23.6 0 33.8l52.6-60.6c11-11.4 23.6-19.4 30-17.2a13 13 0 0 1 7.8 16.1l-87 275.7c-10 32.2 9 63.8 55.1 71 67.9 0 108-43.6 147.5-100.4',
  } as ElementIcon,

  /** 警告提示（实心三角形感叹号） */
  warning: {
    viewBox: '0 0 1024 1024',
    path: 'M512 64a448 448 0 1 1 0 896 448 448 0 0 1 0-896m0 192a58.4 58.4 0 0 0-58.2 63.7L477 576.1a35 35 0 0 0 69.8 0l23.3-256.4A58.4 58.4 0 0 0 512 256m0 512a51.2 51.2 0 1 0 0-102.4 51.2 51.2 0 0 0 0 102.4',
  } as ElementIcon,

  /** 错误提示（实心圆圈 X） */
  error: {
    viewBox: '0 0 1024 1024',
    path: 'M512 64a448 448 0 1 1 0 896 448 448 0 0 1 0-896m0 393.7L408 353.6a38.4 38.4 0 1 0-54.4 54.3l104 104.1-104 104a38.4 38.4 0 1 0 54.3 54.4l104.1-104 104 104a38.4 38.4 0 1 0 54.4-54.3L566.4 512l104-104a38.4 38.4 0 1 0-54.3-54.4z',
  } as ElementIcon,

  /** 成功提示（实心圆圈对勾） */
  success: {
    viewBox: '0 0 1024 1024',
    path: 'M512 64a448 448 0 1 1 0 896 448 448 0 0 1 0-896m-55.8 536.4-99.5-99.6a38.4 38.4 0 1 0-54.4 54.3L429.1 682a38.3 38.3 0 0 0 54.3 0l262.4-262.5a38.4 38.4 0 1 0-54.3-54.3z',
  } as ElementIcon,

  /** 调试提示（暂用 info 图标） */
  debug: {
    viewBox: '0 0 1024 1024',
    path: 'M512 64a448 448 0 1 1 0 896 448 448 0 0 1 0-896m67.2 275c33.3 0 60.3-23 60.3-57.3s-27-57.3-60.3-57.3-60.2 23-60.2 57.3 27 57.4 60.2 57.4m11.7 360.1c0-6.8 2.4-24.6 1-34.8L539.3 725c-10.9 11.4-24.5 19.4-30.9 17.3a13 13 0 0 1-8.2-14.7l87.6-277c7.2-35.2-12.5-67.2-54.3-71.3-44 0-109 44.7-148.5 101.5 0 6.8-1.3 23.6 0 33.8l52.6-60.6c11-11.4 23.6-19.4 30-17.2a13 13 0 0 1 7.8 16.1l-87 275.7c-10 32.2 9 63.8 55.1 71 67.9 0 108-43.6 147.5-100.4',
  } as ElementIcon,
} as const

export type ElementIconName = keyof typeof elementIcons
