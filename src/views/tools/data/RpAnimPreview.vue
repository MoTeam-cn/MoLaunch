<script setup lang="ts">
/**
 * 动画纹理帧预览（读取 .png.mcmeta 动画配置，canvas 逐帧播放）
 *
 * 帧布局：MC 动画纹理为垂直堆叠 sprite sheet，默认帧宽=纹理宽、帧高=帧宽；
 * mcmeta 的 animation 段可指定 width/height/frametime/frames（含单帧 time 覆盖）。
 * 1 tick = 50ms；支持播放/暂停、速度倍率与帧信息展示。
 */
import { computed, defineAsyncComponent, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { FilmIcon, PauseIcon, PlayIcon } from '@heroicons/vue/24/outline'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
import { rpRead } from '@/utils/api/tools'

/** mcmeta 动画配置（animation 段） */
interface AnimConfig {
  width?: number
  height?: number
  frametime?: number
  interpolate?: boolean
  frames?: Array<number | { index: number; time?: number }>
}

const props = defineProps<{
  workDir: string
  relPath: string
  src: string
  name: string
}>()
const emit = defineEmits<{ (e: 'close'): void }>()

const canvasRef = ref<HTMLCanvasElement | null>(null)
const img = ref<HTMLImageElement | null>(null)
const config = ref<AnimConfig | null>(null)
const playing = ref(false)
const speed = ref(1)
const frameCount = ref(0)
const currentFrame = ref(0)
const loadError = ref('')
let frameDurations: number[] = []
let tickAcc = 0
let timer: number | null = null

/** 帧尺寸（默认帧宽=纹理宽、帧高=帧宽） */
const frameSize = computed(() => {
  const image = img.value
  if (!image) return { w: 0, h: 0 }
  const c = config.value
  const w = c?.width ?? image.naturalWidth
  const h = c?.height ?? w
  return { w, h }
})
const displayStyle = computed(() => {
  const { w, h } = frameSize.value
  const scale = w > 0 ? Math.max(1, Math.min(16, Math.floor(256 / w))) : 1
  return { width: `${w * scale}px`, height: `${h * scale}px` }
})
const frameLabel = computed(() => {
  const c = config.value
  const fw = frameSize.value.w
  const fh = frameSize.value.h
  return `${fw}×${fh}${c?.interpolate ? ' · 插值' : ''}${c?.frames ? ` · 帧表 ${c.frames.length}` : ''}`
})

watch(
  () => props.src,
  async () => {
    await load()
  },
  { immediate: true },
)

async function load() {
  stop()
  loadError.value = ''
  config.value = null
  frameCount.value = 0
  currentFrame.value = 0
  if (!props.src) return
  try {
    const image = await loadImage(props.src)
    img.value = image
    const meta = await readAnimMeta()
    config.value = meta
    await nextTick()
    const canvas = canvasRef.value
    if (!canvas) return
    canvas.width = frameSize.value.w
    canvas.height = frameSize.value.h
    buildDurations()
    drawFrame(0)
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e)
  }
}

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image()
    image.onload = () => resolve(image)
    image.onerror = () => reject(new Error('图片加载失败'))
    image.src = src
  })
}

/** 读取同名 .png.mcmeta 并解析 animation 段（缺失/解析失败按默认动画处理） */
async function readAnimMeta(): Promise<AnimConfig | null> {
  try {
    const res = await rpRead(props.workDir, `${props.relPath}.mcmeta`)
    if (res.error) return null
    const obj = JSON.parse(res.content) as { animation?: AnimConfig }
    return obj.animation ?? null
  } catch {
    return null
  }
}

/** 每帧持续时间（tick 数）：单帧 time 覆盖优先，否则 frametime，默认 1 */
function buildDurations() {
  const c = config.value
  const fallback = c?.frametime && c.frametime > 0 ? c.frametime : 1
  const cols = Math.max(1, Math.floor((img.value?.naturalWidth ?? 0) / frameSize.value.w))
  const rows = Math.max(1, Math.floor((img.value?.naturalHeight ?? 0) / frameSize.value.h))
  frameCount.value = cols * rows
  if (Array.isArray(c?.frames) && c.frames.length > 0) {
    frameDurations = c.frames.map((f) => {
      const t = typeof f === 'number' ? undefined : f.time
      return t && t > 0 ? t : fallback
    })
  } else {
    frameDurations = Array.from({ length: frameCount.value }, () => fallback)
  }
  if (frameDurations.length === 0) frameDurations = [1]
}

/** 帧索引 → sprite sheet 中的像素坐标（index 超过帧表长度时按顺序模取） */
function drawFrame(index: number) {
  const canvas = canvasRef.value
  const image = img.value
  const ctx = canvas?.getContext('2d')
  if (!canvas || !image || !ctx) return
  const { w, h } = frameSize.value
  const cols = Math.max(1, Math.floor(image.naturalWidth / w))
  const list = frameDurations.length
  const idx = index % list
  let sourceIdx = idx
  if (Array.isArray(config.value?.frames) && config.value.frames.length > 0) {
    const f = config.value.frames[idx]
    sourceIdx = typeof f === 'number' ? f : (f?.index ?? idx)
  }
  const col = sourceIdx % cols
  const row = Math.floor(sourceIdx / cols)
  ctx.clearRect(0, 0, canvas.width, canvas.height)
  ctx.drawImage(image, col * w, row * h, w, h, 0, 0, canvas.width, canvas.height)
  currentFrame.value = index % list
}

function togglePlay() {
  if (playing.value) stop()
  else start()
}

function start() {
  if (timer !== null || frameCount.value === 0) return
  playing.value = true
  tickAcc = 0
  timer = window.setInterval(() => {
    tickAcc++
    const dur = frameDurations[currentFrame.value] / speed.value
    if (tickAcc >= dur) {
      tickAcc = 0
      drawFrame(currentFrame.value + 1)
    }
  }, 50)
}

function stop() {
  if (timer !== null) {
    window.clearInterval(timer)
    timer = null
  }
  playing.value = false
}

function setSpeed(v: number) {
  speed.value = v
}

onBeforeUnmount(stop)
</script>

<template>
  <div class="space-y-3">
    <div class="flex flex-wrap items-center gap-2">
      <FilmIcon class="h-4 w-4 text-gray-500" />
      <h4 class="text-sm font-medium text-gray-700">动画预览</h4>
      <span class="text-xs text-gray-400">{{ name }}</span>
      <div class="ml-auto flex items-center gap-1.5">
        <button
          v-for="v in [1, 2, 4]"
          :key="v"
          type="button"
          class="rounded border px-2 py-1 text-xs"
          :class="speed === v ? 'border-blue-400 bg-blue-50 text-blue-600' : 'border-gray-300 text-gray-600 hover:bg-gray-50'"
          @click="setSpeed(v)"
        >
          {{ v }}×
        </button>
        <Button size="small" type="outline" @click="emit('close')">返回</Button>
        <Button size="small" :disabled="frameCount === 0" @click="togglePlay">
          <template #icon>
            <PauseIcon v-if="playing" class="h-4 w-4" />
            <PlayIcon v-else class="h-4 w-4" />
          </template>
          {{ playing ? '暂停' : '播放' }}
        </Button>
      </div>
    </div>

    <div v-if="loadError" class="text-sm text-red-600">{{ loadError }}</div>
    <div
      v-else
      class="grid place-items-center rounded border border-gray-200 bg-gray-50 p-4"
    >
      <canvas ref="canvasRef" class="image-pixelated" :style="displayStyle" />
    </div>
    <p v-if="frameCount" class="text-xs text-gray-400">
      {{ frameLabel }} · 共 {{ frameCount }} 帧 · 当前第 {{ currentFrame + 1 }} 帧（1 tick = 50ms）
    </p>
    <p v-else-if="!loadError" class="text-sm text-gray-400">动画纹理加载中…</p>
  </div>
</template>

<style scoped>
.image-pixelated {
  image-rendering: pixelated;
}
</style>
