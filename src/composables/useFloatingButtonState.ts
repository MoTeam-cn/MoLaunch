/**
 * 悬浮按钮布局协调状态
 *
 * BackToTop 与 DownloadPanel 同处右下角，需动态避让：
 * - BackToTop 不可见时：DownloadPanel 贴底（bottom-6, 24px）
 * - BackToTop 可见时：DownloadPanel 上移（bottom-24, 96px）腾出空间
 *
 * 通过模块级共享 ref 实现，BackToTop 写入、DownloadPanel 读取，
 * 无需 Pinia store 或事件总线（仅两个组件协调，过度抽象反而增加复杂度）。
 */

import { ref } from 'vue'

/** BackToTop 按钮当前是否可见（BackToTop 写入，DownloadPanel 读取） */
export const backToTopVisible = ref(false)
