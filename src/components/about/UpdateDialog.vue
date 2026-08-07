<script setup lang="ts">
/**
 * 更新进度弹窗（Drawer 抽屉）
 *
 * 监听 `utils/updater.ts` 的全局响应式状态，自动显示/隐藏：
 * - `checking`：检查中 loading
 * - `available`：发现新版本，展示版本号 + 更新日志（时间线）+ 按钮（立即更新 / 稍后）
 * - `downloading`：下载进度条
 * - `installing`：安装中 loading（即将重启）
 * - `error`：错误信息
 *
 * 强制更新（`forceUpdate=true`）时：
 * - 不显示"稍后"按钮
 * - 禁用关闭按钮 / 遮罩点击 / ESC 关闭
 *
 * See: docs/updater/design.md §4.2.2
 */
import { computed } from 'vue'
import {
  ArrowPathIcon,
  CheckCircleIcon,
  ExclamationTriangleIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Tag from '@/components/common/Tag.vue'
import Drawer from '@/components/common/Drawer.vue'
import ReleaseTimeline from '@/components/about/ReleaseTimeline.vue'
import { onGlobalEvent } from '@/composables/useGlobalTauriEvent'
import { formatBytes } from '@/utils/format'
import {
  updateState,
  checkForUpdate,
  downloadAndInstall,
  closeDialog,
} from '@/utils/updater'

/** 是否显示弹窗（仅当 showDialog=true 时渲染） */
const visible = computed(() => updateState.showDialog)

/**
 * 监听 macOS/Linux 更新下载进度（Rust install_unix.rs 经 Tauri 事件推送）
 *
 * 仅在 downloading 状态写入真实 downloaded/total：
 * - total>0 时进度条显示真实百分比
 * - total=0 时保持 indeterminate 动画（后端尚未获得 Content-Length）
 */
onGlobalEvent<{ downloaded: number; total: number }>('update-download-progress', (payload) => {
  if (updateState.status !== 'downloading') return
  if (typeof payload?.downloaded === 'number') updateState.downloaded = payload.downloaded
  if (typeof payload?.total === 'number') updateState.total = payload.total
})

/** 是否允许关闭（强制更新 / 下载中 / 安装中 时禁止） */
const canClose = computed(
  () =>
    !updateState.forceUpdate &&
    updateState.status !== 'downloading' &&
    updateState.status !== 'installing',
)

/** Drawer 关闭请求（点遮罩 / ESC / 关闭按钮）：canClose 时同步关闭全局弹窗状态 */
function onVisibleChange(v: boolean) {
  if (!v && canClose.value) closeDialog()
}

/** 下载进度百分比（0-100），total 未知时返回 null（indeterminate） */
const progressPct = computed(() => {
  if (updateState.total <= 0) return null
  return Math.min(100, Math.round((updateState.downloaded / updateState.total) * 100))
})

/** 人类可读的下载大小（如 "12.3 MB / 45.6 MB"） */
const downloadedText = computed(() => formatBytes(updateState.downloaded))
const totalText = computed(() => (updateState.total > 0 ? formatBytes(updateState.total) : '未知'))

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
  <Drawer
    :visible="visible"
    placement="right"
    :width="560"
    render-in-place
    popup-container="#app-content"
    :closable="canClose"
    :mask-closable="canClose"
    :esc-to-close="canClose"
    @update:visible="onVisibleChange"
  >
    <template #title>
      <div class="flex items-center gap-2">
        <ArrowPathIcon class="h-5 w-5 text-primary-500" />
        <span>发现新版本</span>
      </div>
    </template>

    <!-- 发现新版本：最新版本行固定，仅下方日志区滚动 -->
    <div
      v-if="updateState.status === 'available'"
      class="flex h-full min-h-0 flex-col"
    >
      <div class="flex items-center gap-2 py-1 text-sm">
        <span class="text-gray-500">最新版本：</span>
        <span class="font-semibold text-primary-600">v{{ updateState.version }}</span>
        <span v-if="updateState.forceUpdate" class="ml-1">
          <Tag size="small" color="red">强制更新</Tag>
        </span>
      </div>
      <!-- 更新日志：时间线组件负责分段渲染；仅此区域独立滚动 -->
      <div
        v-if="updateState.notes"
        class="min-h-0 flex-1 overflow-y-auto rounded-md bg-gray-50 p-3"
      >
        <ReleaseTimeline :notes="updateState.notes" />
      </div>
      <p v-if="updateState.forceUpdate" class="pt-2 text-xs text-red-500">
        此版本为重要更新，需要立即安装后才能继续使用。
      </p>
    </div>

    <!-- 其他状态 -->
    <div v-else class="flex min-h-full flex-col justify-center gap-3 py-2">
      <!-- 检查中 -->
      <div v-if="updateState.status === 'checking'" class="flex flex-col items-center gap-3">
        <div class="h-7 w-7 animate-spin rounded-full border-[3px] border-primary-200 border-t-primary-500" />
        <p class="text-sm text-gray-500">正在检查更新...</p>
      </div>

      <!-- 下载中 -->
      <div v-else-if="updateState.status === 'downloading'" class="space-y-3">
        <div class="flex items-center justify-between text-xs text-gray-500">
          <span>正在下载 v{{ updateState.version }}</span>
          <span>{{ downloadedText }} / {{ totalText }}</span>
        </div>
        <div class="h-2 w-full overflow-hidden rounded-full bg-gray-100">
          <div
            v-if="progressPct !== null"
            class="h-full bg-primary-500 transition-all duration-150"
            :style="{ width: `${progressPct}%` }"
          />
          <div
            v-else
            class="h-full animate-pulse bg-primary-500"
            style="width: 30%"
          />
        </div>
        <p v-if="progressPct !== null" class="text-right text-xs text-gray-400">{{ progressPct }}%</p>
      </div>

      <!-- 安装中 -->
      <div v-else-if="updateState.status === 'installing'" class="flex flex-col items-center gap-3">
        <div class="h-7 w-7 animate-spin rounded-full border-[3px] border-primary-200 border-t-primary-500" />
        <p class="text-sm text-gray-500">正在安装，应用即将重启...</p>
      </div>

      <!-- 错误 -->
      <div v-else-if="updateState.status === 'error'" class="space-y-3">
        <div class="flex items-start gap-2.5">
          <ExclamationTriangleIcon class="h-5 w-5 flex-none text-red-500" />
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">更新失败</p>
            <p class="mt-1 break-all text-xs text-gray-500">{{ updateState.error }}</p>
          </div>
        </div>
      </div>

      <!-- 完成（理论上不会显示，relaunch 后进程退出） -->
      <div v-else-if="updateState.status === 'done'" class="flex flex-col items-center gap-3">
        <CheckCircleIcon class="h-8 w-8 text-green-500" />
        <p class="text-sm text-gray-500">更新完成，即将重启...</p>
      </div>
    </div>

    <!-- 底部按钮：available / error 状态显示，其余状态无按钮 -->
    <template #footer>
      <div
        v-if="updateState.status === 'available' || updateState.status === 'error'"
        class="flex justify-end gap-2"
      >
        <template v-if="updateState.status === 'available'">
          <Button v-if="canClose" type="ghost" size="small" @click="onLater">稍后</Button>
          <Button type="primary" size="small" @click="onUpdate">立即更新</Button>
        </template>
        <template v-else>
          <Button v-if="canClose" type="ghost" size="small" @click="closeDialog">关闭</Button>
          <Button type="primary" size="small" @click="onRetry">重试</Button>
        </template>
      </div>
    </template>
  </Drawer>
</template>
