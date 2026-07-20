<script setup lang="ts">
/**
 * 分页组件
 * 仅 3 个按钮：首页 / 上一页 / 下一页 + 当前页码
 * 无末页、无跳页输入
 */

import {
  ChevronDoubleLeftIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'

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
    <Button
      type="ghost"
      size="mini"
      :disabled="!hasPrev()"
      @click="hasPrev() && emit('change', 0)"
    >
      <template #icon><ChevronDoubleLeftIcon class="w-4 h-4" /></template>
    </Button>

    <!-- 上一页 -->
    <Button
      type="ghost"
      size="mini"
      :disabled="!hasPrev()"
      @click="hasPrev() && emit('change', page - 1)"
    >
      <template #icon><ChevronLeftIcon class="w-4 h-4" /></template>
    </Button>

    <!-- 页码 -->
    <span class="px-3 text-sm text-gray-600 font-medium">{{ page + 1 }}</span>

    <!-- 下一页 -->
    <Button
      type="ghost"
      size="mini"
      :disabled="!hasNext()"
      @click="hasNext() && emit('change', page + 1)"
    >
      <template #icon><ChevronRightIcon class="w-4 h-4" /></template>
    </Button>
  </div>
</template>
