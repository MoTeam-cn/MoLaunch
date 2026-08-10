/**
 * 悬浮按钮布局协调状态（模块级共享 ref）
 *
 * BackToTop 与 DownloadPanel 同处右下角：BackToTop 可见时 DownloadPanel 上移避让。
 * 仅两组件协调，用模块级共享 ref 即可，无需 Pinia store 或事件总线。
 */

import { ref } from 'vue'

/** BackToTop 按钮当前是否可见（BackToTop 写入，DownloadPanel 读取） */
export const backToTopVisible = ref(false)

/**
 * BackToTop 是否允许显示（页面白名单开关）
 *
 * 页内视图切换（非路由切换）不会触发 BackToTop 的路由重置逻辑，
 * 残留按钮会遮挡新视图右下角操作（如 LoaderSelect 的「开始安装」）。
 * 需要隐藏的视图在展开时置 false、收起/离开页面时恢复 true。
 */
export const backToTopEnabled = ref(true)
