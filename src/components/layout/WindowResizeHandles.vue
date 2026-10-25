<script setup lang="ts">
/**
 * 无边框窗口的边缘缩放热区
 *
 * 主窗口 decorations: false，部分平台不会给无边框窗口铺系统缩放边框；
 * 这里沿四边与四角铺一层 6px 透明热区，按下即把缩放交给系统
 * （startResizeDragging），最大化 / 全屏时自动隐藏。
 */
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'

/** 与 @tauri-apps/api 的 ResizeDirection 成员一致（该类型未从包入口导出） */
type ResizeEdge =
  | 'East'
  | 'North'
  | 'NorthEast'
  | 'NorthWest'
  | 'South'
  | 'SouthEast'
  | 'SouthWest'
  | 'West'

type Zone = {
  edge: ResizeEdge
  label: string
  style: Record<string, string>
}

const EDGE = 6
const CORNER = 12

const ZONES: Zone[] = [
  { edge: 'North', label: '上边', style: { top: '0', left: `${CORNER}px`, right: `${CORNER}px`, height: `${EDGE}px`, cursor: 'ns-resize' } },
  { edge: 'South', label: '下边', style: { bottom: '0', left: `${CORNER}px`, right: `${CORNER}px`, height: `${EDGE}px`, cursor: 'ns-resize' } },
  { edge: 'West', label: '左边', style: { top: `${CORNER}px`, bottom: `${CORNER}px`, left: '0', width: `${EDGE}px`, cursor: 'ew-resize' } },
  { edge: 'East', label: '右边', style: { top: `${CORNER}px`, bottom: `${CORNER}px`, right: '0', width: `${EDGE}px`, cursor: 'ew-resize' } },
  { edge: 'NorthWest', label: '左上角', style: { top: '0', left: '0', width: `${CORNER}px`, height: `${CORNER}px`, cursor: 'nwse-resize' } },
  { edge: 'NorthEast', label: '右上角', style: { top: '0', right: '0', width: `${CORNER}px`, height: `${CORNER}px`, cursor: 'nesw-resize' } },
  { edge: 'SouthWest', label: '左下角', style: { bottom: '0', left: '0', width: `${CORNER}px`, height: `${CORNER}px`, cursor: 'nesw-resize' } },
  { edge: 'SouthEast', label: '右下角', style: { bottom: '0', right: '0', width: `${CORNER}px`, height: `${CORNER}px`, cursor: 'nwse-resize' } },
]

const appWindow = getCurrentWebviewWindow()
const enabled = ref(true)
let unlisten: (() => void) | null = null

/** 最大化 / 全屏时不留热区，避免抢走贴边点击 */
async function syncAvailability(): Promise<void> {
  const [maximized, fullscreen] = await Promise.all([
    appWindow.isMaximized().catch(() => false),
    appWindow.isFullscreen().catch(() => false),
  ])
  enabled.value = !maximized && !fullscreen
}

function beginResize(edge: ResizeEdge, event: MouseEvent): void {
  if (event.button !== 0) return
  event.preventDefault()
  void appWindow.startResizeDragging(edge).catch(() => undefined)
}

onMounted(async () => {
  await syncAvailability()
  unlisten = await appWindow.onResized(() => {
    void syncAvailability()
  })
})

onBeforeUnmount(() => {
  unlisten?.()
  unlisten = null
})
</script>

<template>
  <div v-if="enabled" class="pointer-events-none fixed inset-0 z-20000">
    <div
      v-for="zone in ZONES"
      :key="zone.edge"
      class="pointer-events-auto absolute"
      :style="zone.style"
      :aria-label="zone.label"
      @mousedown="beginResize(zone.edge, $event)"
    />
  </div>
</template>
