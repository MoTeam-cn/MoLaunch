<script setup lang="ts">
/**
 * 单个 Mod 列表项
 * - 34×34 圆角真实 Logo 图标（来自平台工程 logo_url，经 image_cache 缓存；无 logo fallback 到默认图）
 * - 标题 14px + 副标题 12px 灰色（译名/文件名按 modLocalNameStyle 切换）
 * - 详情行：文件大小 · 加载器类型 · 文件名（hover Tooltip 显示完整路径）
 * - 六个操作按钮：详情、百科、打开文件位置、启用/禁用、更新、删除
 * - 按钮默认 opacity-0 隐藏，hover 列表项时才显示
 *
 * 选中状态指示：
 * - 不使用复选框图标（太突兀），也不覆盖原有状态色条
 * - 在列表项左边缘外侧挂一条 5px 宽的彩色竖条（通过负 margin -3px 向左探出）
 * - 未选中：高度 0、透明（完全不可见，不影响原有状态色条）
 * - 选中：高度撑满（上下留 6px）、显示主题色，带弹性动画
 * - 选中时标题颜色变为主题强调色
 * - 原有的启用/禁用状态色条保持不变，两者互不干扰
 *
 * 多选交互：
 * - 点击列表项（非按钮区域）切换选中状态（非长按）
 * - Shift+点击 范围选择（由 useMultiSelect composable 处理）
 * - 按钮使用 @click.stop 避免触发选中
 */
import Tooltip from '@/components/common/Tooltip.vue'
import { formatBytes } from '@/utils/format'
import { modTitle, modSubtitle, loaderVisual } from '@/utils/mod-display'
// Mod 默认 logo（无 jar 内 logo 时使用）
import defaultModLogo from '@/assets/Mods/default-min.png'
import {
  PauseIcon,
  PlayIcon,
  TrashIcon,
  InformationCircleIcon,
  BookOpenIcon,
  FolderOpenIcon,
  ArrowPathIcon,
} from '@heroicons/vue/24/outline'
import type { ModInfo } from '@/utils/tauri'

const props = defineProps<{
  mod: ModInfo
  detailLoadingFor: string | null
  modLocalNameStyle: number
  /** 当前项是否被选中 */
  selected?: boolean
}>()

const emit = defineEmits<{
  toggle: [mod: ModInfo]
  delete: [mod: ModInfo]
  'show-info': [mod: ModInfo]
  'open-wiki': [mod: ModInfo]
  'open-file': [mod: ModInfo]
  /** 打开更新/更改版本对话框（仅关联了平台工程的 mod 显示此按钮） */
  'update': [mod: ModInfo]
  /** 点击列表项（非按钮区域），切换选中状态（带 shiftKey 用于范围选择） */
  'select': [mod: ModInfo, shiftKey: boolean]
}>()

/**
 * 点击列表项：切换选中状态
 *
 * 按钮使用 @click.stop 阻止冒泡，不会触发此 handler。
 */
function handleClick(e: MouseEvent) {
  emit('select', props.mod, e.shiftKey)
}
</script>

<template>
  <li
    class="group relative flex items-center gap-3 px-3 py-2.5 transition-colors cursor-default hover:bg-gray-50"
    :class="{ 'bg-gray-50/40': !mod.is_enabled && !selected }"
    @click="handleClick"
  >
    <!-- 原有启用/禁用状态色条（保持不变） -->
    <div
      class="absolute left-0 top-0 h-full w-1 transition-colors"
      :class="mod.is_enabled ? 'bg-primary-500' : 'bg-gray-300'"
    ></div>
    <!-- 选中指示条：
         独立于状态色条，挂在左边缘外侧（-left-1 探出 4px），选中时显示，带弹性动画 -->
    <div
      v-if="selected"
      class="absolute -left-1 top-1.5 bottom-1.5 w-[5px] rounded-full bg-blue-500 animate-check-pop z-10"
    ></div>

    <!-- 图标：使用平台工程 logo_url 经 image_cache 缓存后的 URL，无 logo 用默认图（不再用字母色块） -->
    <div class="relative flex-none">
      <img
        :src="mod.cached_logo_url || defaultModLogo"
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
          class="truncate text-[13px] font-semibold transition-colors"
          :class="selected
            ? 'text-blue-600'
            : (mod.is_enabled ? 'text-gray-800' : 'text-gray-500 line-through decoration-gray-300')"
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
      <div class="mt-1 flex items-center gap-2 text-[11px] text-gray-400">
        <span class="font-medium">{{ formatBytes(mod.size) }}</span>
        <span v-if="mod.loader_type !== 'unknown'" class="text-gray-300">|</span>
        <span v-if="mod.loader_type !== 'unknown'" class="font-medium">{{ loaderVisual(mod.loader_type).label }}</span>
        <span class="text-gray-300">|</span>
        <Tooltip :text="mod.file_name" position="top" :delay="200">
          <span class="cursor-help underline decoration-dotted underline-offset-2 hover:text-gray-600">
            {{ mod.file_name.length > 32 ? mod.file_name.slice(0, 29) + '...' : mod.file_name }}
          </span>
        </Tooltip>
      </div>
    </div>

    <!-- 操作区：六个按钮，默认隐藏，hover 时显示 -->
    <!-- 选中时也显示按钮，按钮的 @click.stop 会阻止触发选中 -->
    <div
      class="flex flex-none items-center gap-1 transition-opacity duration-200"
      :class="detailLoadingFor === mod.file_name
        ? 'opacity-100'
        : 'opacity-0 group-hover:opacity-100'"
    >
      <!-- 保留原生 button：纯图标工具栏按钮（@heroicons h-4 w-4 + p-1.5），每个按钮带不同
           hover 配色（蓝/绿/灰/琥珀/红），Button.vue 的 type 不支持自定义 hover 颜色；
           且最小 mini 尺寸 padding 0 11px 会使图标按钮过宽，需 @click.stop 阻止冒泡到 li 选中 -->
      <Tooltip :text="detailLoadingFor === mod.file_name ? '正在加载详情...' : '查看详情'" position="top">
        <button
          class="rounded-md p-1.5 transition-colors disabled:cursor-wait"
          :class="detailLoadingFor === mod.file_name
            ? 'text-blue-500 bg-blue-50 cursor-wait'
            : 'text-gray-400 hover:bg-blue-50 hover:text-blue-600'"
          :disabled="detailLoadingFor === mod.file_name"
          @click.stop="$emit('show-info', mod)"
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
          @click.stop="$emit('open-wiki', mod)"
        >
          <BookOpenIcon class="h-4 w-4" />
        </button>
      </Tooltip>
      <Tooltip text="打开文件位置" position="top">
        <button
          class="rounded-md p-1.5 text-gray-400 transition-colors hover:bg-gray-100 hover:text-gray-700"
          @click.stop="$emit('open-file', mod)"
        >
          <FolderOpenIcon class="h-4 w-4" />
        </button>
      </Tooltip>
      <Tooltip text="启用/禁用" position="top">
        <button
          class="rounded-md p-1.5 transition-colors"
          :class="mod.is_enabled
            ? 'text-gray-400 hover:bg-amber-50 hover:text-amber-600'
            : 'text-gray-400 hover:bg-green-50 hover:text-green-600'"
          @click.stop="$emit('toggle', mod)"
        >
          <PauseIcon v-if="mod.is_enabled" class="h-4 w-4" />
          <PlayIcon v-else class="h-4 w-4" />
        </button>
      </Tooltip>
      <Tooltip v-if="mod.project" text="更新 / 更改版本" position="top">
        <button
          class="rounded-md p-1.5 text-gray-400 transition-colors hover:bg-blue-50 hover:text-blue-600"
          @click.stop="$emit('update', mod)"
        >
          <ArrowPathIcon class="h-4 w-4" />
        </button>
      </Tooltip>
      <Tooltip text="删除" position="top">
        <button
          class="rounded-md p-1.5 text-gray-400 transition-colors hover:bg-red-50 hover:text-red-600"
          @click.stop="$emit('delete', mod)"
        >
          <TrashIcon class="h-4 w-4" />
        </button>
      </Tooltip>
    </div>
  </li>
</template>

<style scoped>
/* 选中指示条弹性动画：先冲过头再回弹
   使用 transform scaleY 而非 height，避免 height auto 无法动画的问题 */
.animate-check-pop {
  animation: checkPop 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  transform-origin: center;
}

@keyframes checkPop {
  0% {
    transform: scaleY(0);
    opacity: 0;
  }
  60% {
    transform: scaleY(1.15);
    opacity: 1;
  }
  100% {
    transform: scaleY(1);
    opacity: 1;
  }
}
</style>
