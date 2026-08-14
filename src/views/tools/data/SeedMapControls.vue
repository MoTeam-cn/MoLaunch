<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
/**
 * 种子地图地图叠加控件：悬浮坐标/群系名、结构提示、坐标输入面板、缩放按钮、加载遮罩。
 * 从 SeedMap.vue 拆出，避免 Vue 组件超 300 行。
 */
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import { formatCoord } from '@/utils/seedmap/format'
import { getStructIcon } from '@/utils/seedmap/constants'
import type { WorkerStructure } from '@/utils/seedmap/types'
import {
  Squares2X2Icon, MapPinIcon, PlusIcon, MinusIcon, AdjustmentsHorizontalIcon,
} from '@heroicons/vue/24/outline'

const userX = defineModel<string>('userX', { required: true })
const userZ = defineModel<string>('userZ', { required: true })
const yCoord = defineModel<number>('yCoord', { required: true })
const doContour = defineModel<boolean>('doContour', { required: true })
const ymaxLimit = defineModel<number>('ymaxLimit', { required: true })
const largeBiomes = defineModel<boolean>('largeBiomes', { required: true })
const showCoordPanel = defineModel<boolean>('showCoordPanel', { required: true })

defineProps<{
  mouseBlock: { x: number; z: number } | null
  mouseBiomeName: string
  hoverStruct: WorkerStructure | null
  hoverMarker: { label: string; x: number; z: number } | null
  lastClickBlock: { x: number; z: number } | null
  dimension: number
  loading: boolean
}>()

const emit = defineEmits<{
  goToUserCoord: []
  copyCoord: [x: number, z: number]
  zoomIn: []
  zoomOut: []
  resetView: []
}>()
</script>

<template>
  <!-- 悬浮坐标 + 群系名 -->
  <div v-if="mouseBlock" class="absolute top-2 left-2 px-2 py-1 bg-black/70 text-white text-xs rounded font-mono pointer-events-none z-10">
    {{ formatCoord(mouseBlock.x, mouseBlock.z) }}
    <span v-if="mouseBiomeName" class="ml-1 text-gray-300">{{ mouseBiomeName }}</span>
  </div>
  <!-- 悬浮结构提示 -->
  <div v-if="hoverStruct" class="absolute top-2 right-2 px-2 py-1 bg-black/80 text-white text-xs rounded pointer-events-none z-10">
    {{ getStructIcon(hoverStruct.stype).label }}
    {{ formatCoord(hoverStruct.x, hoverStruct.z) }}
  </div>
  <!-- 悬浮出生点/要塞提示 -->
  <div v-else-if="hoverMarker" class="absolute top-2 right-2 px-2 py-1 bg-black/80 text-white text-xs rounded pointer-events-none z-10">
    {{ hoverMarker.label }}
    {{ formatCoord(hoverMarker.x, hoverMarker.z) }}
  </div>

  <!-- 左下角：大型群系 + 坐标输入 + 点击坐标 -->
  <div class="absolute bottom-2 left-2 flex flex-col gap-1 z-10">
    <!-- 坐标输入面板（展开时显示） -->
    <div v-if="showCoordPanel" class="mb-1 p-2 bg-white/95 rounded shadow-lg flex flex-col gap-1">
      <div class="flex items-center gap-1">
        <Input v-model="userX" placeholder="X" width="60px" @keydown.enter="emit('goToUserCoord')" />
        <Input v-model="userZ" placeholder="Z" width="60px" @keydown.enter="emit('goToUserCoord')" />
        <Button type="primary" size="mini" @click="emit('goToUserCoord')">前往</Button>
      </div>
      <div class="flex items-center gap-1">
        <span class="text-xs text-gray-600 w-6">Y</span>
        <Input v-model.number="yCoord" type="number" width="60px" />
        <Button
          :type="doContour ? 'primary' : 'secondary'"
          size="mini"
          class="!h-7"
          @click="doContour = !doContour"
        >等高线</Button>
        <span class="text-xs text-gray-600">限高</span>
        <Input v-model.number="ymaxLimit" type="number" width="50px" placeholder="0" />
      </div>
    </div>
    <!-- 点击坐标显示（含复制按钮，复用项目 Button.vue） -->
    <div v-if="lastClickBlock" class="flex items-center gap-1">
      <span class="px-2 py-0.5 bg-black/70 text-white text-xs rounded font-mono pointer-events-none">
        {{ formatCoord(lastClickBlock.x, lastClickBlock.z) }}
      </span>
      <Tooltip text="复制坐标" position="top" :delay="200">
        <Button
          type="ghost"
          size="mini"
          class="!h-5 !px-1.5 !text-xs"
          @click="emit('copyCoord', lastClickBlock!.x, lastClickBlock!.z)"
        >复制</Button>
      </Tooltip>
    </div>
    <!-- 按钮组 -->
    <div class="flex gap-1">
      <Tooltip text="大型群系" position="top" :delay="200">
        <Button
          :type="largeBiomes ? 'primary' : 'secondary'"
          size="mini"
          :disabled="dimension !== 0"
          class="!w-7 !h-7 !p-0 !flex !justify-center !items-center"
          @click="largeBiomes = !largeBiomes"
        >
          <Squares2X2Icon class="h-4 w-4" />
        </Button>
      </Tooltip>
      <Tooltip text="前往坐标" position="top" :delay="200">
        <Button
          :type="showCoordPanel ? 'primary' : 'secondary'"
          size="mini"
          class="!w-7 !h-7 !p-0 !flex !justify-center !items-center"
          @click="showCoordPanel = !showCoordPanel"
        >
          <MapPinIcon class="h-4 w-4" />
        </Button>
      </Tooltip>
    </div>
  </div>

  <!-- 右下角：缩放控件 -->
  <div class="absolute bottom-2 right-2 flex flex-col gap-1 z-10">
    <Tooltip text="放大" position="left" :delay="200">
      <Button type="secondary" size="mini" class="!w-7 !h-7 !p-0 !flex !justify-center !items-center" @click="emit('zoomIn')"><PlusIcon class="h-4 w-4" /></Button>
    </Tooltip>
    <Tooltip text="缩小" position="left" :delay="200">
      <Button type="secondary" size="mini" class="!w-7 !h-7 !p-0 !flex !justify-center !items-center" @click="emit('zoomOut')"><MinusIcon class="h-4 w-4" /></Button>
    </Tooltip>
    <Tooltip text="重置视图" position="left" :delay="200">
      <Button type="secondary" size="mini" class="!w-7 !h-7 !p-0 !flex !justify-center !items-center" @click="emit('resetView')"><AdjustmentsHorizontalIcon class="h-4 w-4" /></Button>
    </Tooltip>
  </div>

  <!-- 加载遮罩 -->
  <div v-if="loading" class="absolute inset-0 bg-black/30 flex items-center justify-center pointer-events-none z-20">
    <div class="text-white text-sm">加载中...</div>
  </div>
</template>
