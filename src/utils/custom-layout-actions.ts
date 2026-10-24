/**
 * 自定义布局按钮动作执行器
 *
 * 支持四类动作：启动游戏（交由主页启动面板处理）、打开链接、切换页面、复制文本。
 * 所有动作都做了白名单/协议校验，布局内容即使来自远程 URL 也无法执行任意代码。
 */

import { open } from '@tauri-apps/plugin-shell'
import type { Router } from 'vue-router'
import { ALLOWED_NAV_PATHS } from '@/plugins/custom-layout/schema-types'
import type { ButtonAction } from '@/plugins/custom-layout/types'
import { toastError, toastInfo } from '@/utils/toast'

/** 主页启动面板监听的「请求启动」事件名 */
export const LAUNCH_REQUEST_EVENT = 'molaunch:launch-request'

/** 执行一个布局按钮动作；返回是否成功 */
export async function runLayoutAction(action: ButtonAction, router: Router): Promise<boolean> {
  switch (action.type) {
    case 'launch': {
      window.dispatchEvent(
        new CustomEvent(LAUNCH_REQUEST_EVENT, { detail: { version: action.version ?? '' } }),
      )
      return true
    }
    case 'open_url': {
      if (!/^https?:\/\//i.test(action.url)) {
        toastError('仅支持 http(s) 链接')
        return false
      }
      try {
        await open(action.url)
        return true
      } catch {
        toastError('打开链接失败')
        return false
      }
    }
    case 'navigate': {
      if (!ALLOWED_NAV_PATHS.has(action.path)) {
        toastError('该页面不允许跳转')
        return false
      }
      await router.push(action.path)
      return true
    }
    case 'copy': {
      try {
        await navigator.clipboard.writeText(action.text)
        toastInfo('已复制到剪贴板')
        return true
      } catch {
        toastError('复制失败')
        return false
      }
    }
    default:
      return false
  }
}
