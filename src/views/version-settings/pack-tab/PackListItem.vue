<script setup lang="ts">
/**
 * 单个资源包/光影列表项
 * - 36×36 圆角图标（平台 logo 优先，其次包内 pack.png 经 get_pack_icon 提取，无则保底图）
 * - 名称 + 类型标签（zip/文件夹）+ 大小 + 文件名（hover Tooltip）
 * - 操作：详情、打开文件位置、启用/禁用、更新（有 project 时）、删除（hover 时显示）
 */
import { computed, defineAsyncComponent } from 'vue'
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
import { formatBytes } from '@/utils/format'
import { usePackIcon } from '@/composables/usePackIcon'
import { defaultAsset } from '@/utils/assets'
import {
  PauseIcon,
  PlayIcon,
  TrashIcon,
  FolderOpenIcon,
  InformationCircleIcon,
  ArrowPathIcon,
} from '@heroicons/vue/24/outline'
import type { PackInfo, PackKind } from '@/utils/tauri'

const props = defineProps<{
  pack: PackInfo
  selectedId: string | null
  kind: PackKind
  /** 正在加载详情的包 file_name（详情按钮 spinner + 防重复点击） */
  detailLoadingFor?: string | null
}>()

defineEmits<{
  toggle: [pack: PackInfo]
  delete: [pack: PackInfo]
  'open-file': [pack: PackInfo]
  'show-info': [pack: PackInfo]
  'update': [pack: PackInfo]
}>()

const selectedIdRef = computed(() => props.selectedId)
const kindRef = computed(() => props.kind)
const fileNameRef = computed(() => props.pack.file_name)
const { iconUrl } = usePackIcon(selectedIdRef, kindRef, fileNameRef)

/** 图标优先级：平台 logo（预加载填充）→ 包内图标 → 保底图 */
const displayUrl = computed(() => props.pack.cached_logo_url || iconUrl.value || defaultAsset(true))
</script>

<template>
  <li class="group relative flex items-center gap-3 px-4 py-3 transition-colors hover:bg-gray-50">
    <!-- 启用/禁用状态色条 -->
    <div
      class="absolute left-0 top-0 h-full w-1 transition-colors"
      :class="pack.is_enabled ? 'bg-primary-500' : 'bg-gray-300'"
    ></div>

    <!-- 图标 -->
    <div class="relative flex-none">
      <img
        :src="displayUrl"
        class="h-9 w-9 rounded-lg object-cover"
        :class="{ 'opacity-50 grayscale': !pack.is_enabled }"
        alt=""
        @error="(e) => { (e.target as HTMLImageElement).src = defaultAsset(true) }"
      >
      <!-- 禁用角标 -->
      <div
        v-if="!pack.is_enabled"
        class="absolute -bottom-1 -right-1 flex h-4 w-4 items-center justify-center rounded-full bg-gray-400 text-white shadow"
      >
        <PauseIcon class="h-2.5 w-2.5" />
      </div>
    </div>

    <!-- 信息区 -->
    <div class="min-w-0 flex-1">
      <div class="flex items-center gap-2">
        <span
          class="truncate text-[13px] font-semibold transition-colors"
          :class="pack.is_enabled ? 'text-gray-800' : 'text-gray-500 line-through decoration-gray-300'"
        >
          {{ pack.enabled_name }}
        </span>
        <Tag size="small" color="gray" class="flex-none">{{ pack.is_folder ? '文件夹' : 'zip' }}</Tag>
        <span
          v-if="pack.project"
          class="truncate text-xs text-gray-400"
        >{{ pack.project.raw_name }}</span>
      </div>
      <div class="mt-1 flex items-center gap-2 text-[11px] text-gray-400">
        <span class="font-medium">{{ formatBytes(pack.size) }}</span>
        <span class="text-gray-300">|</span>
        <Tooltip :text="pack.file_name" position="top" :delay="200">
          <span class="cursor-help underline decoration-dotted underline-offset-2 hover:text-gray-600">
            {{ pack.file_name.length > 32 ? pack.file_name.slice(0, 29) + '...' : pack.file_name }}
          </span>
        </Tooltip>
      </div>
    </div>

    <!-- 操作区：默认隐藏，hover 时显示 -->
    <div
      class="flex flex-none items-center gap-1 transition-opacity duration-200"
      :class="detailLoadingFor === pack.file_name
        ? 'opacity-100'
        : 'opacity-0 group-hover:opacity-100'"
    >
      <Tooltip :text="detailLoadingFor === pack.file_name ? '正在加载详情...' : '查看详情'" position="top">
        <button
          class="rounded-md p-1.5 transition-colors disabled:cursor-wait"
          :class="detailLoadingFor === pack.file_name
            ? 'text-blue-500 bg-blue-50 cursor-wait'
            : 'text-gray-400 hover:bg-blue-50 hover:text-blue-600'"
          :disabled="detailLoadingFor === pack.file_name"
          @click.stop="$emit('show-info', pack)"
        >
          <svg v-if="detailLoadingFor === pack.file_name" class="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
            <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
          </svg>
          <InformationCircleIcon v-else class="h-4 w-4" />
        </button>
      </Tooltip>
      <Tooltip text="打开文件位置" position="top">
        <button
          class="rounded-md p-1.5 text-gray-400 transition-colors hover:bg-gray-100 hover:text-gray-700"
          @click.stop="$emit('open-file', pack)"
        >
          <FolderOpenIcon class="h-4 w-4" />
        </button>
      </Tooltip>
      <Tooltip text="启用/禁用" position="top">
        <button
          class="rounded-md p-1.5 transition-colors"
          :class="pack.is_enabled
            ? 'text-gray-400 hover:bg-amber-50 hover:text-amber-600'
            : 'text-gray-400 hover:bg-green-50 hover:text-green-600'"
          @click.stop="$emit('toggle', pack)"
        >
          <PauseIcon v-if="pack.is_enabled" class="h-4 w-4" />
          <PlayIcon v-else class="h-4 w-4" />
        </button>
      </Tooltip>
      <Tooltip v-if="pack.project" text="更新 / 更改版本" position="top">
        <button
          class="rounded-md p-1.5 text-gray-400 transition-colors hover:bg-blue-50 hover:text-blue-600"
          @click.stop="$emit('update', pack)"
        >
          <ArrowPathIcon class="h-4 w-4" />
        </button>
      </Tooltip>
      <Tooltip text="删除" position="top">
        <button
          class="rounded-md p-1.5 text-gray-400 transition-colors hover:bg-red-50 hover:text-red-600"
          @click.stop="$emit('delete', pack)"
        >
          <TrashIcon class="h-4 w-4" />
        </button>
      </Tooltip>
    </div>
  </li>
</template>
