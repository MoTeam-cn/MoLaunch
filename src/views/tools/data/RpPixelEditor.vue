<script setup lang="ts">
/**
 * 纹理像素画板（canvas 原生，不引库）
 *
 * 加载纹理到 offscreen canvas 逐像素编辑：铅笔/橡皮/取色 + MC 16 色预设与自定义取色，
 * 撤销栈 + 放大网格显示；保存经 rpWrite(base64) 写回包内同名文件。
 */
import { computed, defineAsyncComponent, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import {
  ArrowPathIcon,
  CheckIcon,
  CursorArrowRaysIcon,
  NoSymbolIcon,
  PaintBrushIcon,
  PhotoIcon,
  XMarkIcon,
} from '@heroicons/vue/24/outline'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import { toastError, toastSuccess } from '@/utils/toast'
import { rpWrite } from '@/utils/api/tools'

/** Minecraft 16 色（染料/文本颜色，常用像素画预设） */
const MC_COLORS = [
  '#000000', '#0000AA', '#00AA00', '#00AAAA',
  '#AA0000', '#AA00AA', '#FFAA00', '#AAAAAA',
  '#555555', '#5555FF', '#55FF55', '#55FFFF',
  '#FF5555', '#FF55FF', '#FFFF55', '#FFFFFF',
]

const props = defineProps<{
  workDir: string
  relPath: string
  src: string
  name: string
}>()
const emit = defineEmits<{ (e: 'saved'): void; (e: 'close'): void }>()

const canvasRef = ref<HTMLCanvasElement | null>(null)
const imgSize = ref({ w: 0, h: 0 })
const tool = ref<'pencil' | 'eraser' | 'picker'>('pencil')
const color = ref('#FFFFFF')
const showGrid = ref(true)
const saving = ref(false)
const loading = ref(false)
/** 撤销栈：绘制前快照（ImageData），最多 30 步 */
const undoStack = ref<ImageData[]>([])
let painting = false
let lastPaint = { x: -1, y: -1 }

/** 放大显示：单像素显示像素数（自适应，最长边 512px 以内） */
const scale = computed(() => {
  const max = Math.max(imgSize.value.w, imgSize.value.h)
  if (max === 0) return 1
  return Math.max(1, Math.min(32, Math.floor(512 / max)))
})
const displayStyle = computed(() => ({
  width: `${imgSize.value.w * scale.value}px`,
  height: `${imgSize.value.h * scale.value}px`,
  '--grid-size': `${scale.value}px`,
}))

const canUndo = computed(() => undoStack.value.length > 0)

watch(
  () => props.src,
  async () => {
    await loadTexture()
  },
  { immediate: true },
)

async function loadTexture() {
  if (!props.src) return
  loading.value = true
  try {
    const img = await loadImage(props.src)
    imgSize.value = { w: img.naturalWidth, h: img.naturalHeight }
    await nextTick()
    const canvas = canvasRef.value
    if (!canvas) return
    canvas.width = img.naturalWidth
    canvas.height = img.naturalHeight
    const ctx = canvas.getContext('2d')
    if (!ctx) return
    ctx.clearRect(0, 0, canvas.width, canvas.height)
    ctx.drawImage(img, 0, 0)
    pushSnapshot()
  } catch {
    toastError('纹理加载失败')
  } finally {
    loading.value = false
  }
}

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => resolve(img)
    img.onerror = () => reject(new Error('图片加载失败'))
    img.src = src
  })
}

function pushSnapshot() {
  const ctx = canvasRef.value?.getContext('2d')
  if (!ctx) return
  undoStack.value.push(ctx.getImageData(0, 0, ctx.canvas.width, ctx.canvas.height))
  if (undoStack.value.length > 30) undoStack.value.shift()
}

function undo() {
  const ctx = canvasRef.value?.getContext('2d')
  const snap = undoStack.value.pop()
  if (!ctx || !snap) return
  ctx.putImageData(snap, 0, 0)
}

function canvasPos(e: MouseEvent): { x: number; y: number } {
  const canvas = canvasRef.value
  if (!canvas) return { x: -1, y: -1 }
  const rect = canvas.getBoundingClientRect()
  return {
    x: Math.floor(((e.clientX - rect.left) / rect.width) * canvas.width),
    y: Math.floor(((e.clientY - rect.top) / rect.height) * canvas.height),
  }
}

function paintAt(x: number, y: number) {
  const canvas = canvasRef.value
  const ctx = canvas?.getContext('2d')
  if (!canvas || !ctx) return
  if (x < 0 || y < 0 || x >= canvas.width || y >= canvas.height) return
  if (tool.value === 'picker') {
    const d = ctx.getImageData(x, y, 1, 1).data
    color.value = rgbToHex(d[0], d[1], d[2])
    tool.value = 'pencil'
    return
  }
  ctx.fillStyle = tool.value === 'eraser' ? 'rgba(0,0,0,0)' : color.value
  ctx.fillRect(x, y, 1, 1)
}

function onMouseDown(e: MouseEvent) {
  if (e.button !== 0) return
  painting = true
  pushSnapshot()
  const { x, y } = canvasPos(e)
  paintAt(x, y)
  lastPaint = { x, y }
}

function onMouseMove(e: MouseEvent) {
  if (!painting) return
  const { x, y } = canvasPos(e)
  if (x === lastPaint.x && y === lastPaint.y) return
  paintAt(x, y)
  lastPaint = { x, y }
}

function onMouseUp() {
  painting = false
  lastPaint = { x: -1, y: -1 }
}

function rgbToHex(r: number, g: number, b: number): string {
  return `#${[r, g, b].map((v) => v.toString(16).padStart(2, '0')).join('')}`
}

async function save() {
  const canvas = canvasRef.value
  if (!canvas) return
  saving.value = true
  try {
    const dataUri = canvas.toDataURL('image/png')
    const res = await rpWrite({
      work_dir: props.workDir,
      rel_path: props.relPath,
      kind: 'base64',
      content: dataUri,
    })
    if (!res.success) {
      toastError(res.message)
      return
    }
    toastSuccess('像素画已保存')
    emit('saved')
  } catch (e) {
    toastError(`保存失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    saving.value = false
  }
}

onBeforeUnmount(() => {
  painting = false
})
</script>

<template>
  <div class="space-y-3">
    <div class="flex flex-wrap items-center gap-2">
      <PhotoIcon class="h-4 w-4 text-gray-500" />
      <h4 class="text-sm font-medium text-gray-700">像素画板</h4>
      <span class="text-xs text-gray-400">{{ name }} · {{ imgSize.w }}×{{ imgSize.h }}</span>
      <div class="ml-auto flex items-center gap-1.5">
        <button
          type="button"
          class="flex items-center gap-1 rounded border px-2 py-1 text-xs"
          :class="tool === 'pencil' ? 'border-blue-400 bg-blue-50 text-blue-600' : 'border-gray-300 text-gray-600 hover:bg-gray-50'"
          @click="tool = 'pencil'"
        >
          <PaintBrushIcon class="h-3.5 w-3.5" />画笔
        </button>
        <button
          type="button"
          class="flex items-center gap-1 rounded border px-2 py-1 text-xs"
          :class="tool === 'eraser' ? 'border-blue-400 bg-blue-50 text-blue-600' : 'border-gray-300 text-gray-600 hover:bg-gray-50'"
          @click="tool = 'eraser'"
        >
          <NoSymbolIcon class="h-3.5 w-3.5" />橡皮
        </button>
        <button
          type="button"
          class="flex items-center gap-1 rounded border px-2 py-1 text-xs"
          :class="tool === 'picker' ? 'border-blue-400 bg-blue-50 text-blue-600' : 'border-gray-300 text-gray-600 hover:bg-gray-50'"
          @click="tool = 'picker'"
        >
          <CursorArrowRaysIcon class="h-3.5 w-3.5" />取色
        </button>
        <button
          type="button"
          class="flex items-center gap-1 rounded border px-2 py-1 text-xs"
          :class="showGrid ? 'border-blue-400 bg-blue-50 text-blue-600' : 'border-gray-300 text-gray-600 hover:bg-gray-50'"
          @click="showGrid = !showGrid"
        >
          网格
        </button>
        <button
          type="button"
          class="flex items-center gap-1 rounded border border-gray-300 px-2 py-1 text-xs text-gray-600 hover:bg-gray-50 disabled:opacity-40"
          :disabled="!canUndo"
          @click="undo"
        >
          <ArrowPathIcon class="h-3.5 w-3.5" />撤销
        </button>
        <Button size="small" type="outline" @click="emit('close')">
          <template #icon><XMarkIcon class="h-4 w-4" /></template>
          返回
        </Button>
        <Button size="small" :loading="saving" @click="save">
          <template #icon><CheckIcon class="h-4 w-4" /></template>
          {{ saving ? '保存中…' : '保存' }}
        </Button>
      </div>
    </div>

    <div class="flex flex-wrap items-center gap-1.5 rounded border border-gray-200 bg-gray-50 p-2">
      <span class="text-xs text-gray-500">颜色</span>
      <Tooltip v-for="c in MC_COLORS" :key="c" :text="c">
        <button
          type="button"
          class="h-5 w-5 rounded-sm border border-black/10"
          :class="color.toLowerCase() === c.toLowerCase() ? 'ring-2 ring-blue-500 ring-offset-1' : ''"
          :style="{ backgroundColor: c }"
          @click="color = c"
        />
      </Tooltip>
      <Tooltip text="自定义颜色">
        <label
          class="relative ml-1 h-5 w-5 cursor-pointer overflow-hidden rounded-sm border border-gray-300 bg-white"
        >
          <input v-model="color" type="color" class="absolute inset-0 h-full w-full cursor-pointer opacity-0" />
        </label>
      </Tooltip>
      <span class="ml-1 text-xs text-gray-500">{{ color }}</span>
    </div>

    <div v-if="loading" class="py-8 text-center text-sm text-gray-400">纹理加载中…</div>
    <div
      v-else
      class="grid place-items-center overflow-auto rounded border border-gray-200 bg-white p-2"
    >
      <canvas
        ref="canvasRef"
        class="cursor-crosshair touch-none select-none"
        :class="{ 'image-pixelated': true, 'show-grid': showGrid }"
        :style="displayStyle"
        @mousedown="onMouseDown"
        @mousemove="onMouseMove"
        @mouseup="onMouseUp"
        @mouseleave="onMouseUp"
      />
    </div>
    <p v-if="imgSize.w" class="text-xs text-gray-400">
      提示：点击 / 拖拽在对应像素上绘制；「取色」点击后自动切回画笔；「保存」写回包内同名文件。
    </p>
  </div>
</template>

<style scoped>
.image-pixelated {
  image-rendering: pixelated;
  background-image:
    linear-gradient(45deg, #f0f0f0 25%, transparent 25%, transparent 75%, #f0f0f0 75%),
    linear-gradient(45deg, #f0f0f0 25%, transparent 25%, transparent 75%, #f0f0f0 75%);
  background-size:
    var(--grid-size) var(--grid-size),
    var(--grid-size) var(--grid-size);
  background-position:
    0 0,
    calc(var(--grid-size) / 2) calc(var(--grid-size) / 2);
}
.show-grid {
  outline: 1px solid rgba(0, 0, 0, 0.08);
  background-image:
    linear-gradient(45deg, #f0f0f0 25%, transparent 25%, transparent 75%, #f0f0f0 75%),
    linear-gradient(45deg, #f0f0f0 25%, transparent 25%, transparent 75%, #f0f0f0 75%),
    repeating-linear-gradient(to right, rgba(0, 0, 0, 0.1) 0 1px, transparent 1px var(--grid-size)),
    repeating-linear-gradient(to bottom, rgba(0, 0, 0, 0.1) 0 1px, transparent 1px var(--grid-size));
  background-size:
    var(--grid-size) var(--grid-size),
    var(--grid-size) var(--grid-size),
    var(--grid-size) var(--grid-size),
    var(--grid-size) var(--grid-size);
  background-position:
    0 0,
    calc(var(--grid-size) / 2) calc(var(--grid-size) / 2),
    0 0,
    0 0;
}
</style>
