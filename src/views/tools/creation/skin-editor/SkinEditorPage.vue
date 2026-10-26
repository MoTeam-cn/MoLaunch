<script setup lang="ts">
/**
 * 皮肤编辑器
 *
 * 左侧像素画布（64×64 / 64×32），右侧 3D 实时预览；
 * 支持画笔 / 橡皮 / 油漆桶 / 取色、笔刷粗细、部件定位、撤销重做、导入导出 PNG。
 * 纯前端实现，不依赖任何第三方编辑器。
 */

import { defineAsyncComponent, onMounted, ref } from 'vue'
import {
  ArrowsPointingOutIcon,
  MagnifyingGlassMinusIcon,
  MagnifyingGlassPlusIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import { toastError, toastSuccess } from '@/utils/toast'
import { useSkinEditor } from '@/composables/useSkinEditor'
import { pickFile, pickSavePath } from '@/utils/fileDialog'
import { systemManager, SYSTEM_ACTIONS } from '@/utils/api/system-manager'
import SkinCanvas from './SkinCanvas.vue'
import SkinToolbar from './SkinToolbar.vue'

const SkinModel3D = defineAsyncComponent(() => import('@/components/common/SkinModel3D.vue'))

const {
  variant,
  buffer,
  revision,
  tool,
  color,
  brushSize,
  activePart,
  canUndo,
  canRedo,
  previewUrl,
  fileName,
  parts,
  strokeStart,
  strokeMove,
  strokeEnd,
  refreshPreview,
  undo,
  redo,
  clear,
  loadDataUrl,
  toDataUrl,
} = useSkinEditor()

const MIN_ZOOM = 4
const MAX_ZOOM = 16
const DEFAULT_ZOOM = 8
const zoom = ref(DEFAULT_ZOOM)

const importing = ref(false)
const exporting = ref(false)

function zoomIn(): void {
  zoom.value = Math.min(MAX_ZOOM, zoom.value + 2)
}

function zoomOut(): void {
  zoom.value = Math.max(MIN_ZOOM, zoom.value - 2)
}

function resetZoom(): void {
  zoom.value = DEFAULT_ZOOM
}

async function importPng(): Promise<void> {
  importing.value = true
  try {
    const path = await pickFile({
      title: '选择皮肤 PNG',
      filters: [{ name: 'PNG 图片', extensions: ['png'] }],
    })
    if (!path) return
    const dataUrl = await systemManager<string>(SYSTEM_ACTIONS.READ_IMAGE_DATA_URL, { path })
    const name = path.split(/[\\/]/).pop() ?? 'skin.png'
    await loadDataUrl(dataUrl, name)
    toastSuccess('皮肤已载入')
  } catch (err) {
    toastError(`导入失败：${err}`)
  } finally {
    importing.value = false
  }
}

async function exportPng(): Promise<void> {
  exporting.value = true
  try {
    const path = await pickSavePath({
      title: '导出皮肤 PNG',
      defaultPath: fileName.value,
      filters: [{ name: 'PNG 图片', extensions: ['png'] }],
    })
    if (!path) return
    const base64 = toDataUrl().replace(/^data:image\/png;base64,/, '')
    await systemManager(SYSTEM_ACTIONS.WRITE_BINARY_FILE, { path, base64 })
    toastSuccess('皮肤已导出')
  } catch (err) {
    toastError(`导出失败：${err}`)
  } finally {
    exporting.value = false
  }
}

onMounted(() => {
  refreshPreview()
})
</script>

<template>
  <div class="flex flex-col gap-4">
    <SkinToolbar
      :tool="tool"
      :color="color"
      :brush-size="brushSize"
      :can-undo="canUndo"
      :can-redo="canRedo"
      :active-part="activePart"
      :parts="parts"
      @update:tool="tool = $event"
      @update:color="color = $event"
      @update:brush-size="brushSize = $event"
      @update:active-part="activePart = $event"
      @undo="undo"
      @redo="redo"
      @clear="clear"
      @import="importPng"
      @export="exportPng"
    />

    <div class="flex flex-wrap items-start gap-4">
      <div class="flex flex-col items-center gap-2">
        <div class="flex items-center gap-2">
          <Button size="small" type="outline" :disabled="zoom <= MIN_ZOOM" @click="zoomOut">
            <MagnifyingGlassMinusIcon class="h-4 w-4" />
          </Button>
          <span class="text-xs text-gray-600">{{ zoom }}×</span>
          <Button size="small" type="outline" :disabled="zoom >= MAX_ZOOM" @click="zoomIn">
            <MagnifyingGlassPlusIcon class="h-4 w-4" />
          </Button>
          <Button size="small" type="outline" @click="resetZoom">
            <ArrowsPointingOutIcon class="h-4 w-4" />
            适配
          </Button>
        </div>

        <SkinCanvas
          :buffer="buffer"
          :revision="revision"
          :zoom="zoom"
          :parts="parts"
          :active-part="activePart"
          @stroke-start="strokeStart"
          @stroke-move="strokeMove"
          @stroke-end="strokeEnd"
          @part-change="activePart = $event"
        />
      </div>

      <div
        class="flex min-w-[240px] flex-1 flex-col gap-2 rounded-lg border border-gray-300 bg-white p-3"
      >
        <p class="text-xs font-semibold text-gray-700">3D 预览</p>
        <SkinModel3D :skin-url="previewUrl" :variant="variant" :height="300" animation="idle" />
        <p class="text-xs text-gray-500">
          导出后可在「版本设置 → 皮肤」中导入使用；64×32 旧版皮肤没有外层区。
        </p>
      </div>
    </div>
  </div>
</template>
