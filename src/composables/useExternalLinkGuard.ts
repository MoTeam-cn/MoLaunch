/**
 * 全局外部链接拦截 composable（App.vue 挂载，全局生效）
 *
 * 禁止页面在 webview 内直接跳转外部网站（否则会跳出 SPA 页面、无法返回/关闭）。
 * 复用 utils/markdown.ts 的 handleMarkdownLinkClick：任何未被组件自行处理的
 * http(s) 链接点击 → 二次确认 → 经 shell 插件在系统浏览器打开。
 */

import { onMounted, onUnmounted } from 'vue'
import { handleMarkdownLinkClick } from '@/utils/markdown'

function onGlobalClick(e: MouseEvent) {
  void handleMarkdownLinkClick(e)
}

export function useExternalLinkGuard() {
  onMounted(() => window.addEventListener('click', onGlobalClick))
  onUnmounted(() => window.removeEventListener('click', onGlobalClick))
}
