<script setup lang="ts">
/**
 * 前置 Mod 依赖项卡片
 *
 * 展示单个前置 mod 信息：复选框 + logo + 名称 + 平台 + 建议版本 + 递归深度标识。
 * 用于 DependencyConfirmDialog 列表。
 */
import type { ResolvedDependency } from '@/types/community'
import Tooltip from '@/components/common/Tooltip.vue'
import CachedImage from '@/components/common/CachedImage.vue'
import Tag from '@/components/common/Tag.vue'
import {
  CheckCircleIcon,
  ExclamationCircleIcon,
} from '@heroicons/vue/24/outline'

const props = defineProps<{
  dep: ResolvedDependency
  selected: boolean
}>()

const emit = defineEmits<{ toggle: [] }>()

const projectName = () => props.dep.project.translated_name || props.dep.project.raw_name
const platformLabel = () => (props.dep.project.platform === 'CurseForge' ? 'CF' : 'MR')
const depthLabel = () => {
  if (props.dep.depth === 0) return ''
  if (props.dep.depth === 1) return '前置的前置'
  return `${props.dep.depth} 层依赖`
}
const versionInfo = () => {
  const sv = props.dep.suggestedVersion
  if (!sv) return '无兼容版本'
  return sv.version || sv.display || sv.file_name
}
const logoUrl = () => props.dep.project.logo_url || ''
const hasCompatibleVersion = () => props.dep.suggestedVersion !== null
</script>

<template>
  <div
    class="flex items-center gap-3 px-3 py-2 rounded-md border transition-colors"
    :class="selected
      ? 'border-primary-200 bg-primary-50/50'
      : 'border-gray-200 bg-white hover:bg-gray-50'"
  >
    <!-- 复选框 -->
    <!-- 保留原生 button：自定义复选框（w-5 h-5 + border-2 + checked 状态），
         Button.vue 的 scoped size 类与布局不适合复选框场景 -->
    <button
      type="button"
      class="w-5 h-5 shrink-0 flex items-center justify-center rounded border-2 transition-colors"
      :class="selected
        ? 'bg-primary-500 border-primary-500'
        : 'bg-white border-gray-300 hover:border-primary-400'"
      :disabled="!hasCompatibleVersion()"
      @click="hasCompatibleVersion() && emit('toggle')"
    >
      <svg v-if="selected" class="w-3 h-3 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
        <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
      </svg>
    </button>

    <!-- logo -->
    <div class="w-8 h-8 shrink-0 rounded overflow-hidden bg-gray-100 flex items-center justify-center">
      <CachedImage :src="logoUrl()" :alt="projectName()" class="w-full h-full object-cover">
        <template #fallback>
          <svg class="w-5 h-5 text-gray-300" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
          </svg>
        </template>
      </CachedImage>
    </div>

    <!-- 名称 + 版本 -->
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-1.5">
        <span class="text-sm font-medium text-gray-900 truncate">{{ projectName() }}</span>
        <Tag size="small" color="gray" class="shrink-0">{{ platformLabel() }}</Tag>
        <Tooltip v-if="depthLabel()" :text="depthLabel()" position="top">
          <span class="text-[10px] px-1 py-0.5 rounded bg-amber-100 text-amber-600 shrink-0">{{ depthLabel() }}</span>
        </Tooltip>
      </div>
      <div class="text-xs text-gray-500 truncate">{{ versionInfo() }}</div>
    </div>

    <!-- 状态 -->
    <div class="shrink-0">
      <Tooltip v-if="!hasCompatibleVersion()" text="未找到兼容版本，请手动安装" position="left">
        <ExclamationCircleIcon class="w-4 h-4 text-amber-500" />
      </Tooltip>
      <CheckCircleIcon v-else class="w-4 h-4 text-primary-500" />
    </div>
  </div>
</template>
