<script setup lang="ts">
/**
 * 皮肤编辑器工具栏
 *
 * 工具选择 / 颜色 / 笔刷粗细 / 部件定位 / 撤销重做 / 导入导出。
 * 纯展示组件，所有状态由父级 useSkinEditor 持有。
 */

import {
  ArrowDownTrayIcon,
  ArrowUpTrayIcon,
  ArrowUturnLeftIcon,
  ArrowUturnRightIcon,
  BackspaceIcon,
  EyeDropperIcon,
  PaintBrushIcon,
  PencilIcon,
  SwatchIcon,
  TrashIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import ColorPicker from '@/components/common/ColorPicker.vue'
import SegmentedButtons from '@/components/common/SegmentedButtons.vue'
import { SKIN_PALETTE, type SkinTool } from '@/composables/useSkinEditor'
import type { SkinPart, SkinPartId } from '@/utils/skin-editor/layout'

defineProps<{
  tool: SkinTool
  color: string
  brushSize: number
  canUndo: boolean
  canRedo: boolean
  activePart: SkinPartId
  parts: SkinPart[]
}>()

const emit = defineEmits<{
  'update:tool': [value: SkinTool]
  'update:color': [value: string]
  'update:brushSize': [value: number]
  'update:activePart': [value: SkinPartId]
  undo: []
  redo: []
  clear: []
  import: []
  export: []
}>()

const brushOptions = [
  { label: '1', value: 1 },
  { label: '2', value: 2 },
  { label: '3', value: 3 },
]

const tools: { id: SkinTool; label: string; icon: unknown }[] = [
  { id: 'pencil', label: '画笔', icon: PencilIcon },
  { id: 'eraser', label: '橡皮', icon: BackspaceIcon },
  { id: 'fill', label: '填充', icon: PaintBrushIcon },
  { id: 'picker', label: '取色', icon: EyeDropperIcon },
]
</script>

<template>
  <div class="flex flex-col gap-3 rounded-lg border border-gray-300 bg-white p-3">
    <div class="flex flex-wrap items-center gap-2">
      <Button
        v-for="item in tools"
        :key="item.id"
        :type="tool === item.id ? 'primary' : 'outline'"
        size="small"
        @click="emit('update:tool', item.id)"
      >
        <component :is="item.icon" class="h-4 w-4" />
        {{ item.label }}
      </Button>

      <span class="mx-1 h-5 w-px bg-gray-200" />

      <Button size="small" type="outline" :disabled="!canUndo" @click="emit('undo')">
        <ArrowUturnLeftIcon class="h-4 w-4" />
        撤销
      </Button>
      <Button size="small" type="outline" :disabled="!canRedo" @click="emit('redo')">
        <ArrowUturnRightIcon class="h-4 w-4" />
        重做
      </Button>
      <Button size="small" type="outline" @click="emit('clear')">
        <TrashIcon class="h-4 w-4" />
        清空
      </Button>

      <span class="mx-1 h-5 w-px bg-gray-200" />

      <Button size="small" type="outline" @click="emit('import')">
        <ArrowUpTrayIcon class="h-4 w-4" />
        导入 PNG
      </Button>
      <Button size="small" @click="emit('export')">
        <ArrowDownTrayIcon class="h-4 w-4" />
        导出 PNG
      </Button>
    </div>

    <div class="flex flex-wrap items-center gap-3">
      <div class="flex items-center gap-2">
        <SwatchIcon class="h-4 w-4 text-gray-500" />
        <span class="text-xs text-gray-600">颜色</span>
        <ColorPicker
          :model-value="color"
          @update:model-value="emit('update:color', $event)"
        />
      </div>

      <div class="flex flex-wrap items-center gap-1">
        <button
          v-for="swatch in SKIN_PALETTE"
          :key="swatch"
          type="button"
          class="h-5 w-5 rounded border border-gray-300 transition hover:scale-110"
          :style="{ backgroundColor: swatch }"
          :aria-label="swatch"
          @click="emit('update:color', swatch)"
        />
      </div>

      <div class="flex items-center gap-2">
        <span class="text-xs text-gray-600">笔刷</span>
        <SegmentedButtons
          :model-value="brushSize"
          :options="brushOptions"
          @update:model-value="emit('update:brushSize', Number($event))"
        />
      </div>

      <div class="flex flex-wrap items-center gap-1">
        <span class="text-xs text-gray-600">部件</span>
        <button
          v-for="part in parts"
          :key="part.id"
          type="button"
          class="rounded border px-2 py-0.5 text-xs transition"
          :class="
            activePart === part.id
              ? 'border-brand-2 bg-brand-2 text-white'
              : 'border-gray-300 bg-white text-gray-700 hover:border-brand-3'
          "
          @click="emit('update:activePart', part.id)"
        >
          {{ part.label }}
        </button>
      </div>
    </div>
  </div>
</template>
