<script setup lang="ts">
/**
 * 投影工具栏
 *
 * 打开 / 轴与层 / 编辑工具（浏览·平移·画笔·矩形·球体·圆柱·替换）/ 笔尖与半径
 * / 撤销重做 / 旋转镜像 / 缩放 / 导出。纯展示组件，状态由页面持有。
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
import type { SchematicTool } from './types'

defineProps<{
  loading: boolean
  busy: boolean
  session: SchematicOpenResult | null
  axis: SliceAxisParam
  layer: number
  planeLayers: number
  tool: SchematicTool
  canUndo: boolean
  canRedo: boolean
  exportFormat: string
  nonAir: number
  brushSize: number
  radius: number
  cylinderHeight: number
}>()

const emit = defineEmits<{
  open: []
  'update:axis': [value: SliceAxisParam]
  'update:layer': [value: number]
  'update:tool': [value: SchematicTool]
  'update:exportFormat': [value: string]
  'update:brushSize': [value: number]
  'update:radius': [value: number]
  'update:cylinderHeight': [value: number]
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
  { label: '平移', value: 'pan' },
  { label: '画笔', value: 'brush' },
  { label: '矩形', value: 'rect' },
  { label: '球体', value: 'sphere' },
  { label: '圆柱', value: 'cylinder' },
  { label: '替换', value: 'replace' },
]
const formatOptions = [
  { label: 'Sponge', value: 'sponge_v2' },
  { label: 'Litematic', value: 'litematic' },
  { label: '结构文件', value: 'structure' },
]

function onNumber(event: Event, key: 'brushSize' | 'radius' | 'cylinderHeight'): void {
  const value = Number((event.target as HTMLInputElement).value)
  if (key === 'brushSize') emit('update:brushSize', value)
  else if (key === 'radius') emit('update:radius', value)
  else emit('update:cylinderHeight', value)
}
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
      <div class="flex min-w-[180px] flex-1 items-center gap-2">
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
      <Button size="small" type="outline" :disabled="!canUndo" @click="emit('undo')">
        <ArrowUturnLeftIcon class="h-4 w-4" />
      </Button>
      <Button size="small" type="outline" :disabled="!canRedo" @click="emit('redo')">
        <ArrowUturnRightIcon class="h-4 w-4" />
      </Button>
    </div>

    <div v-if="session" class="flex flex-wrap items-center gap-2">
      <SegmentedButtons
        :model-value="tool"
        :options="toolOptions"
        @update:model-value="emit('update:tool', $event as SchematicTool)"
      />
      <label v-if="tool === 'brush'" class="flex items-center gap-2 text-xs text-gray-600">
        笔尖
        <input
          :value="brushSize"
          type="range"
          min="1"
          max="15"
          class="w-24 accent-brand-2"
          @input="onNumber($event, 'brushSize')"
        />
        <span class="w-6 text-right">{{ brushSize }}</span>
      </label>
      <label v-if="tool === 'sphere' || tool === 'cylinder'" class="flex items-center gap-2 text-xs text-gray-600">
        半径
        <input
          :value="radius"
          type="range"
          min="1"
          max="32"
          class="w-24 accent-brand-2"
          @input="onNumber($event, 'radius')"
        />
        <span class="w-6 text-right">{{ radius }}</span>
      </label>
      <label v-if="tool === 'cylinder'" class="flex items-center gap-2 text-xs text-gray-600">
        高度
        <input
          :value="cylinderHeight"
          type="range"
          min="1"
          max="64"
          class="w-24 accent-brand-2"
          @input="onNumber($event, 'cylinderHeight')"
        />
        <span class="w-6 text-right">{{ cylinderHeight }}</span>
      </label>
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
      <span class="text-xs text-gray-600">视口</span>
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
      <Button size="small" type="outline" @click="emit('zoom', -1)">
        <MagnifyingGlassMinusIcon class="h-4 w-4" />
      </Button>
      <Button size="small" type="outline" @click="emit('zoom', 1)">
        <MagnifyingGlassPlusIcon class="h-4 w-4" />
      </Button>
    </div>
  </div>
</template>
