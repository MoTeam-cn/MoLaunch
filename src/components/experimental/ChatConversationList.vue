<script setup lang="ts">
/**
 * AI 会话列表（左侧栏）
 *
 * - 拖动自由调宽：右侧边缘拖拽手柄，宽度范围 160~360px，持久化到 localStorage
 * - 折叠态 hover 自动展开：收起为窄条后，鼠标移入自动展开、移出自动收起
 */
import { onBeforeUnmount, ref } from 'vue'
import {
  ChatBubbleLeftRightIcon,
  ChevronDoubleLeftIcon,
  ChevronDoubleRightIcon,
  PlusIcon,
  TrashIcon,
  LockClosedIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import type { ConversationItem } from '@/utils/api/experimental'

defineProps<{
  conversations: ConversationItem[]
  activeId: number
  creating: boolean
}>()

const emit = defineEmits<{
  select: [id: number]
  create: []
  remove: [conv: ConversationItem]
}>()

const WIDTH_KEY = 'experimental.chat.convListWidth'
const MIN_WIDTH = 160
const MAX_WIDTH = 360
const COLLAPSED_WIDTH = 36

/** 展开宽度（px，可拖动调节） */
const width = ref(clamp(readStoredWidth(), MIN_WIDTH, MAX_WIDTH))
/** 是否折叠为窄条（收起后 hover 自动展开、移出自动收起） */
const collapsed = ref(false)
/** 鼠标是否停留在会话列表区域（折叠态 hover 展开的依据） */
const hovered = ref(false)
/** 是否正在拖动调宽 */
const dragging = ref(false)

const asideEl = ref<HTMLElement | null>(null)
let startX = 0
let startWidth = MIN_WIDTH

function readStoredWidth(): number {
  try {
    const v = Number(localStorage.getItem(WIDTH_KEY))
    return Number.isFinite(v) && v > 0 ? v : 224
  } catch {
    return 224
  }
}

function clamp(v: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, v))
}

/** 有效展开态：未折叠，或折叠中但鼠标停留（hover 自动展开） */
const effectiveExpanded = () => !collapsed.value || hovered.value

function onDragMove(e: MouseEvent) {
  width.value = clamp(startWidth + e.clientX - startX, MIN_WIDTH, MAX_WIDTH)
}

function onDragEnd() {
  dragging.value = false
  document.body.style.userSelect = ''
  window.removeEventListener('mousemove', onDragMove)
  window.removeEventListener('mouseup', onDragEnd)
  try {
    localStorage.setItem(WIDTH_KEY, String(width.value))
  } catch {
    /* 忽略 localStorage 写入失败 */
  }
}

function startDrag(e: MouseEvent) {
  if (!effectiveExpanded()) return
  e.preventDefault()
  dragging.value = true
  startX = e.clientX
  startWidth = width.value
  document.body.style.userSelect = 'none'
  window.addEventListener('mousemove', onDragMove)
  window.addEventListener('mouseup', onDragEnd)
}

onBeforeUnmount(() => {
  if (dragging.value) onDragEnd()
})
</script>

<template>
  <aside
    ref="asideEl"
    class="relative shrink-0 border-r border-gray-200 bg-gray-50/60 transition-[width] duration-150"
    :class="effectiveExpanded() ? 'flex flex-col' : ''"
    :style="{ width: (collapsed && !hovered) ? COLLAPSED_WIDTH + 'px' : width + 'px' }"
    @mouseenter="hovered = true"
    @mouseleave="hovered = false"
  >
    <template v-if="collapsed && !hovered">
      <!-- 折叠态：垂直窄条，仅展开图标（hover 区域即自动展开） -->
      <div class="flex h-full flex-col items-center py-2.5">
        <Tooltip text="展开会话列表" position="right">
          <button
            type="button"
            class="rounded-md p-1 text-gray-400 transition-colors hover:bg-gray-100 hover:text-primary-500"
            @click="collapsed = false"
          >
            <ChevronDoubleRightIcon class="h-4 w-4" />
          </button>
        </Tooltip>
      </div>
    </template>

    <template v-else>
      <div class="flex items-center justify-between px-3 py-2.5">
        <span class="truncate text-sm font-semibold whitespace-nowrap text-gray-700">AI 会话</span>
        <div class="flex shrink-0 items-center gap-1">
          <Button type="primary" size="mini" :loading="creating" @click="emit('create')">
            <template #icon><PlusIcon class="h-3.5 w-3.5" /></template>
            新建
          </Button>
          <!-- hover 临时展开时提供「固定展开」：点击后保持展开，不再随鼠标移出收起 -->
          <Tooltip v-if="collapsed && hovered" text="固定展开" position="bottom">
            <button
              type="button"
              class="rounded-md p-1 text-primary-500 transition-colors hover:bg-primary-50"
              @click="collapsed = false"
            >
              <LockClosedIcon class="h-3.5 w-3.5" />
            </button>
          </Tooltip>
          <Tooltip text="收起会话列表">
            <button
              type="button"
              class="rounded-md p-1 text-gray-400 transition-colors hover:bg-gray-100 hover:text-primary-500"
              @click="collapsed = true"
            >
              <ChevronDoubleLeftIcon class="h-3.5 w-3.5" />
            </button>
          </Tooltip>
        </div>
      </div>
      <div class="min-h-0 flex-1 overflow-y-auto px-2 pb-2">
        <!-- TransitionGroup：新建/删除/切换会话时列表项平滑移动与淡入淡出 -->
        <TransitionGroup name="conv" tag="div" class="relative space-y-0.5">
          <div
            v-for="conv in conversations"
            :key="conv.id"
            class="group relative flex cursor-pointer items-center gap-2 rounded-md py-2.5 pr-8 pl-2.5 transition-colors"
            :class="conv.id === activeId ? 'bg-primary-100 text-primary-700' : 'text-gray-600 hover:bg-gray-100'"
            @click="emit('select', conv.id)"
          >
            <ChatBubbleLeftRightIcon class="h-4 w-4 shrink-0" />
            <Tooltip :text="conv.title" position="top" class="min-w-0 flex-1">
              <span class="block min-w-0 truncate text-xs">{{ conv.title }}</span>
            </Tooltip>
            <Tooltip text="删除会话">
              <button
                class="absolute top-1/2 right-1.5 hidden -translate-y-1/2 rounded p-1 text-gray-400 transition-colors hover:text-red-500 group-hover:block"
                @click.stop="emit('remove', conv)"
              >
                <TrashIcon class="h-3.5 w-3.5" />
              </button>
            </Tooltip>
          </div>
        </TransitionGroup>
        <div v-if="conversations.length === 0" class="flex flex-col items-center justify-center py-10 text-gray-400">
          <ChatBubbleLeftRightIcon class="mb-2 h-8 w-8" />
          <span class="text-xs">暂无会话，点击"新建"开始</span>
        </div>
      </div>
    </template>

    <!-- 右侧拖拽手柄（展开态可见）：拖动自由调节宽度 -->
    <div
      v-if="effectiveExpanded()"
      class="absolute inset-y-0 right-0 z-10 w-1.5 cursor-col-resize transition-colors"
      :class="dragging ? 'bg-primary-400' : 'hover:bg-primary-200'"
      @mousedown="startDrag"
    />
  </aside>
</template>

<style scoped>
/* 会话列表项增删/移动过渡：新建、删除、切换时列表平滑变化 */
.conv-enter-active,
.conv-leave-active {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}

.conv-enter-from {
  opacity: 0;
  transform: translateY(-8px);
}

.conv-leave-to {
  opacity: 0;
  transform: translateY(8px);
}

/* 删除中的列表项脱离文档流，其余项平滑补位 */
.conv-leave-active {
  position: absolute;
  width: 100%;
}

.conv-move {
  transition: transform 0.2s ease;
}
</style>
