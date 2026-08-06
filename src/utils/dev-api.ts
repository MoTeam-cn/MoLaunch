/**
 * 调试 API（仅 dev 模式可用）
 *
 * 挂载到 window.molaunch，供 DevTools Console 调用（molaunch.help() 查看命令列表）；
 * 仅 import.meta.env.DEV 时挂载，store 动态 import 不影响首屏体积。
 */
import { invoke } from '@tauri-apps/api/core'
import type { Router } from 'vue-router'
import { openPickerWindow, openDisplayWindow } from '@/utils/picker-window'
import { PICKER_TEMPLATES } from '@/config/picker-templates'
import { showCrashDialog, type CrashInfo } from '@/utils/crashDialog'

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
  /** 刷新当前页面；force=true 时强制无缓存刷新（URL 追加时间戳绕过 WebView2 缓存） */
  reload(force?: boolean): Promise<void>
  /** 调用后端 tools_manager IPC */
  tools<T = unknown>(action: string, params?: Record<string, unknown>): Promise<T>
  /** 调用后端 frp_manager IPC */
  frp<T = unknown>(action: string, params?: Record<string, unknown>): Promise<T>
  /** 调用后端 experimental_manager IPC（AI action 已并入实验性分发） */
  ai<T = unknown>(action: string, params?: Record<string, unknown>): Promise<T>
  /** 触发崩溃弹窗（演示用；正常情况下游戏崩溃后自动弹出，平时难以复现） */
  showCrashDialog(): void
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

  molaunch.reload(force?)
      刷新当前页面
      - 无参 / false：普通刷新
      - true：强制无缓存刷新（URL 追加时间戳后重新加载，绕过 WebView2 本地缓存）

  molaunch.tools(action, params?)
      调用后端 tools_manager IPC，返回 Promise
      - action: 子命令（如 'list_open_ports' / 'tcp_check' / 'network_latency_test' / 'server_ping'）
      - params: 子命令参数对象

  molaunch.frp(action, params?)
      调用后端 frp_manager IPC，返回 Promise
      - action: 子命令（如 'list_tunnels' / 'list_providers' / 'start_tunnel' / 'list_public_servers'）
      - params: 子命令参数对象

  molaunch.ai(action, params?)
      调用后端 experimental_manager IPC（AI action 已并入实验性分发），返回 Promise
      - action: 子命令（如 'analyze_crash' / 'check_status' / 'save_config' / 'load_config'）
      - params: 子命令参数对象

  molaunch.showCrashDialog()
      触发错误日志弹窗（演示用样例数据；游戏崩溃后会自动弹出，本文用于检查展示）

  molaunch.stores()
      返回所有 Pinia store 的 $state（Promise<Record<string, unknown>>）

示例：
  await molaunch.pickPort()
  await molaunch.picker('confirm', { message: '测试弹窗' })
  await molaunch.picker('qrcode', { text: 'https://molaunch.moiu.cn' })
  await molaunch.tools('list_open_ports')
  await molaunch.frp('list_tunnels')
  await molaunch.ai('check_status')
  await molaunch.showCrashDialog()
  await molaunch.navigate('/apps/online')
  await molaunch.reload()
  await molaunch.reload(true)
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
    async reload(force = false) {
      const loc = window.location
      if (!force) {
        // 普通刷新
        loc.reload()
        return
      }
      // 强制无缓存：URL 追加时间戳参数后重新导航，URL 变化使浏览器绕过本地缓存重新拉取资源
      const url = new URL(loc.href)
      url.searchParams.set('_molaunch_reload', String(Date.now()))
      loc.href = url.toString()
    },
    async tools(action, params) {
      return invoke('tools_manager', { req: { action, params: params ?? {} } })
    },
    async frp(action, params) {
      return invoke('frp_manager', { req: { action, params: params ?? {} } })
    },
    async ai(action, params) {
      return invoke('experimental_manager', { req: { action, params: params ?? {} } })
    },
    showCrashDialog() {
      const sample: CrashInfo = {
        reason: '无法加载主类：net.minecraft.client.main.Main',
        category: 'Java',
        log_lines: [
          'Error: A JNI error has occurred, please check your installation and try again',
          'Exception in thread "main" java.lang.NoClassDefFoundError: net/minecraft/client/main/Main',
          'Caused by: java.lang.ClassNotFoundException: net.minecraft.client.main.Main',
        ],
        suggestion:
          '游戏主类加载失败，通常由 Java 版本不匹配或版本文件损坏导致。\n建议：1. 更换 Java 版本（1.8.x）；2. 重新安装该版本；3. 检查版本 JSON 中主类配置。',
        problematic_mod: null,
        crash_report_path: 'C:\\Users\\Test\\.minecraft\\crash-reports\\crash-2026-08-05_12.30.00-server.txt',
        log_tail: [
          '[12:30:00] [main/INFO]: Loading Minecraft 1.8.9 with Fabric Loader 0.16.5',
          '[12:30:00] [main/INFO]: Loading 12 mods: fabricloader, fabric-api, ...',
          '[12:30:01] [main/ERROR]: Failed to start the minecraft server',
        ],
      }
      showCrashDialog(sample)
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
