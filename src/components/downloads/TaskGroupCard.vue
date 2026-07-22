<script setup lang="ts">
/**
 * 下载任务卡片
 *
 * 包含：
 * - 卡片头部：版本名、总进度百分比、暂停/恢复/取消按钮
 * - 卡片主体：按 group 聚合的任务分组列表，可折叠展开查看子阶段
 *
 * 抽取自 Downloads.vue，任务列表卡片。
 * 分组聚合与折叠状态由 useDownloadTaskGroups composable 管理。
 */
import { formatBytes } from '@/utils/format'
import Tooltip from '@/components/common/Tooltip.vue'
import Button from '@/components/common/Button.vue'
import {
  CheckCircleIcon,
  ClockIcon,
  ChevronDownIcon,
  PauseCircleIcon,
  PlayCircleIcon,
  XCircleIcon,
} from '@heroicons/vue/24/outline'
import { useDownloadTaskGroups } from '@/composables/useDownloadTaskGroups'
import type { DownloadStage } from '@/types/download'

const props = defineProps<{
  /** 当前下载的版本名（来自 versionStore.downloadingVersion） */
  versionName: string | null
  /** 总进度百分比（0-100） */
  percentage: number
  /** 是否已暂停 */
  isPaused: boolean
  /** 暂停/恢复按钮是否在请求中 */
  togglingPause: boolean
  /** 取消按钮是否在请求中 */
  cancelling: boolean
  /** 下载阶段列表（来自 downloadProgress.stages） */
  stages: DownloadStage[]
}>()

const emit = defineEmits<{
  /** 点击暂停/恢复按钮 */
  togglePause: []
  /** 点击取消按钮（父组件负责弹确认框） */
  cancel: []
}>()

const { taskGroups, toggleGroup, isExpanded } = useDownloadTaskGroups(() => props.stages)
</script>

<template>
  <div class="bg-white rounded-xl border border-gray-200 overflow-hidden">
    <!-- 卡片头部 -->
    <div class="px-4 py-3 border-b border-gray-100 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <PauseCircleIcon v-if="isPaused" class="w-4 h-4 text-amber-500" />
        <span class="text-sm font-medium text-gray-900">
          {{ versionName }}
        </span>
        <span v-if="isPaused" class="text-xs text-amber-600 font-medium">已暂停</span>
      </div>
      <div class="flex items-center gap-3">
        <span class="text-xs text-gray-500">
          {{ percentage.toFixed(1) }}%
        </span>
        <!-- 暂停/恢复按钮 -->
        <Tooltip :text="isPaused ? '恢复下载' : '暂停下载'" position="top">
          <Button
            type="text"
            size="mini"
            :disabled="togglingPause"
            :class="isPaused
              ? '!text-green-600 hover:!bg-green-50'
              : '!text-amber-600 hover:!bg-amber-50'"
            @click="emit('togglePause')"
          >
            <template #icon>
              <component
                :is="isPaused ? PlayCircleIcon : PauseCircleIcon"
                class="w-4 h-4"
                :class="{ 'animate-pulse': togglingPause }"
              />
            </template>
            {{ isPaused ? '恢复' : '暂停' }}
          </Button>
        </Tooltip>
        <!-- 取消按钮 -->
        <Tooltip text="取消下载" position="top">
          <Button
            type="text"
            size="mini"
            :disabled="cancelling"
            class="!text-red-600 hover:!bg-red-50"
            @click="emit('cancel')"
          >
            <template #icon>
              <XCircleIcon class="w-4 h-4" :class="{ 'animate-pulse': cancelling }" />
            </template>
            取消
          </Button>
        </Tooltip>
      </div>
    </div>

    <!-- 任务分组列表（按 group 分组，可折叠展开看子阶段） -->
    <div class="divide-y divide-gray-100">
      <div v-for="g in taskGroups" :key="g.key" class="px-4 py-2">
        <!-- 分组标题栏（点击折叠/展开）— 结构性可点击行，非视觉按钮 -->
        <div
          role="button"
          tabindex="0"
          class="w-full flex items-center gap-2 py-1"
          :class="g.isIndependent ? 'cursor-default' : 'hover:bg-gray-50 rounded -mx-1 px-1 cursor-pointer'"
          @click="g.isIndependent ? undefined : toggleGroup(g.key)"
          @keydown.enter="g.isIndependent ? undefined : toggleGroup(g.key)"
          @keydown.space.prevent="g.isIndependent ? undefined : toggleGroup(g.key)"
        >
          <!-- 状态图标 -->
          <div class="w-5 h-5 flex items-center justify-center shrink-0">
            <CheckCircleIcon v-if="g.status === 'finished'" class="w-5 h-5 text-green-500" />
            <div v-else-if="g.status === 'loading' && !isPaused" class="w-3.5 h-3.5 rounded-full border border-gray-200 border-t-primary-500 animate-spin" />
            <PauseCircleIcon v-else-if="g.status === 'loading' && isPaused" class="w-5 h-5 text-amber-500" />
            <div v-else-if="g.status === 'failed'" class="w-5 h-5 rounded-full bg-red-500" />
            <ClockIcon v-else class="w-5 h-5 text-gray-300" />
          </div>
          <!-- 分组标题 -->
          <span
            class="text-sm flex-1 text-left"
            :class="{
              'text-gray-900 font-medium': g.status === 'loading',
              'text-gray-700': g.status === 'finished',
              'text-gray-400': g.status === 'waiting',
              'text-red-600': g.status === 'failed',
            }"
          >
            {{ g.title }}
          </span>
          <!-- 子阶段计数 + 分组字节 -->
          <span v-if="!g.isIndependent" class="text-[10px] text-gray-400">
            {{ g.stages.length }} 项
          </span>
          <span
            v-if="g.bytesTotal > 0 && (g.status === 'loading' || g.status === 'finished')"
            class="text-[10px] text-gray-400"
          >
            {{ formatBytes(g.bytesDownloaded) }} / {{ formatBytes(g.bytesTotal) }}
          </span>
          <!-- 进度 -->
          <span
            v-if="g.status === 'loading'"
            class="text-xs text-primary-600 font-medium"
          >
            {{ Math.round(g.progress * 100) }}%
          </span>
          <span v-else-if="g.status === 'finished'" class="text-xs text-green-600 font-medium">100%</span>
          <!-- 展开箭头（只有非独立分组才有） -->
          <ChevronDownIcon
            v-if="!g.isIndependent"
            class="w-4 h-4 text-gray-400 transition-transform duration-200"
            :class="isExpanded(g.key) ? 'rotate-180' : 'rotate-0'"
          />
        </div>

        <!-- 子阶段列表（独立分组只显示自身，不展开子列表） -->
        <div
          v-if="!g.isIndependent && isExpanded(g.key)"
          class="mt-1 ml-7 space-y-1 transition-all duration-200"
        >
          <div
            v-for="(s, idx) in g.stages"
            :key="idx"
            class="flex items-center gap-2 py-1"
          >
            <!-- 子状态图标 -->
            <div class="w-4 h-4 flex items-center justify-center shrink-0">
              <CheckCircleIcon v-if="s.status === 'finished'" class="w-4 h-4 text-green-400" />
              <div v-else-if="s.status === 'loading' && !isPaused" class="w-3 h-3 rounded-full border border-gray-200 border-t-primary-400 animate-spin" />
              <PauseCircleIcon v-else-if="s.status === 'loading' && isPaused" class="w-4 h-4 text-amber-400" />
              <ClockIcon v-else class="w-4 h-4 text-gray-300" />
            </div>
            <span
              class="text-sm flex-1"
              :class="{
                'text-gray-900 font-medium': s.status === 'loading',
                'text-gray-600': s.status === 'finished',
                'text-gray-400': s.status === 'waiting',
              }"
            >
              {{ s.name }}
            </span>
            <!-- 字节/文件进度 -->
            <span v-if="s.status === 'loading' && s.bytes_total > 0" class="text-xs text-gray-400">
              {{ formatBytes(s.bytes_downloaded) }} / {{ formatBytes(s.bytes_total) }}
            </span>
            <span v-else-if="s.status === 'loading' && s.files_total > 0" class="text-xs text-gray-400">
              {{ s.files_downloaded }} / {{ s.files_total }} 文件
            </span>
            <span v-if="s.status === 'loading'" class="text-xs text-primary-600 font-medium">
              {{ Math.round(s.progress * 100) }}%
            </span>
            <span v-else-if="s.status === 'finished'" class="text-xs text-green-600">100%</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
