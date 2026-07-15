<script setup lang="ts">
/**
 * 分页组件（参考 PCL2 CardPages）
 * 仅 3 个按钮：首页 / 上一页 / 下一页 + 当前页码
 * 无末页、无跳页输入
 */

import {
  ChevronDoubleLeftIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
} from '@heroicons/vue/24/outline'

interface Props {
  page: number
  total: number
  pageSize: number
}

const props = defineProps<Props>()
const emit = defineEmits<{ change: [page: number] }>()

/** 是否有上一页/下一页 */
function hasPrev() { return props.page > 0 }
function hasNext() { return (props.page + 1) * props.pageSize < props.total }
</script>

<template>
  <div
    v-if="total > pageSize"
    class="flex items-center justify-center gap-2 py-4"
  >
    <!-- 首页 -->
    <button
      class="p-1.5 rounded-md transition-colors"
      :class="hasPrev() ? 'text-gray-500 hover:bg-gray-100' : 'text-gray-300 cursor-not-allowed'"
      :disabled="!hasPrev()"
      @click="hasPrev() && emit('change', 0)"
    >
      <ChevronDoubleLeftIcon class="w-4 h-4" />
    </button>

    <!-- 上一页 -->
    <button
      class="p-1.5 rounded-md transition-colors"
      :class="hasPrev() ? 'text-gray-500 hover:bg-gray-100' : 'text-gray-300 cursor-not-allowed'"
      :disabled="!hasPrev()"
      @click="hasPrev() && emit('change', page - 1)"
    >
      <ChevronLeftIcon class="w-4 h-4" />
    </button>

    <!-- 页码 -->
    <span class="px-3 text-sm text-gray-600 font-medium">{{ page + 1 }}</span>

    <!-- 下一页 -->
    <button
      class="p-1.5 rounded-md transition-colors"
      :class="hasNext() ? 'text-gray-500 hover:bg-gray-100' : 'text-gray-300 cursor-not-allowed'"
      :disabled="!hasNext()"
      @click="hasNext() && emit('change', page + 1)"
    >
      <ChevronRightIcon class="w-4 h-4" />
    </button>
  </div>
</template>
