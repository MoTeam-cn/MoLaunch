/**
 * 选择器子窗口工具
 *
 * 通过后端 open_picker_window action 创建 Tauri 子窗口，由后端按 template
 * 名称加载 resources 中的 HTML 模板并注入 data，用户在子窗口中点击选项后
 * 通过 picker-result 事件返回选中值。
 *
 * ## 两类窗口
 *
 * - **选择型**（openPickerWindow）：用户必须点击选项，通过 picker-result 返回值；
 *   用户关窗视为取消，reject。适用：port-picker、confirm。
 * - **展示型**（openDisplayWindow）：纯展示内容，用户关窗即结束，resolve；
 *   不需要 picker-result。适用：redirect、info、image-viewer、markdown、qrcode。
 *
 * ## CSP 传递
 *
 * 各模板的 CSP 在 `picker-templates.ts` 中配置，便捷函数自动从配置中读取
 * 并通过 params.csp 传给后端，后端注入到 picker:// 响应头中。
 *
 * 用法：
 *   const value = await openPickerWindow({ template: 'port-picker', title: '选择端口' })
 *   const ok = await openConfirmWindow({ message: '确认删除？' })
 *   await openMarkdownWindow({ content: '# Hello' })
 */

import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getTemplateConfig, isUrlAllowed } from '@/config/picker-templates'

export interface PickerWindowParams {
  title: string
  template: string
  data?: unknown
  width?: number
  height?: number
  /** Content-Security-Policy 策略字符串，覆盖模板配置中的 csp */
  csp?: string
}

/**
 * 打开选择器子窗口，返回用户选中的值
 *
 * 内部流程：
 * 1. 先注册 picker-result / picker-cancelled 事件监听
 * 2. 调用 tools_manager open_picker_window action 创建子窗口
 * 3. 用户点击选项 → picker-result 事件 → resolve(value)
 * 4. 用户关闭窗口 → picker-cancelled 事件 → reject
 *
 * 自动从 PICKER_TEMPLATES 读取默认 width/height/csp，调用方可覆盖。
 */
export async function openPickerWindow(params: PickerWindowParams): Promise<string> {
  const config = getTemplateConfig(params.template)
  const merged: PickerWindowParams = {
    title: params.title,
    template: params.template,
    data: params.data ?? {},
    width: params.width ?? config?.width,
    height: params.height ?? config?.height,
    csp: params.csp ?? config?.csp,
  }

  let pickerId: string | null = null
  let resolveFn!: (value: string) => void
  let rejectFn!: (err: Error) => void
  const promise = new Promise<string>((resolve, reject) => {
    resolveFn = resolve
    rejectFn = reject
  })

  let unlistenResult: UnlistenFn | null = null
  let unlistenCancel: UnlistenFn | null = null
  const cleanup = () => {
    unlistenResult?.()
    unlistenCancel?.()
  }

  // 先注册事件监听（避免错过早期事件）
  unlistenResult = await listen<{ id: string; value: string }>('picker-result', (event) => {
    if (pickerId && event.payload.id === pickerId) {
      cleanup()
      resolveFn(event.payload.value)
    }
  })
  unlistenCancel = await listen<string>('picker-cancelled', (event) => {
    if (pickerId && event.payload === pickerId) {
      cleanup()
      rejectFn(new Error('用户取消选择'))
    }
  })

  // 调用后端创建子窗口
  try {
    const result = await invoke<{ id: string }>('tools_manager', {
      req: { action: 'open_picker_window', params: merged },
    })
    pickerId = result.id
  } catch (err) {
    cleanup()
    throw err
  }

  return promise
}

/**
 * 打开展示型子窗口，用户关闭窗口即 resolve
 *
 * 与 openPickerWindow 的区别：不需要用户选择，关窗即结束。
 * 适用于 redirect、info、image-viewer、markdown、qrcode 等纯展示模板。
 *
 * 自动从 PICKER_TEMPLATES 读取默认 width/height/csp，调用方可覆盖。
 */
export async function openDisplayWindow(params: PickerWindowParams): Promise<void> {
  const config = getTemplateConfig(params.template)
  const merged: PickerWindowParams = {
    title: params.title,
    template: params.template,
    data: params.data ?? {},
    width: params.width ?? config?.width,
    height: params.height ?? config?.height,
    csp: params.csp ?? config?.csp,
  }

  let pickerId: string | null = null
  let resolveFn!: () => void
  const promise = new Promise<void>((resolve) => {
    resolveFn = resolve
  })

  // 展示型窗口：监听 picker-cancelled（用户关窗）即视为正常结束
  const unlistenCancel = await listen<string>('picker-cancelled', (event) => {
    if (pickerId && event.payload === pickerId) {
      unlistenCancel()
      resolveFn()
    }
  })

  try {
    const result = await invoke<{ id: string }>('tools_manager', {
      req: { action: 'open_picker_window', params: merged },
    })
    pickerId = result.id
  } catch (err) {
    unlistenCancel()
    throw err
  }

  return promise
}

// ============== 便捷调用函数 ==============

/**
 * 打开重定向子窗口
 *
 * 先校验 URL 是否在白名单中，校验通过后创建 redirect 模板子窗口。
 * 白名单配置在 src/config/picker-templates.ts 中，修改白名单不需动此文件。
 *
 * @param url 目标 URL
 * @throws URL 不在白名单中时抛出错误
 */
export async function openRedirectWindow(url: string): Promise<void> {
  const config = getTemplateConfig('redirect')
  if (!config) throw new Error('redirect 模板配置不存在')

  if (config.allowedDomains && !isUrlAllowed(url, config.allowedDomains)) {
    throw new Error(`URL 不在白名单中: ${url}`)
  }

  await openDisplayWindow({
    title: config.title,
    template: config.template,
    data: { url },
  })
}

/**
 * 打开确认对话框
 *
 * @param params.title 标题（默认「确认」）
 * @param params.message 提示消息
 * @param params.confirmText 确认按钮文字（默认「确认」）
 * @param params.cancelText 取消按钮文字（默认「取消」）
 * @param params.danger 是否危险操作（确认按钮显示红色）
 * @returns 用户点击确认返回 true，取消返回 false
 */
export async function openConfirmWindow(params: {
  title?: string
  message: string
  confirmText?: string
  cancelText?: string
  danger?: boolean
}): Promise<boolean> {
  const config = getTemplateConfig('confirm')!
  try {
    const value = await openPickerWindow({
      title: params.title ?? config.title,
      template: 'confirm',
      data: {
        message: params.message,
        confirmText: params.confirmText ?? '确认',
        cancelText: params.cancelText ?? '取消',
        danger: params.danger ?? false,
      },
    })
    return value === 'true'
  } catch {
    // 用户关窗视为取消
    return false
  }
}

/**
 * 打开信息展示窗口
 *
 * @param params.title 标题
 * @param params.content 正文内容（支持简单 HTML：b/i/code/br/p）
 */
export async function openInfoWindow(params: {
  title: string
  content: string
}): Promise<void> {
  await openDisplayWindow({
    title: params.title,
    template: 'info',
    data: { title: params.title, content: params.content },
  })
}

/**
 * 打开图片查看器
 *
 * @param params.url 图片 URL（支持 http/https/data/blob）
 * @param params.alt 图片描述（可选）
 */
export async function openImageViewerWindow(params: {
  url: string
  alt?: string
}): Promise<void> {
  await openDisplayWindow({
    title: '图片查看',
    template: 'image-viewer',
    data: { url: params.url, alt: params.alt ?? '' },
  })
}

/**
 * 打开 Markdown 渲染窗口
 *
 * 通过 res:// 协议加载后端嵌入的 marked.min.js 渲染 markdown 文本。
 *
 * @param params.title 标题
 * @param params.content Markdown 文本
 */
export async function openMarkdownWindow(params: {
  title: string
  content: string
}): Promise<void> {
  await openDisplayWindow({
    title: params.title,
    template: 'markdown',
    data: { title: params.title, content: params.content },
  })
}

/**
 * 打开教程渲染窗口（亮色主题）
 *
 * 与 openMarkdownWindow 类似，但使用 tutorial.html 亮色模板，
 * 样式与前端 Vue 应用一致（白底灰字 + 主色蓝）。
 *
 * @param params.title 标题
 * @param params.content Markdown 文本
 */
export async function openTutorialWindow(params: {
  title: string
  content: string
}): Promise<void> {
  await openDisplayWindow({
    title: params.title,
    template: 'tutorial',
    data: { title: params.title, content: params.content },
  })
}

/**
 * 打开二维码展示窗口
 *
 * @param params.text 要生成二维码的文本（URL 或任意字符串）
 * @param params.label 二维码下方说明文字（可选）
 */
export async function openQrcodeWindow(params: {
  text: string
  label?: string
}): Promise<void> {
  await openDisplayWindow({
    title: '二维码',
    template: 'qrcode',
    data: { text: params.text, label: params.label ?? '' },
  })
}
