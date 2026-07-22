<script setup lang="ts">
/**
 * HTML 自定义布局渲染面板
 *
 * 使用 shadow DOM 渲染用户自定义 HTML 内容：
 * - CSS 隔离：shadow root 内的样式不影响主页面，主页面的样式也不泄漏到 shadow 内
 * - JS 执行：用户脚本通过 new Function 在主窗口上下文执行，可直接调用 window.molaunch SDK
 * - 无 iframe：消除 sandbox="allow-scripts allow-same-origin" 安全警告
 *
 * 与 CustomLayoutPanel 的 html section 区别：
 * - 本组件用于纯 HTML 格式的完整自定义布局（非 JSON/XML 结构化布局）
 * - window.molaunch 通过 Proxy 代理到 pluginSdk（开放所有只读 SDK 方法）
 */
import { ref, onMounted, watch, nextTick } from 'vue'
import { pluginSdk } from '@/plugins/sdk'
import { ExclamationTriangleIcon } from '@heroicons/vue/24/outline'
import { safeCallSync } from '@/utils/async'

const props = defineProps<{
  /** HTML 内容 */
  content: string
}>()

/** 容器引用 */
const containerRef = ref<HTMLDivElement | null>(null)
/** 加载错误 */
const error = ref<string | null>(null)

/** 不允许自定义 HTML 调用的方法（spawnProcess / createWindow 仅外部插件可用） */
const BLOCKED_METHODS = new Set(['spawnProcess', 'createWindow'])

/** 确保 window.molaunch API 已定义（通过 Proxy 代理到 pluginSdk） */
let molaunchApiReady = false
function setupMolaunchApi() {
  if (molaunchApiReady) return
  molaunchApiReady = true

  const proxy = new Proxy({} as Record<string, (...args: unknown[]) => Promise<unknown>>, {
    get(_, prop) {
      const method = prop as string
      if (BLOCKED_METHODS.has(method)) {
        return () => Promise.reject(new Error(`自定义 HTML 布局不支持 ${method} 方法`))
      }
      const fn = (pluginSdk as unknown as Record<string, (...a: unknown[]) => Promise<unknown>>)[method]
      if (typeof fn === 'function') {
        return (...args: unknown[]) => fn.apply(pluginSdk, args)
      }
      return undefined
    },
  })

  ;(window as Record<string, unknown>).molaunch = proxy
}

/** 用 shadow DOM 渲染 HTML 内容 */
function renderHtml() {
  const container = containerRef.value
  if (!container) return

  try {
    if (!props.content.trim()) {
      error.value = 'HTML 内容为空'
      return
    }

    // 获取或创建 shadow root
    let shadow = container.shadowRoot
    if (!shadow) {
      shadow = container.attachShadow({ mode: 'open' })
    }
    shadow.innerHTML = ''

    // 注入用户 HTML
    const wrapper = document.createElement('div')
    wrapper.innerHTML = props.content
    shadow.appendChild(wrapper)

    // 确保 window.molaunch API 可用
    setupMolaunchApi()

    // 提取并执行 <script> 标签（innerHTML 插入的 script 不会自动执行）
    const scripts = wrapper.querySelectorAll('script')
    scripts.forEach((scriptEl) => {
      const code = scriptEl.textContent || ''
      if (code.trim()) {
        safeCallSync(() => new Function(code)(), '[HtmlLayout] run user script')
      }
      // 移除已执行的 script 标签
      scriptEl.remove()
    })

    error.value = null
  } catch (e) {
    error.value = String(e)
  }
}

onMounted(() => {
  nextTick(renderHtml)
})

watch(() => props.content, () => {
  nextTick(renderHtml)
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
      <p class="text-sm font-medium text-gray-900">HTML 布局加载失败</p>
      <p class="mt-1 text-xs text-gray-500">{{ error }}</p>
    </div>

    <!-- shadow DOM 容器 -->
    <div ref="containerRef" class="h-full w-full" />
  </div>
</template>
