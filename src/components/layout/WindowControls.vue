<script setup lang="ts">
/**
 * 标题栏窗口控制按钮（最小化 / 最大化还原 / 关闭）
 *
 * 主窗口 decorations: false，没有系统标题栏，按钮需自己画。
 * 关闭按钮不直接关窗，只把意图抛给上层，由上层按 closeBehavior 分流。
 */
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { toastError } from '@/utils/toast'

const emit = defineEmits<{ close: [] }>()

const appWindow = getCurrentWebviewWindow()
const maximized = ref(false)
let unlisten: (() => void) | null = null

async function syncMaximized(): Promise<void> {
  maximized.value = await appWindow.isMaximized().catch(() => false)
}

async function toggleMaximize(): Promise<void> {
  await appWindow.toggleMaximize().catch(() => toastError('切换窗口大小失败，请重试'))
  await syncMaximized()
}

function minimize(): void {
  void appWindow.minimize().catch(() => toastError('最小化失败，请重试'))
}

onMounted(async () => {
  await syncMaximized()
  unlisten = await appWindow.onResized(() => {
    void syncMaximized()
  })
})

onBeforeUnmount(() => {
  unlisten?.()
  unlisten = null
})
</script>

<template>
  <!-- 保留原生 button：窗口控制需 h-full 撑满 48px 标题栏，
       Button.vue 的 scoped size 类固定 height 无法被 h-full 覆盖 -->
  <div class="flex items-center h-full">
    <!-- 最小化 -->
    <button
      class="h-full w-11 flex items-center justify-center hover:bg-white/10 transition-colors group"
      aria-label="最小化"
      @click="minimize()"
    >
      <svg class="w-3.5 h-3.5 text-white/60 group-hover:text-white" viewBox="0 0 12 12" fill="none">
        <rect x="1" y="5.5" width="10" height="1" rx="0.5" fill="currentColor" />
      </svg>
    </button>

    <!-- 最大化 / 还原 -->
    <button
      class="h-full w-11 flex items-center justify-center hover:bg-white/10 transition-colors group"
      :aria-label="maximized ? '还原窗口' : '最大化窗口'"
      @click="toggleMaximize()"
    >
      <svg v-if="maximized" class="w-3.5 h-3.5 text-white/60 group-hover:text-white" viewBox="0 0 12 12" fill="none">
        <rect x="1.2" y="3.2" width="7.6" height="7.6" rx="0.8" stroke="currentColor" stroke-width="1.1" />
        <path
          d="M3.6 3.2V1.6c0-.44.36-.8.8-.8h5.8c.44 0 .8.36.8.8v5.8c0 .44-.36.8-.8.8H8.8"
          stroke="currentColor"
          stroke-width="1.1"
          stroke-linecap="round"
        />
      </svg>
      <svg v-else class="w-3.5 h-3.5 text-white/60 group-hover:text-white" viewBox="0 0 12 12" fill="none">
        <rect x="1.5" y="1.5" width="9" height="9" rx="1" stroke="currentColor" stroke-width="1.1" />
      </svg>
    </button>

    <!-- 关闭：只抛意图，由上层按 closeBehavior 处理 -->
    <button
      class="h-full w-11 flex items-center justify-center hover:bg-red-500 transition-colors group"
      aria-label="关闭"
      @click="emit('close')"
    >
      <svg class="w-3.5 h-3.5 text-white/60 group-hover:text-white" viewBox="0 0 12 12" fill="none">
        <path
          d="M1.5 1.5L10.5 10.5M1.5 10.5L10.5 1.5"
          stroke="currentColor"
          stroke-width="1.2"
          stroke-linecap="round"
        />
      </svg>
    </button>
  </div>
</template>
