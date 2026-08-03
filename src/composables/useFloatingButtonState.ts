/**
 * 悬浮按钮布局协调状态（模块级共享 ref）
 *
 * BackToTop 与 DownloadPanel 同处右下角：BackToTop 可见时 DownloadPanel 上移避让。
 * 仅两组件协调，用模块级共享 ref 即可，无需 Pinia store 或事件总线。
 */

import { ref } from 'vue'

/** BackToTop 按钮当前是否可见（BackToTop 写入，DownloadPanel 读取） */
export const backToTopVisible = ref(false)
