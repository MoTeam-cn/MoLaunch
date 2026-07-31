/**
 * 调试 API（仅 dev 模式可用）
 *
 * 挂载到 `window.molaunch`，提供调试入口便于在 DevTools Console 中测试：
 * - help()          打印所有可用命令
 * - templates()     列出所有可用 picker 模板名
 * - picker(t, d?)   打开 picker 子窗口（选择型返回值，展示型返回 undefined）
 * - pickPort()      打开端口选择器（返回 number | null）
 * - navigate(path)  路由跳转
 * - tools(action, params?)  调用后端 tools_manager IPC
 * - frp(action, params?)    调用后端 frp_manager IPC
 * - stores()        返回所有 Pinia store 的 $state
 *
 * 用法：在 DevTools Console 输入 `molaunch.help()` 查看帮助
 *
 * 设计说明：
 * - 仅在 `import.meta.env.DEV` 时挂载，生产构建中不存在此 API，避免体积与安全风险
 * - 通过 `Object.defineProperty` 设置 writable:false 防止被覆盖
 * - 所有 Pinia store 通过动态 import 加载，不影响首屏 bundle 体积
 * - picker 子命令复用 `utils/picker-window.ts` 既有便捷函数，避免重复实现
 */
import { invoke } from '@tauri-apps/api/core'
import type { Router } from 'vue-router'
import { openPickerWindow, openDisplayWindow } from '@/utils/picker-window'
import { PICKER_TEMPLATES } from '@/config/picker-templates'

/** 调试 API 接口 */
export interface MolaunchDevAPI {
  /** 打印帮助信息 */
  help(): void
  /** 列出所有可用 picker 模板名 */
  templates(): string[]
  /** 打开 picker 子窗口；选择型模板返回用户选中的值，展示型模板返回 undefined */
  picker(template: string, data?: Record<string, unknown>): Promise<unknown>
  /** 打开端口选择器，返回端口号；用户取消返回 null */
  pickPort(): Promise<number | null>
  /** 路由跳转 */
  navigate(path: string): Promise<unknown>
  /** 调用后端 tools_manager IPC */
  tools<T = unknown>(action: string, params?: Record<string, unknown>): Promise<T>
  /** 调用后端 frp_manager IPC */
  frp<T = unknown>(action: string, params?: Record<string, unknown>): Promise<T>
  /** 返回所有 Pinia store 的 $state */
  stores(): Promise<Record<string, unknown>>
}

/** 全局 Window 类型扩展，使 `window.molaunch` 在 TypeScript 中可识别 */
declare global {
  interface Window {
    molaunch?: MolaunchDevAPI
  }
}

/** 展示型模板：关窗即结束，不返回值 */
const DISPLAY_TEMPLATES = ['redirect', 'info', 'image-viewer', 'markdown', 'qrcode']

const HELP_TEXT = `
MoLaunch Dev API 可用命令：

  molaunch.help()
      打印本帮助信息

  molaunch.templates()
      列出所有可用 picker 模板名（数组）

  molaunch.picker(template, data?)
      打开 picker 子窗口，选择型模板返回用户选择的值，展示型模板返回 undefined
      - template: 模板名（如 port-picker / confirm / info / image-viewer / markdown / qrcode / redirect）
      - data:     传给模板的数据（如 { message: 'xx' } / { url: 'xx' } / { text: 'xx' }）

  molaunch.pickPort()
      打开端口选择器，返回端口号（取消返回 null）

  molaunch.navigate(path)
      路由跳转（如 '/apps/online'、'/apps/settings'、'/apps/versions'）

  molaunch.tools(action, params?)
      调用后端 tools_manager IPC，返回 Promise
      - action: 子命令（如 'list_open_ports' / 'tcp_check' / 'network_latency_test' / 'server_ping'）
      - params: 子命令参数对象

  molaunch.frp(action, params?)
      调用后端 frp_manager IPC，返回 Promise
      - action: 子命令（如 'list_tunnels' / 'list_providers' / 'start_tunnel' / 'list_public_servers'）
      - params: 子命令参数对象

  molaunch.stores()
      返回所有 Pinia store 的 $state（Promise<Record<string, unknown>>）

示例：
  await molaunch.pickPort()
  await molaunch.picker('confirm', { message: '测试弹窗' })
  await molaunch.picker('qrcode', { text: 'https://molaunch.moiu.cn' })
  await molaunch.tools('list_open_ports')
  await molaunch.frp('list_tunnels')
  await molaunch.navigate('/apps/online')
  const s = await molaunch.stores(); console.log(s.frp)
`.trim()

/** 动态加载所有 Pinia store 并返回 $state 映射 */
async function loadAllStores(): Promise<Record<string, unknown>> {
  const loaders: Record<string, () => Promise<{ $state: unknown }>> = {
    auth: () => import('@/stores/auth').then(m => m.useAuthStore()),
    frp: () => import('@/stores/frp').then(m => m.useFrpStore()),
    java: () => import('@/stores/java').then(m => m.useJavaStore()),
    online: () => import('@/stores/online').then(m => m.useOnlineStore()),
    plugins: () => import('@/stores/plugins').then(m => m.usePluginStore()),
    sdk: () => import('@/stores/sdk').then(m => m.useSdkStore()),
    settings: () => import('@/stores/settings').then(m => m.useSettingsStore()),
    version: () => import('@/stores/version').then(m => m.useVersionStore()),
  }
  const result: Record<string, unknown> = {}
  for (const [name, loader] of Object.entries(loaders)) {
    try {
      const store = await loader()
      result[name] = store.$state
    } catch (e) {
      result[name] = { _error: String(e) }
    }
  }
  return result
}

/**
 * 安装调试 API 到 `window.molaunch`（仅 dev 模式）
 *
 * @param router 应用路由实例（用于 navigate 子命令）
 */
export function setupDevApi(router: Router): void {
  if (!import.meta.env.DEV) return

  const api: MolaunchDevAPI = {
    help() {
      console.log('%c[MoLaunch Dev API]\n%s', 'color:#165dff;font-weight:bold', HELP_TEXT)
    },
    templates() {
      return Object.keys(PICKER_TEMPLATES)
    },
    async picker(template, data) {
      const config = PICKER_TEMPLATES[template]
      if (!config) throw new Error(`未知的 picker 模板: ${template}，可用模板: ${Object.keys(PICKER_TEMPLATES).join(', ')}`)
      if (DISPLAY_TEMPLATES.includes(template)) {
        await openDisplayWindow({
          title: config.title,
          template,
          data: data ?? {},
        })
        return undefined
      }
      return openPickerWindow({
        title: config.title,
        template,
        data: data ?? {},
      })
    },
    async pickPort() {
      try {
        const value = await openPickerWindow({
          title: '选择本机端口',
          template: 'port-picker',
          data: {},
        })
        return Number(value)
      } catch {
        return null
      }
    },
    navigate(path) {
      return router.push(path)
    },
    async tools(action, params) {
      return invoke('tools_manager', { req: { action, params: params ?? {} } })
    },
    async frp(action, params) {
      return invoke('frp_manager', { req: { action, params: params ?? {} } })
    },
    stores: loadAllStores,
  }

  Object.defineProperty(window, 'molaunch', {
    value: api,
    writable: false,
    configurable: false,
    enumerable: true,
  })

  console.log(
    '%c[MoLaunch Dev API]%c 已就绪，输入 molaunch.help() 查看用法',
    'color:#165dff;font-weight:bold',
    'color:inherit',
  )
}
