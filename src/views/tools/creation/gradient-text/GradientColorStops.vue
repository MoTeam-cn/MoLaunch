<script setup lang="ts">
/**
 * 渐变文字生成器 - 颜色停靠点编辑器（增删/排序/随机）
 */
import { defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const ColorPicker = defineAsyncComponent(() => import('@/components/common/ColorPicker.vue'))
import { normalizeHexColor } from '@/utils/gradient-text'
import { ArrowDownIcon, ArrowUpIcon, PlusIcon, TrashIcon } from '@heroicons/vue/24/outline'

const props = defineProps<{
  colors: string[]
}>()

const emit = defineEmits<{
  'update:colors': [colors: string[]]
}>()

function addColor() {
  emit('update:colors', [...props.colors, '#165DFF'])
}

function removeColor(index: number) {
  if (props.colors.length <= 1) return
  const next = [...props.colors]
  next.splice(index, 1)
  emit('update:colors', next)
}

function moveColor(index: number, direction: -1 | 1) {
  const target = index + direction
  if (target < 0 || target >= props.colors.length) return
  const next = [...props.colors]
  ;[next[index], next[target]] = [next[target], next[index]]
  emit('update:colors', next)
}

function updateColor(index: number, value: string) {
  const next = [...props.colors]
  next[index] = value
  emit('update:colors', next)
}

function randomColor() {
  emit('update:colors', [
    ...props.colors,
    normalizeHexColor(
      `#${Math.floor(Math.random() * 0xffffff)
        .toString(16)
        .padStart(6, '0')}`,
    ) ?? '#165DFF',
  ])
}
</script>

<template>
  <div>
    <div class="mb-1.5 flex items-center justify-between">
      <label class="text-xs font-medium text-gray-700">颜色停靠点</label>
      <div class="flex items-center gap-1">
        <Button type="text" size="small" @click="randomColor">
          <template #icon><PlusIcon class="h-3.5 w-3.5" /></template>
          随机
        </Button>
        <Button type="outline" size="small" @click="addColor">
          <template #icon><PlusIcon class="h-3.5 w-3.5" /></template>
          添加
        </Button>
      </div>
    </div>
    <div class="space-y-2">
      <div
        v-for="(_, index) in colors"
        :key="index"
        class="flex items-center gap-2 rounded border border-gray-200 bg-gray-50 px-3 py-2"
      >
        <span class="w-6 flex-none text-xs text-gray-400">{{ index + 1 }}</span>
        <ColorPicker :model-value="colors[index]" @update:model-value="updateColor(index, $event)" />
        <span class="flex-1" />
        <Button type="text" size="small" :disabled="index === 0" @click="moveColor(index, -1)">
          <template #icon><ArrowUpIcon class="h-3.5 w-3.5" /></template>
        </Button>
        <Button
          type="text"
          size="small"
          :disabled="index === colors.length - 1"
          @click="moveColor(index, 1)"
        >
          <template #icon><ArrowDownIcon class="h-3.5 w-3.5" /></template>
        </Button>
        <Button type="text" size="small" :disabled="colors.length <= 1" @click="removeColor(index)">
          <template #icon><TrashIcon class="h-3.5 w-3.5" /></template>
        </Button>
      </div>
    </div>
  </div>
</template>