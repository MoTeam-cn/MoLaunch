<script setup lang="ts">
/**
 * 导出选项列表组件
 *
 * 接收扁平的 ExportOption 数组（含父子关系），按 parent 字段分组渲染：
 * - 顶层选项：勾选框 + 标题 + 描述
 * - 子选项：缩进显示，勾选框 + 标题 + 描述
 * - 必选项（enabled=false）：禁用勾选框，强制勾选
 * - 不可见选项（visible=false）：不显示
 *
 * 与父组件 ExportTab 通过 v-model:checked 双向绑定，保持选项状态同步。
 */
import { computed } from 'vue'
import Checkbox from '@/components/common/Checkbox.vue'
import type { ExportOption } from '@/utils/api/version-export-manager'

interface Props {
  /** 所有选项（含子选项） */
  options: ExportOption[]
}

const props = defineProps<Props>()

const emit = defineEmits<{
  /** 切换某个选项的勾选状态 */
  toggle: [option: ExportOption]
}>()

/** 可见选项 */
const visibleOptions = computed(() => props.options.filter(o => o.visible))

/** 顶层选项（无 parent） */
const topLevelOptions = computed(() => visibleOptions.value.filter(o => !o.parent))

/** 获取指定父选项的可见子选项列表 */
function getChildren(parentId: string): ExportOption[] {
  return visibleOptions.value.filter(o => o.parent === parentId)
}
</script>

<template>
  <div v-if="topLevelOptions.length === 0" class="flex h-32 flex-col items-center justify-center gap-2 text-gray-400">
    <svg class="h-8 w-8" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path stroke-linecap="round" stroke-linejoin="round" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
    </svg>
    <p class="text-xs">没有可导出的内容</p>
  </div>

  <div v-else class="space-y-1">
    <div v-for="opt in topLevelOptions" :key="opt.id">
      <!-- 顶层选项 -->
      <div
        class="flex cursor-pointer items-start gap-3 rounded-md px-2 py-1.5 transition-colors"
        :class="opt.enabled ? 'hover:bg-gray-50' : 'cursor-not-allowed opacity-80'"
        @click.prevent="opt.enabled && emit('toggle', opt)"
      >
        <Checkbox
          :checked="opt.checked"
          :disabled="!opt.enabled"
          @click.stop.prevent="opt.enabled && emit('toggle', opt)"
        />
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-1.5">
            <span class="text-sm text-gray-800">{{ opt.title }}</span>
            <span
              v-if="!opt.enabled"
              class="rounded bg-gray-100 px-1.5 py-0.5 text-[10px] text-gray-500"
            >必选</span>
          </div>
          <div v-if="opt.description" class="mt-0.5 text-xs text-gray-400">{{ opt.description }}</div>
        </div>
      </div>

      <!-- 子选项 -->
      <div v-if="getChildren(opt.id).length > 0" class="ml-7 mt-0.5 space-y-0.5 border-l border-gray-100 pl-3">
        <div
          v-for="sub in getChildren(opt.id)"
          :key="sub.id"
          class="flex cursor-pointer items-start gap-3 rounded-md px-2 py-1 transition-colors hover:bg-gray-50"
          @click.prevent="sub.enabled && emit('toggle', sub)"
        >
          <Checkbox
            :checked="sub.checked"
            :disabled="!sub.enabled"
            @click.stop.prevent="sub.enabled && emit('toggle', sub)"
        />
          <div class="min-w-0 flex-1">
            <div class="text-sm text-gray-700">{{ sub.title }}</div>
            <div v-if="sub.description" class="mt-0.5 text-xs text-gray-400">{{ sub.description }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
