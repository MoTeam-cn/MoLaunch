<script setup lang="ts">
/**
 * HTML 自定义布局渲染面板
 *
 * 使用 sandbox="allow-scripts" iframe 渲染用户自定义 HTML 内容（无 allow-same-origin，沙箱隔离）：
 * - 脚本在 iframe 内执行，无法访问主窗口 DOM / cookie / localStorage
 * - window.molaunch 经 postMessage 桥接到父级 pluginSdk，父级按只读方法白名单鉴权
 * - 复用 sandbox-bootstrap 的注入脚本（与外部插件沙箱同一协议）
 */
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { pluginSdk } from '@/plugins/sdk'
import { buildSandboxHtml } from '@/plugins/sandbox/sandbox-bootstrap'
import { ExclamationTriangleIcon } from '@heroicons/vue/24/outline'

const props = defineProps<{
  /** HTML 内容 */
  content: string
}>()

/** iframe 引用 */
const iframeRef = ref<HTMLIFrameElement | null>(null)
/** 注入到 iframe 的 HTML（srcdoc） */
const sandboxHtml = ref('')
/** 加载错误 */
const error = ref<string | null>(null)

/** 允许调用的只读 SDK 方法白名单（spawnProcess / createWindow 等敏感方法一律拒绝） */
const ALLOWED_METHODS = new Set([
  'getConfig',
  'listInstalledVersions',
  'listInstalledVersionsWithType',
  'listLaunchHistory',
  'getSystemMemory',
  'getRunningGamePid',
  'getCacheStats',
])
/** 始终允许的方法（无敏感数据） */
const ALWAYS_ALLOWED = new Set(['emit', 'log'])

/** 构建沙箱 HTML（注入 bootstrap 脚本，window.molaunch 在用户脚本运行前就绪） */
function buildSandbox() {
  try {
    if (!props.content.trim()) {
      error.value = 'HTML 内容为空'
      sandboxHtml.value = ''
      return
    }
    sandboxHtml.value = buildSandboxHtml(props.content, 'custom-layout')
    error.value = null
  } catch (e) {
    error.value = String(e)
  }
}

/**
 * 处理来自 iframe 的请求消息
 *
 * 根据方法名转发到 pluginSdk，并按白名单拒绝未授权调用。
 */
async function handleMessage(event: MessageEvent) {
  // 仅接受来自当前 iframe 的消息
  if (event.source !== iframeRef.value?.contentWindow) return

  const data = event.data
  if (!data || typeof data !== 'object') return

  // 沙箱就绪通知
  if (data.type === 'ready') return

  // 请求消息
  if (data.type === 'request' && typeof data.id === 'string' && typeof data.method === 'string') {
    const { id, method, args } = data
    const sandboxWindow = iframeRef.value?.contentWindow

    // 权限校验：未在白名单内拒绝
    if (!ALWAYS_ALLOWED.has(method) && !ALLOWED_METHODS.has(method)) {
      sandboxWindow?.postMessage(
        { type: 'response', id, error: `权限拒绝：自定义 HTML 布局不支持 ${method} 方法` },
        '*',
      )
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

onMounted(() => {
  window.addEventListener('message', handleMessage)
  buildSandbox()
})

onUnmounted(() => {
  window.removeEventListener('message', handleMessage)
})

watch(() => props.content, buildSandbox)
</script>

<template>
  <div class="h-full w-full">
    <!-- 加载错误 -->
    <div
      v-if="error"
      class="flex h-full flex-col items-center justify-center p-6 text-center"
    >
      <ExclamationTriangleIcon class="mb-3 h-10 w-10 text-yellow-500" />
      <p class="text-sm font-medium text-gray-900">HTML 布局加载失败</p>
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
      title="custom-layout-sandbox"
    />
  </div>
</template>