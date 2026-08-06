<script setup lang="ts">
/**
 * 全局拖拽遮蔽层
 *
 * 由 useDragDrop 的 dragState 驱动，在文件拖入窗口时显示蒙层，
 * 提示用户当前拖拽类型与释放动作。
 *
 * 行为：
 * - 拖入 enter 时显示，leave/drop 时隐藏
 * - 极简风格：四周虚线边框（颜色即检测结果：浅绿=可拖入 / 浅黄=待分析 / 红=不支持）
 *   + 居中上传图标与提示文案
 * - 以 absolute 定位铺满所在容器（挂载在 App.vue 内容容器 `div.relative.h-full` 内，
 *   即 nav 下方的内容区），物理上不覆盖顶部 nav，顶部虚线不会穿到 nav 区域
 * - 背景模糊加强，遮蔽罩下方内容不可见；pointer-events-none 不拦截拖放事件
 *
 * 用法：在 App.vue 内容容器内渲染 <DragOverlay />，组件内部自管理显隐。
 */

import { computed } from 'vue'
import { ArrowUpTrayIcon } from '@heroicons/vue/24/outline'
import { useDragDropState } from '@/composables/useDragDrop'

const dragState = useDragDropState()

/** 检测结果 → 虚线边框颜色（浅绿 / 浅黄 / 红） */
const borderColor = computed(() => {
  switch (dragState.status) {
    case 'accept':
      return 'border-emerald-300'
    case 'reject':
      return 'border-red-400'
    default:
      return 'border-yellow-300'
  }
})
</script>

<template>
  <Transition name="drag-overlay">
    <div
      v-if="dragState.active"
      class="pointer-events-none absolute inset-0 z-50 bg-black/45 backdrop-blur-md"
    >
      <div
        class="absolute inset-3 rounded-xl border-2 border-dashed sm:inset-5"
        :class="borderColor"
      />
      <div class="absolute inset-0 flex flex-col items-center justify-center gap-3 text-white">
        <ArrowUpTrayIcon class="h-10 w-10" aria-hidden="true" />
        <p class="text-sm text-white/90">{{ dragState.hint }}</p>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.drag-overlay-enter-active,
.drag-overlay-leave-active {
  transition: opacity 0.15s ease;
}

.drag-overlay-enter-from,
.drag-overlay-leave-to {
  opacity: 0;
}
</style>
