/**
 * 皮肤编辑器状态与操作
 *
 * 管理像素缓冲、当前工具/颜色/笔刷、部件高亮与撤销重做，
 * 并对 3D 预览地址做节流刷新（避免拖动时高频重建 skinview3d）。
 */

import { computed, ref, shallowRef, onUnmounted } from 'vue'
import { PixelBuffer, type Rgba } from '@/utils/skin-editor/pixel-buffer'
import { decodeSkinPng, encodeSkinPng } from '@/utils/skin-editor/image'
import {
  getSkinParts,
  type SkinLayout,
  type SkinPartId,
  type SkinVariant,
} from '@/utils/skin-editor/layout'

export type SkinTool = 'pencil' | 'eraser' | 'fill' | 'picker'

/** 常用色板（皮肤常用肤色/发色/衣物色） */
export const SKIN_PALETTE: string[] = [
  '#ffffff', '#c8c8c8', '#8a8a8a', '#3f3f3f', '#000000',
  '#f6d5b8', '#e0b090', '#c98b6b', '#8d5524', '#5a3213',
  '#f5d76e', '#e8a33d', '#d9534f', '#c0392b', '#8e44ad',
  '#3498db', '#2980b9', '#2ecc71', '#27ae60', '#16a085',
]

/** 预览节流间隔（毫秒） */
const PREVIEW_THROTTLE_MS = 120

function hexToRgba(hex: string): Rgba {
  const normalized = hex.replace('#', '')
  const full =
    normalized.length === 3
      ? normalized.split('').map((c) => c + c).join('')
      : normalized.padEnd(6, '0')
  return {
    r: Number.parseInt(full.slice(0, 2), 16),
    g: Number.parseInt(full.slice(2, 4), 16),
    b: Number.parseInt(full.slice(4, 6), 16),
    a: 255,
  }
}

const TRANSPARENT: Rgba = { r: 0, g: 0, b: 0, a: 0 }

export function useSkinEditor() {
  const layout = ref<SkinLayout>(64)
  const variant = ref<SkinVariant>('classic')
  const buffer = shallowRef(PixelBuffer.blank(64))
  const revision = ref(0)
  const tool = ref<SkinTool>('pencil')
  const color = ref('#3498db')
  const brushSize = ref(1)
  const activePart = ref<SkinPartId>('head')
  const canUndo = ref(false)
  const canRedo = ref(false)
  const previewUrl = ref<string | null>(null)
  const fileName = ref('skin.png')

  const parts = computed(() => getSkinParts(layout.value))
  const size = computed(() => buffer.value.width)

  let lastPoint: { x: number; y: number } | null = null
  let previewTimer: ReturnType<typeof setTimeout> | null = null

  function refreshPreview(): void {
    if (previewTimer) clearTimeout(previewTimer)
    previewTimer = setTimeout(() => {
      previewUrl.value = encodeSkinPng(buffer.value)
    }, PREVIEW_THROTTLE_MS)
  }

  function touch(): void {
    revision.value += 1
    canUndo.value = buffer.value.canUndo()
    canRedo.value = buffer.value.canRedo()
    refreshPreview()
  }

  function currentColor(): Rgba {
    return tool.value === 'eraser' ? TRANSPARENT : hexToRgba(color.value)
  }

  /** 按下：开始一次笔画，油漆桶与取色器在此一步完成 */
  function strokeStart(x: number, y: number): void {
    if (tool.value === 'picker') {
      const pixel = buffer.value.getPixel(x, y)
      if (pixel.a === 0) return
      color.value = `#${[pixel.r, pixel.g, pixel.b]
        .map((v) => v.toString(16).padStart(2, '0'))
        .join('')}`
      tool.value = 'pencil'
      return
    }
    buffer.value.beginStroke()
    if (tool.value === 'fill') {
      buffer.value.floodFill(x, y, currentColor())
      buffer.value.endStroke()
      lastPoint = null
      touch()
      return
    }
    buffer.value.paint(x, y, currentColor(), brushSize.value)
    lastPoint = { x, y }
    touch()
  }

  /** 拖动：从上一像素连直线到当前像素 */
  function strokeMove(x: number, y: number): void {
    if (!lastPoint || tool.value === 'fill' || tool.value === 'picker') return
    if (lastPoint.x === x && lastPoint.y === y) return
    buffer.value.line(lastPoint.x, lastPoint.y, x, y, currentColor(), brushSize.value)
    lastPoint = { x, y }
    touch()
  }

  /** 抬起：结算笔画 */
  function strokeEnd(): void {
    if (!lastPoint) {
      lastPoint = null
      return
    }
    buffer.value.endStroke()
    lastPoint = null
    touch()
  }

  function undo(): void {
    if (buffer.value.undo()) touch()
  }

  function redo(): void {
    if (buffer.value.redo()) touch()
  }

  function clear(): void {
    buffer.value.clear()
    touch()
  }

  /** 载入 PNG dataURL，自动识别布局 */
  async function loadDataUrl(dataUrl: string, name?: string): Promise<void> {
    const decoded = await decodeSkinPng(dataUrl)
    layout.value = decoded.layout
    buffer.value = decoded.buffer
    activePart.value = 'head'
    if (name) fileName.value = name
    touch()
  }

  /** 切换布局（64×64 / 64×32），保留已画内容 */
  function setLayout(next: SkinLayout): void {
    if (next === layout.value) return
    const resized = new PixelBuffer(next, next)
    resized.beginStroke()
    const source = buffer.value
    for (let y = 0; y < Math.min(next, source.height); y++) {
      for (let x = 0; x < Math.min(next, source.width); x++) {
        resized.setPixel(x, y, source.getPixel(x, y))
      }
    }
    resized.endStroke()
    buffer.value = resized
    layout.value = next
    touch()
  }

  /** 导出当前编辑结果为 PNG dataURL */
  function toDataUrl(): string {
    return encodeSkinPng(buffer.value)
  }

  onUnmounted(() => {
    if (previewTimer) clearTimeout(previewTimer)
  })

  return {
    layout,
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
    size,
    strokeStart,
    strokeMove,
    strokeEnd,
    refreshPreview,
    undo,
    redo,
    clear,
    loadDataUrl,
    setLayout,
    toDataUrl,
  }
}
