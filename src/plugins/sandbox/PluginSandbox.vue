<script setup lang="ts">
/**
 * 外部插件沙箱代理组件
 *
 * 用于在主页右侧内容区渲染外部插件：
 * - 通过后端命令读取插件入口 HTML 文件内容
 * - 在 bootstrap 脚本中注入 `window.molaunch` 全局 API
 * - 使用 `<iframe sandbox="allow-scripts">` 加载（无 allow-same-origin，沙箱隔离）
 * - 父级监听 iframe 的 postMessage 请求，转发到 pluginSdk 执行
 * - 根据 manifest.permissions 白名单拒绝未授权的方法调用
 * - 监听 plugin:game-launch / plugin:game-exit 等事件，转发给 iframe
 */
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { pluginSdk } from '@/plugins/sdk'
import type { ProcessResult, CreateWindowOptions } from '@/plugins/sdk'
import { buildSandboxHtml } from './sandbox-bootstrap'
import { readExternalPluginFile } from '@/utils/api/plugins'
import { ExclamationTriangleIcon } from '@heroicons/vue/24/outline'

const props = defineProps<{
  /** 插件 ID */
  pluginId: string
  /** HTML 入口文件相对路径（如 "index.html"） */
  entry: string
  /** 权限白名单（SDK 方法名数组） */
  permissions?: string[]
}>()

/** 注入到 iframe 的 HTML（srcdoc） */
const sandboxHtml = ref('')
/** 加载错误 */
const error = ref<string | null>(null)
/** iframe 引用 */
const iframeRef = ref<HTMLIFrameElement | null>(null)

/** 始终允许的方法（无敏感数据） */
const ALWAYS_ALLOWED = new Set(['emit', 'log'])

/** 待响应的请求映射（id → resolve/reject） */
const pendingRequests = new Map<string, { resolve: (v: unknown) => void; reject: (e: Error) => void }>()

/** 加载插件 HTML */
async function loadPluginHtml() {
  try {
    const html = await readExternalPluginFile(props.pluginId, props.entry)
    sandboxHtml.value = buildSandboxHtml(html, props.pluginId)
    error.value = null
  } catch (e) {
    error.value = String(e)
    pluginSdk.log('error', `[PluginSandbox] 加载插件 ${props.pluginId} 失败: ${e}`)
  }
}

/**
 * 处理来自 iframe 的请求消息
 *
 * 根据方法名转发到 pluginSdk，并根据 permissions 白名单拒绝未授权调用。
 */
async function handleMessage(event: MessageEvent) {
  // 仅接受来自当前 iframe 的消息
  if (event.source !== iframeRef.value?.contentWindow) return

  const data = event.data
  if (!data || typeof data !== 'object') return

  // 沙箱就绪通知
  if (data.type === 'ready') {
    pluginSdk.log('info', `[PluginSandbox] 沙箱就绪: ${props.pluginId}`)
    return
  }

  // 请求消息
  if (data.type === 'request' && typeof data.id === 'string' && typeof data.method === 'string') {
    const { id, method, args } = data
    const sandboxWindow = iframeRef.value?.contentWindow

    // 权限校验：未在白名单内拒绝
    if (!ALWAYS_ALLOWED.has(method) && !(props.permissions ?? []).includes(method)) {
      const errMsg = `权限拒绝：插件 ${props.pluginId} 未声明 ${method} 权限`
      pluginSdk.log('warn', `[PluginSandbox] ${errMsg}`)
      sandboxWindow?.postMessage(
        { type: 'response', id, error: errMsg },
        '*',
      )
      return
    }

    // spawnProcess 特殊处理：注入 pluginId 上下文，直接调用后端命令
    // args 格式: [command, args[], options?]
    if (method === 'spawnProcess') {
      try {
        const command = String(args?.[0] ?? '')
        const procArgs = Array.isArray(args?.[1]) ? args[1] as string[] : []
        const options = (args?.[2] ?? {}) as { cwd?: string }
        const result = await invoke<ProcessResult>('plugin_spawn_process', {
          pluginId: props.pluginId,
          command,
          args: procArgs,
          cwd: options.cwd ?? null,
        })
        sandboxWindow?.postMessage({ type: 'response', id, result }, '*')
      } catch (e) {
        sandboxWindow?.postMessage(
          { type: 'response', id, error: e instanceof Error ? e.message : String(e) },
          '*',
        )
      }
      return
    }

    // createWindow 特殊处理：注入 pluginId 上下文，直接调用后端命令
    // args 格式: [options]
    if (method === 'createWindow') {
      try {
        const opts = (args?.[0] ?? {}) as Partial<CreateWindowOptions>
        await invoke('plugin_create_window', {
          pluginId: props.pluginId,
          label: String(opts.label ?? ''),
          url: String(opts.url ?? ''),
          title: String(opts.title ?? ''),
          width: opts.width ?? null,
          height: opts.height ?? null,
        })
        sandboxWindow?.postMessage({ type: 'response', id, result: null }, '*')
      } catch (e) {
        sandboxWindow?.postMessage(
          { type: 'response', id, error: e instanceof Error ? e.message : String(e) },
          '*',
        )
      }
      return
    }

    // 转发到 pluginSdk
    try {
      const sdkMethod = (pluginSdk as unknown as Record<string, (...a: unknown[]) => Promise<unknown>>)[method]
      if (typeof sdkMethod !== 'function') {
        throw new Error(`未知 SDK 方法: ${method}`)
      }
      const result = await sdkMethod.apply(pluginSdk, args ?? [])
      sandboxWindow?.postMessage(
        { type: 'response', id, result },
        '*',
      )
    } catch (e) {
      sandboxWindow?.postMessage(
        { type: 'response', id, error: e instanceof Error ? e.message : String(e) },
        '*',
      )
    }
    return
  }
}

/**
 * 推送事件到 iframe
 *
 * 由 pluginStore.notifyGameLaunch / notifyGameExit 等触发，
 * 通过 window 自定义事件桥接到沙箱。
 */
function pushEvent(name: string, payload?: unknown) {
  const sandboxWindow = iframeRef.value?.contentWindow
  if (!sandboxWindow) return
  sandboxWindow.postMessage(
    { type: 'event', name, payload },
    '*',
  )
}

/** 事件桥接：监听 window 上的 plugin: 事件并转发到 iframe */
function onPluginGameLaunch() {
  pushEvent('game-launch')
}
function onPluginGameExit(e: Event) {
  const detail = (e as CustomEvent).detail as { versionId: string; exitCode: number | null } | undefined
  pushEvent('game-exit', detail)
}

onMounted(async () => {
  window.addEventListener('message', handleMessage)
  window.addEventListener('plugin:game-launch', onPluginGameLaunch)
  window.addEventListener('plugin:game-exit', onPluginGameExit)
  await loadPluginHtml()
})

onUnmounted(() => {
  window.removeEventListener('message', handleMessage)
  window.removeEventListener('plugin:game-launch', onPluginGameLaunch)
  window.removeEventListener('plugin:game-exit', onPluginGameExit)
  // 拒绝所有未完成的 pending 请求
  for (const { reject } of pendingRequests.values()) {
    reject(new Error('插件已卸载'))
  }
  pendingRequests.clear()
})
</script>

<template>
  <div class="h-full w-full">
    <!-- 加载错误 -->
    <div
      v-if="error"
      class="flex h-full flex-col items-center justify-center p-6 text-center"
    >
      <ExclamationTriangleIcon class="mb-3 h-10 w-10 text-yellow-500" />
      <p class="text-sm font-medium text-gray-900">插件加载失败</p>
      <p class="mt-1 text-xs text-gray-500">{{ error }}</p>
    </div>

    <!-- 沙箱 iframe -->
    <!--
      sandbox="allow-scripts" 允许执行 JS，但不赋予同源，无法访问父窗口 DOM / cookie / localStorage
      srcdoc 注入 HTML 内容（避免文件协议路径问题）
    -->
    <iframe
      v-else
      ref="iframeRef"
      class="h-full w-full border-0"
      sandbox="allow-scripts"
      :srcdoc="sandboxHtml"
      title="plugin-sandbox"
    />
  </div>
</template>
