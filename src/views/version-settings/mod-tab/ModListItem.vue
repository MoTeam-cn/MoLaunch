<script setup lang="ts">
/**
 * 单个 Mod 列表项（参考 PCL2 MyLocalModItem）
 * - 左侧 4px 状态色条（启用=primary/禁用=gray）
 * - 34×34 圆角真实 Logo 图标（从 jar 内提取，无 logo fallback 到默认图）
 * - 标题 14px + 副标题 12px 灰色（译名/文件名按 modLocalNameStyle 切换）
 * - 详情行：文件大小 · 加载器类型 · 文件名（hover Tooltip 显示完整路径）
 * - 五个操作按钮：详情、百科、打开文件位置、启用/禁用、删除
 * - 按钮默认 opacity-0 隐藏，hover 列表项时才显示（与 PCL2 ButtonStack 行为一致）
 */
import Tooltip from '@/components/common/Tooltip.vue'
import { formatBytes } from '@/utils/format'
import { modTitle, modSubtitle, loaderVisual } from '@/utils/mod-display'
// Mod 默认 logo（无 jar 内 logo 时使用，参考 PCL2 Icons/NoIcon.png）
import defaultModLogo from '@/assets/Mods/default-min.png'
import {
  PauseIcon,
  PlayIcon,
  TrashIcon,
  InformationCircleIcon,
  BookOpenIcon,
  FolderOpenIcon,
} from '@heroicons/vue/24/outline'
import type { ModInfo } from '@/utils/tauri'

defineProps<{
  mod: ModInfo
  detailLoadingFor: string | null
  modLocalNameStyle: number
}>()

defineEmits<{
  toggle: [mod: ModInfo]
  delete: [mod: ModInfo]
  'show-info': [mod: ModInfo]
  'open-wiki': [mod: ModInfo]
  'open-file': [mod: ModInfo]
}>()
</script>

<template>
  <li
    class="group relative flex items-center gap-3 px-3 py-2.5 transition-colors hover:bg-gray-50"
    :class="{ 'bg-gray-50/40': !mod.is_enabled }"
  >
    <!-- 左侧状态色条 -->
    <div
      class="absolute left-0 top-0 h-full w-1 transition-colors"
      :class="mod.is_enabled ? 'bg-primary-500' : 'bg-gray-300'"
    ></div>

    <!-- 图标：有 logo 用真实 logo，否则用默认图片（不再用字母色块） -->
    <div class="relative flex-none">
      <img
        :src="mod.logo_data || defaultModLogo"
        class="h-9 w-9 rounded-lg object-cover"
        :class="{ 'opacity-50 grayscale': !mod.is_enabled }"
        alt=""
        @error="(e) => { (e.target as HTMLImageElement).src = defaultModLogo }"
      >
      <!-- 禁用角标 -->
      <div
        v-if="!mod.is_enabled"
        class="absolute -bottom-1 -right-1 flex h-4 w-4 items-center justify-center rounded-full bg-gray-400 text-white shadow"
      >
        <PauseIcon class="h-2.5 w-2.5" />
      </div>
    </div>

    <!-- 信息区 -->
    <div class="min-w-0 flex-1">
      <div class="flex items-center gap-2">
        <span
          class="truncate text-sm font-medium"
          :class="mod.is_enabled ? 'text-gray-800' : 'text-gray-500 line-through decoration-gray-300'"
        >
          {{ modTitle(mod, modLocalNameStyle) }}
        </span>
        <span
          v-if="mod.version"
          class="flex-none rounded bg-gray-100 px-1.5 py-0.5 text-[10px] font-medium text-gray-500"
        >{{ mod.version }}</span>
        <span
          v-if="modSubtitle(mod, modLocalNameStyle) && modSubtitle(mod, modLocalNameStyle) !== modTitle(mod, modLocalNameStyle)"
          class="truncate text-xs text-gray-400"
        >{{ modSubtitle(mod, modLocalNameStyle) }}</span>
      </div>
      <div class="mt-0.5 flex items-center gap-1.5 text-xs text-gray-400">
        <span>{{ formatBytes(mod.size) }}</span>
        <span v-if="mod.loader_type !== 'unknown'">·</span>
        <span v-if="mod.loader_type !== 'unknown'">{{ loaderVisual(mod.loader_type).label }}</span>
        <span>·</span>
        <Tooltip :text="mod.file_name" position="top" :delay="200">
          <span class="cursor-help underline decoration-dotted underline-offset-2 hover:text-gray-600">
            {{ mod.file_name.length > 28 ? mod.file_name.slice(0, 25) + '...' : mod.file_name }}
          </span>
        </Tooltip>
      </div>
    </div>

    <!-- 操作区：五个按钮，默认隐藏，hover 时显示（参考 PCL2 ButtonStack opacity 动画） -->
    <!-- 加载中时强制显示，避免鼠标离开后 spinner 消失让用户以为没点到（防呆） -->
    <div
      class="flex flex-none items-center gap-1 transition-opacity duration-200"
      :class="detailLoadingFor === mod.file_name
        ? 'opacity-100'
        : 'opacity-0 group-hover:opacity-100'"
    >
      <Tooltip :text="detailLoadingFor === mod.file_name ? '正在加载详情...' : '查看详情'" position="top">
        <button
          class="rounded-md p-1.5 transition-colors disabled:cursor-wait"
          :class="detailLoadingFor === mod.file_name
            ? 'text-blue-500 bg-blue-50 cursor-wait'
            : 'text-gray-400 hover:bg-blue-50 hover:text-blue-600'"
          :disabled="detailLoadingFor === mod.file_name"
          @click="$emit('show-info', mod)"
        >
          <!-- 加载中：旋转 spinner，让用户明确感知按钮已响应（防呆） -->
          <svg v-if="detailLoadingFor === mod.file_name" class="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
            <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
          </svg>
          <InformationCircleIcon v-else class="h-4 w-4" />
        </button>
      </Tooltip>
      <Tooltip text="前往百科" position="top">
        <button
          class="rounded-md p-1.5 text-gray-400 transition-colors hover:bg-emerald-50 hover:text-emerald-600"
          @click="$emit('open-wiki', mod)"
        >
          <BookOpenIcon class="h-4 w-4" />
        </button>
      </Tooltip>
      <Tooltip text="打开文件位置" position="top">
        <button
          class="rounded-md p-1.5 text-gray-400 transition-colors hover:bg-gray-100 hover:text-gray-700"
          @click="$emit('open-file', mod)"
        >
          <FolderOpenIcon class="h-4 w-4" />
        </button>
      </Tooltip>
      <Tooltip :text="mod.is_enabled ? '禁用' : '启用'" position="top">
        <button
          class="rounded-md p-1.5 transition-colors"
          :class="mod.is_enabled
            ? 'text-gray-400 hover:bg-amber-50 hover:text-amber-600'
            : 'text-gray-400 hover:bg-green-50 hover:text-green-600'"
          @click="$emit('toggle', mod)"
        >
          <PauseIcon v-if="mod.is_enabled" class="h-4 w-4" />
          <PlayIcon v-else class="h-4 w-4" />
        </button>
      </Tooltip>
      <Tooltip text="删除" position="top">
        <button
          class="rounded-md p-1.5 text-gray-400 transition-colors hover:bg-red-50 hover:text-red-600"
          @click="$emit('delete', mod)"
        >
          <TrashIcon class="h-4 w-4" />
        </button>
      </Tooltip>
    </div>
  </li>
</template>
