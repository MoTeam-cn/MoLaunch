<script setup lang="ts">
/**
 * 全局拖拽遮蔽层
 *
 * 由 useDragDrop 的 dragState 驱动，在文件拖入窗口时显示全屏蒙层，
 * 提示用户当前拖拽类型与释放动作。参考 PCL2 / 其他启动器的拖拽视觉反馈。
 *
 * 行为：
 * - 拖入 enter 时显示，leave/drop 时隐藏
 * - 根据 kind 显示不同图标和主标题
 * - 中央虚线大卡片 + 半透明背景 + backdrop-blur，视觉柔和
 *
 * 用法：在 App.vue 顶层直接渲染 <DragOverlay />，组件内部自管理显隐。
 */

import { computed } from 'vue'
import {
  ArrowDownTrayIcon,
  ArchiveBoxIcon,
  CubeIcon,
  ExclamationCircleIcon,
} from '@heroicons/vue/24/outline'
import { useDragDropState } from '@/composables/useDragDrop'

const dragState = useDragDropState()

const kindConfig = {
  modpack: {
    icon: ArchiveBoxIcon,
    title: '安装整合包',
    iconColor: 'text-primary-500',
    borderColor: 'border-primary-300',
    bgColor: 'bg-primary-50',
  },
  mod: {
    icon: CubeIcon,
    title: '安装 Mod',
    iconColor: 'text-emerald-500',
    borderColor: 'border-emerald-300',
    bgColor: 'bg-emerald-50',
  },
  'multi-mod': {
    icon: CubeIcon,
    title: '批量安装 Mod',
    iconColor: 'text-emerald-500',
    borderColor: 'border-emerald-300',
    bgColor: 'bg-emerald-50',
  },
  unknown: {
    icon: ExclamationCircleIcon,
    title: '不支持的文件',
    iconColor: 'text-amber-500',
    borderColor: 'border-amber-300',
    bgColor: 'bg-amber-50',
  },
} as const

const config = computed(() => kindConfig[dragState.kind])
</script>

<template>
  <Teleport to="body">
    <Transition name="drag-overlay">
      <div
        v-if="dragState.active"
        class="fixed inset-0 z-[10001] flex items-center justify-center bg-black/40 backdrop-blur-sm"
      >
        <div
          class="flex w-[420px] max-w-[80vw] flex-col items-center gap-4 rounded-2xl border-2 border-dashed bg-white/95 p-10 shadow-2xl"
          :class="config.borderColor"
        >
          <div
            class="flex h-20 w-20 items-center justify-center rounded-full"
            :class="config.bgColor"
          >
            <component
              :is="config.icon"
              class="h-10 w-10"
              :class="config.iconColor"
              aria-hidden="true"
            />
          </div>
          <div class="flex flex-col items-center gap-1.5">
            <p class="text-base font-medium text-gray-800">{{ config.title }}</p>
            <p class="text-sm text-gray-500">{{ dragState.hint }}</p>
          </div>
          <div class="mt-1 flex items-center gap-1.5 text-xs text-gray-400">
            <ArrowDownTrayIcon class="h-3.5 w-3.5" aria-hidden="true" />
            <span>将文件拖到此处释放</span>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.drag-overlay-enter-active,
.drag-overlay-leave-active {
  transition: opacity 0.18s ease;
}

.drag-overlay-enter-active > div,
.drag-overlay-leave-active > div {
  transition:
    transform 0.18s ease,
    opacity 0.18s ease;
}

.drag-overlay-enter-from,
.drag-overlay-leave-to {
  opacity: 0;
}

.drag-overlay-enter-from > div,
.drag-overlay-leave-to > div {
  transform: scale(0.96);
  opacity: 0;
}
</style>
