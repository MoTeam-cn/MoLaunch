<script setup lang="ts">
/**
 * 更新进度弹窗
 *
 * 监听 `utils/updater.ts` 的全局响应式状态，自动显示/隐藏：
 * - `checking`：检查中 loading
 * - `available`：发现新版本，展示版本号 + 更新日志 + 按钮（立即更新 / 稍后）
 * - `downloading`：下载进度条
 * - `installing`：安装中 loading（即将重启）
 * - `error`：错误信息
 *
 * 强制更新（`forceUpdate=true`）时：
 * - 不显示"稍后"按钮
 * - 禁用遮罩点击 / ESC 关闭
 *
 * See: docs/updater/design.md §4.2.2
 */
import { computed } from 'vue'
import {
  ArrowPathIcon,
  CheckCircleIcon,
  ExclamationTriangleIcon,
  XMarkIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import {
  updateState,
  checkForUpdate,
  downloadAndInstall,
  closeDialog,
} from '@/utils/updater'

/** 是否显示弹窗（仅当 showDialog=true 时渲染） */
const visible = computed(() => updateState.showDialog)

/** 是否允许关闭（强制更新 / 下载中 / 安装中 时禁止） */
const canClose = computed(
  () =>
    !updateState.forceUpdate &&
    updateState.status !== 'downloading' &&
    updateState.status !== 'installing',
)

/** 下载进度百分比（0-100），total 未知时返回 null（indeterminate） */
const progressPct = computed(() => {
  if (updateState.total <= 0) return null
  return Math.min(100, Math.round((updateState.downloaded / updateState.total) * 100))
})

/** 人类可读的下载大小（如 "12.3 MB / 45.6 MB"） */
const downloadedText = computed(() => formatBytes(updateState.downloaded))
const totalText = computed(() => (updateState.total > 0 ? formatBytes(updateState.total) : '未知'))

function formatBytes(bytes: number): string {
  if (bytes <= 0) return '0 B'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`
  return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`
}

/** 点击遮罩（仅 canClose 时生效） */
function onMaskClick() {
  if (canClose.value) closeDialog()
}

/** ESC 键关闭（仅 canClose 时生效） */
function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && canClose.value) closeDialog()
}

/** 立即更新 */
function onUpdate() {
  downloadAndInstall()
}

/** 稍后（仅非强制更新可用） */
function onLater() {
  if (!canClose.value) return
  closeDialog()
}

/** 重试（错误状态下重新检查） */
function onRetry() {
  checkForUpdate({ silent: false })
}
</script>

<template>
  <teleport to="body">
    <transition
      enter-active-class="transition ease-out duration-150"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition ease-in duration-100"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="visible"
        class="fixed inset-0 z-[10000] flex items-center justify-center p-4"
        tabindex="0"
        @click.self="onMaskClick"
        @keydown="onKeydown"
      >
        <div class="absolute inset-0 bg-black/40" />

        <div class="relative w-full max-w-md bg-white rounded-lg shadow-xl">
          <!-- 标题栏 -->
          <div class="flex items-center justify-between px-6 pt-5 pb-3">
            <div class="flex items-center gap-2.5">
              <ArrowPathIcon class="h-5 w-5 text-primary-500" />
              <h3 class="text-base font-semibold text-gray-900">发现新版本</h3>
            </div>
            <button
              v-if="canClose"
              class="text-gray-400 hover:text-gray-600 transition-colors"
              @click="closeDialog"
            >
              <XMarkIcon class="h-5 w-5" />
            </button>
          </div>

          <!-- 内容区 -->
          <div class="px-6 pb-2">
            <!-- 检查中 -->
            <div v-if="updateState.status === 'checking'" class="py-6 flex flex-col items-center gap-3">
              <div class="h-7 w-7 animate-spin rounded-full border-[3px] border-primary-200 border-t-primary-500" />
              <p class="text-sm text-gray-500">正在检查更新...</p>
            </div>

            <!-- 发现新版本 -->
            <div v-else-if="updateState.status === 'available'" class="space-y-3">
              <div class="flex items-center gap-2 text-sm">
                <span class="text-gray-500">最新版本：</span>
                <span class="font-semibold text-primary-600">v{{ updateState.version }}</span>
                <span
                  v-if="updateState.forceUpdate"
                  class="ml-1 rounded-full bg-red-50 px-2 py-0.5 text-[11px] font-medium text-red-600"
                >
                  强制更新
                </span>
              </div>
              <div v-if="updateState.notes" class="rounded-md bg-gray-50 p-3 max-h-44 overflow-y-auto">
                <p class="text-xs text-gray-600 leading-relaxed whitespace-pre-line">{{ updateState.notes }}</p>
              </div>
              <p v-if="updateState.forceUpdate" class="text-xs text-red-500">
                此版本为重要更新，需要立即安装后才能继续使用。
              </p>
            </div>

            <!-- 下载中 -->
            <div v-else-if="updateState.status === 'downloading'" class="space-y-3 py-2">
              <div class="flex items-center justify-between text-xs text-gray-500">
                <span>正在下载 v{{ updateState.version }}</span>
                <span>{{ downloadedText }} / {{ totalText }}</span>
              </div>
              <div class="h-2 w-full rounded-full bg-gray-100 overflow-hidden">
                <div
                  v-if="progressPct !== null"
                  class="h-full bg-primary-500 transition-all duration-150"
                  :style="{ width: `${progressPct}%` }"
                />
                <div
                  v-else
                  class="h-full bg-primary-500 animate-pulse"
                  style="width: 30%"
                />
              </div>
              <p v-if="progressPct !== null" class="text-right text-xs text-gray-400">{{ progressPct }}%</p>
            </div>

            <!-- 安装中 -->
            <div v-else-if="updateState.status === 'installing'" class="py-6 flex flex-col items-center gap-3">
              <div class="h-7 w-7 animate-spin rounded-full border-[3px] border-primary-200 border-t-primary-500" />
              <p class="text-sm text-gray-500">正在安装，应用即将重启...</p>
            </div>

            <!-- 错误 -->
            <div v-else-if="updateState.status === 'error'" class="py-4 space-y-3">
              <div class="flex items-start gap-2.5">
                <ExclamationTriangleIcon class="h-5 w-5 flex-none text-red-500" />
                <div class="min-w-0">
                  <p class="text-sm font-medium text-gray-900">更新失败</p>
                  <p class="mt-1 text-xs text-gray-500 break-all">{{ updateState.error }}</p>
                </div>
              </div>
            </div>

            <!-- 完成（理论上不会显示，relaunch 后进程退出） -->
            <div v-else-if="updateState.status === 'done'" class="py-6 flex flex-col items-center gap-3">
              <CheckCircleIcon class="h-8 w-8 text-green-500" />
              <p class="text-sm text-gray-500">更新完成，即将重启...</p>
            </div>
          </div>

          <!-- 按钮栏 -->
          <div class="flex justify-end gap-2 px-6 py-3.5 bg-gray-50 rounded-b-lg">
            <template v-if="updateState.status === 'available'">
              <Button
                v-if="canClose"
                type="ghost"
                size="small"
                @click="onLater"
              >
                稍后
              </Button>
              <Button
                type="primary"
                size="small"
                @click="onUpdate"
              >
                立即更新
              </Button>
            </template>

            <template v-else-if="updateState.status === 'error'">
              <Button
                v-if="canClose"
                type="ghost"
                size="small"
                @click="closeDialog"
              >
                关闭
              </Button>
              <Button
                type="primary"
                size="small"
                @click="onRetry"
              >
                重试
              </Button>
            </template>

            <!-- downloading / installing / checking / done / no-update 不显示按钮 -->
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>
