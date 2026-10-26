<script setup lang="ts">
/**
 * 投影预览
 *
 * 打开 Sponge .schematic / Litematica .litematic / 原版结构 .nbt，
 * 按 X/Y/Z 轴逐层浏览方块分布、查看方块统计，并可导出为三种格式之一。
 * 只读预览：不含方块编辑与旋转镜像。
 */

import { computed, ref, watch } from 'vue'
import { ArrowDownTrayIcon, FolderOpenIcon } from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import SegmentedButtons from '@/components/common/SegmentedButtons.vue'
import { pickFile, pickSavePath } from '@/utils/fileDialog'
import { toastError, toastSuccess } from '@/utils/toast'
import { schematicExport, schematicOpen, type SchematicOpenResult } from '@/utils/api/tools/schematic'
import {
  decodeBlocks,
  extractSlice,
  findAirIndex,
  sliceBounds,
  type SliceAxis,
} from '@/utils/schematic/decode'
import SchematicCanvas from './SchematicCanvas.vue'
import SchematicStats from './SchematicStats.vue'

const payload = ref<SchematicOpenResult | null>(null)
const blocks = ref<Uint16Array | null>(null)
const sourcePath = ref('')
const airIndex = ref(0)
const axis = ref<SliceAxis>('y')
const layer = ref(0)
const hover = ref<{ x: number; y: number; id: string } | null>(null)
const loading = ref(false)
const exporting = ref(false)
const exportFormat = ref('sponge_v2')

const axisOptions = [
  { label: 'Y 轴（水平层）', value: 'y' },
  { label: 'X 轴', value: 'x' },
  { label: 'Z 轴', value: 'z' },
]

const formatOptions = [
  { label: 'Sponge', value: 'sponge_v2' },
  { label: 'Litematic', value: 'litematic' },
  { label: '结构文件', value: 'structure' },
]

const FORMAT_EXT: Record<string, string> = {
  sponge_v2: 'schem',
  litematic: 'litematic',
  structure: 'nbt',
}

const bounds = computed(() =>
  payload.value
    ? sliceBounds(payload.value.size, axis.value)
    : { layers: 1, width: 1, height: 1 },
)

const slice = computed(() => {
  if (!payload.value || !blocks.value) return null
  return extractSlice(blocks.value, payload.value.size, axis.value, layer.value)
})

const hoverLabel = computed(() => {
  if (!hover.value) return ''
  const id = hover.value.id.replace('minecraft:', '')
  return `(${hover.value.x}, ${hover.value.y}) ${id}`
})

async function openFile(): Promise<void> {
  loading.value = true
  try {
    const path = await pickFile({
      title: '选择投影文件',
      filters: [
        { name: '投影文件', extensions: ['schem', 'schematic', 'litematic', 'nbt'] },
      ],
    })
    if (!path) return
    const result = await schematicOpen(path)
    payload.value = result
    blocks.value = decodeBlocks(result.blocksBase64)
    sourcePath.value = path
    airIndex.value = findAirIndex(result.palette)
    layer.value = Math.floor(sliceBounds(result.size, axis.value).layers / 2)
    toastSuccess(`已打开 ${result.name}（${result.format}）`)
  } catch (err) {
    toastError(`打开失败：${err}`)
  } finally {
    loading.value = false
  }
}

async function doExport(): Promise<void> {
  if (!payload.value || !sourcePath.value) return
  exporting.value = true
  try {
    const ext = FORMAT_EXT[exportFormat.value] ?? 'schem'
    const target = await pickSavePath({
      title: '导出投影',
      defaultPath: `${payload.value.name}.${ext}`,
      filters: [{ name: ext.toUpperCase(), extensions: [ext] }],
    })
    if (!target) return
    const result = await schematicExport(sourcePath.value, target, exportFormat.value)
    toastSuccess(`已导出（${(result.bytes / 1024).toFixed(1)} KB）`)
  } catch (err) {
    toastError(`导出失败：${err}`)
  } finally {
    exporting.value = false
  }
}

watch(axis, () => {
  layer.value = Math.floor(bounds.value.layers / 2)
})
</script>

<template>
  <section class="flex flex-col gap-4 rounded-lg border border-gray-300 bg-white p-4">
    <div class="flex flex-wrap items-center gap-2">
      <Button :loading="loading" @click="openFile">
        <FolderOpenIcon class="h-4 w-4" />
        打开投影
      </Button>

      <template v-if="payload">
        <span class="text-xs text-gray-500">
          {{ payload.name }} · {{ payload.format }} ·
          {{ payload.size[0] }}×{{ payload.size[1] }}×{{ payload.size[2] }}
        </span>
        <span class="mx-1 h-5 w-px bg-gray-200" />
        <SegmentedButtons v-model="exportFormat" :options="formatOptions" />
        <Button type="outline" :loading="exporting" @click="doExport">
          <ArrowDownTrayIcon class="h-4 w-4" />
          导出
        </Button>
      </template>
    </div>

    <p v-if="!payload" class="text-xs text-gray-500">
      支持 Sponge（.schem / .schematic，v1 与 v2）、Litematica（.litematic）与原版结构文件（.nbt）。
      单个投影上限 400 万方块。
    </p>

    <template v-else>
      <div class="flex flex-wrap items-center gap-3">
        <SegmentedButtons v-model="axis" :options="axisOptions" />
        <div class="flex min-w-[240px] flex-1 items-center gap-2">
          <span class="text-xs text-gray-600">第 {{ layer + 1 }} / {{ bounds.layers }} 层</span>
          <input
            v-model.number="layer"
            type="range"
            min="0"
            :max="Math.max(bounds.layers - 1, 0)"
            class="flex-1 accent-brand-2"
          />
        </div>
      </div>

      <div class="flex flex-wrap items-start gap-4">
        <div class="flex flex-col gap-2">
          <SchematicCanvas
            v-if="slice"
            :slice="slice"
            :palette="payload.palette"
            :air-index="airIndex"
            @hover="hover = $event"
          />
          <span class="h-4 text-xs text-gray-500">{{ hoverLabel }}</span>
        </div>

        <div class="min-w-[260px] flex-1">
          <SchematicStats
            :counts="payload.counts"
            :non-air="payload.nonAir"
            :total="payload.total"
          />
        </div>
      </div>
    </template>
  </section>
</template>
