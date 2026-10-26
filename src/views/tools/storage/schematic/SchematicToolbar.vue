<script setup lang="ts">
/**
 * 投影工具栏
 *
 * 打开文件 / 轴与层切换 / 编辑工具 / 撤销重做 / 旋转镜像 / 缩放 / 导出。
 * 纯展示组件，状态由页面持有。
 */

import {
  ArrowDownTrayIcon,
  ArrowPathRoundedSquareIcon,
  ArrowUturnLeftIcon,
  ArrowUturnRightIcon,
  ChevronDownIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  ChevronUpIcon,
  FolderOpenIcon,
  MagnifyingGlassMinusIcon,
  MagnifyingGlassPlusIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import SegmentedButtons from '@/components/common/SegmentedButtons.vue'
import type { SchematicOpenResult, SliceAxisParam } from '@/utils/api/tools/schematic'

defineProps<{
  loading: boolean
  busy: boolean
  session: SchematicOpenResult | null
  axis: SliceAxisParam
  layer: number
  planeLayers: number
  tool: 'browse' | 'fill' | 'replace'
  canUndo: boolean
  canRedo: boolean
  exportFormat: string
  nonAir: number
}>()

const emit = defineEmits<{
  open: []
  'update:axis': [value: SliceAxisParam]
  'update:layer': [value: number]
  'update:tool': [value: 'browse' | 'fill' | 'replace']
  'update:exportFormat': [value: string]
  undo: []
  redo: []
  transform: [kind: 'rotate_cw' | 'rotate_ccw' | 'mirror_x' | 'mirror_z' | 'flip_y']
  zoom: [delta: number]
  pan: [dx: number, dy: number]
  export: []
}>()

const axisOptions = [
  { label: 'Y 轴', value: 'y' },
  { label: 'X 轴', value: 'x' },
  { label: 'Z 轴', value: 'z' },
]
const toolOptions = [
  { label: '浏览', value: 'browse' },
  { label: '填充', value: 'fill' },
  { label: '替换', value: 'replace' },
]
const formatOptions = [
  { label: 'Sponge', value: 'sponge_v2' },
  { label: 'Litematic', value: 'litematic' },
  { label: '结构文件', value: 'structure' },
]
</script>

<template>
  <div class="flex flex-col gap-3">
    <div class="flex flex-wrap items-center gap-2">
      <Button :loading="loading" @click="emit('open')">
        <FolderOpenIcon class="h-4 w-4" />
        打开投影
      </Button>
      <template v-if="session">
        <span class="text-xs text-gray-500">
          {{ session.name }} · {{ session.format }} ·
          {{ session.size[0] }}×{{ session.size[1] }}×{{ session.size[2] }} ·
          {{ nonAir.toLocaleString() }} 方块
        </span>
        <span class="mx-1 h-5 w-px bg-gray-200" />
        <SegmentedButtons
          :model-value="exportFormat"
          :options="formatOptions"
          @update:model-value="emit('update:exportFormat', String($event))"
        />
        <Button type="outline" :loading="busy" @click="emit('export')">
          <ArrowDownTrayIcon class="h-4 w-4" />
          导出
        </Button>
      </template>
    </div>

    <div v-if="session" class="flex flex-wrap items-center gap-3">
      <SegmentedButtons
        :model-value="axis"
        :options="axisOptions"
        @update:model-value="emit('update:axis', $event as SliceAxisParam)"
      />
      <div class="flex min-w-[200px] flex-1 items-center gap-2">
        <span class="text-xs text-gray-600">第 {{ layer + 1 }} / {{ planeLayers }} 层</span>
        <input
          :value="layer"
          type="range"
          min="0"
          :max="Math.max(planeLayers - 1, 0)"
          class="flex-1 accent-brand-2"
          @input="emit('update:layer', Number(($event.target as HTMLInputElement).value))"
        />
      </div>
      <SegmentedButtons
        :model-value="tool"
        :options="toolOptions"
        @update:model-value="emit('update:tool', $event as 'browse' | 'fill' | 'replace')"
      />
      <Button size="small" type="outline" :disabled="!canUndo" @click="emit('undo')">
        <ArrowUturnLeftIcon class="h-4 w-4" />
      </Button>
      <Button size="small" type="outline" :disabled="!canRedo" @click="emit('redo')">
        <ArrowUturnRightIcon class="h-4 w-4" />
      </Button>
    </div>

    <div v-if="session" class="flex flex-wrap items-center gap-2">
      <span class="text-xs text-gray-600">变换</span>
      <Button size="small" type="outline" :loading="busy" @click="emit('transform', 'rotate_cw')">
        <ArrowPathRoundedSquareIcon class="h-4 w-4" />
        顺时针
      </Button>
      <Button size="small" type="outline" :loading="busy" @click="emit('transform', 'rotate_ccw')">
        逆时针
      </Button>
      <Button size="small" type="outline" :loading="busy" @click="emit('transform', 'mirror_x')">
        镜像 X
      </Button>
      <Button size="small" type="outline" :loading="busy" @click="emit('transform', 'mirror_z')">
        镜像 Z
      </Button>
      <Button size="small" type="outline" :loading="busy" @click="emit('transform', 'flip_y')">
        上下翻转
      </Button>
      <span class="mx-1 h-5 w-px bg-gray-200" />
      <span class="text-xs text-gray-600">平移</span>
      <Button size="small" type="outline" @click="emit('pan', 0, -1)">
        <ChevronUpIcon class="h-4 w-4" />
      </Button>
      <Button size="small" type="outline" @click="emit('pan', 0, 1)">
        <ChevronDownIcon class="h-4 w-4" />
      </Button>
      <Button size="small" type="outline" @click="emit('pan', -1, 0)">
        <ChevronLeftIcon class="h-4 w-4" />
      </Button>
      <Button size="small" type="outline" @click="emit('pan', 1, 0)">
        <ChevronRightIcon class="h-4 w-4" />
      </Button>
      <span class="mx-1 h-5 w-px bg-gray-200" />
      <Button size="small" type="outline" @click="emit('zoom', -1)">
        <MagnifyingGlassMinusIcon class="h-4 w-4" />
      </Button>
      <Button size="small" type="outline" @click="emit('zoom', 1)">
        <MagnifyingGlassPlusIcon class="h-4 w-4" />
      </Button>
    </div>
  </div>
</template>
