<script setup lang="ts">
/**
 * 封条遮罩：绝对定位于父容器，提示某区块因外部条件（如云端离线）被封存。
 *
 * 视觉为虚线红框 + 锁图标 + 原因说明；点击整块遮罩会 emit `request`，
 * 由父组件拦截弹窗（Modal）告知用户封存原因。
 */
import { LockClosedIcon } from '@heroicons/vue/24/outline'

defineProps<{
  /** 封存原因说明（显示在图标下方，可空） */
  reason?: string
}>()

defineEmits<{
  /** 用户点击封条（父组件用于弹窗告知原因） */
  request: []
}>()
</script>

<template>
  <div
    class="absolute inset-0 z-10 flex flex-col items-center justify-center gap-1.5 rounded-md border-2 border-dashed border-red-300 bg-gray-50/85 cursor-not-allowed select-none"
    @click="$emit('request')"
  >
    <div class="flex items-center gap-1 text-red-500">
      <LockClosedIcon class="h-4 w-4" />
      <span class="text-xs font-semibold">已封存</span>
    </div>
    <p v-if="reason" class="max-w-[240px] px-2 text-center text-[11px] leading-relaxed text-gray-500">
      {{ reason }}
    </p>
  </div>
</template>