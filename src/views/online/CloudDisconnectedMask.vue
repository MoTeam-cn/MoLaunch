<script setup lang="ts">
/**
 * 云端连接失败空状态遮罩
 *
 * 从 Online.vue 抽离：云端未连接且未在房间时整页空状态，
 * 阻止通过 URL 直接访问绕过 TopNavLayout 禁用。
 *
 * 显示条件由父组件 Online.vue 通过 v-if 控制（与下方主内容区 v-else 配对），
 * 本组件仅负责遮罩自身的样式与文案，不重复判断 cloudConnected / initializing / isInRoom。
 *
 * 复用项目自定义 Button 组件，不使用原生 HTML。
 */
import { CloudIcon, Cog6ToothIcon } from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'

defineProps<{
  /** 云端错误信息（cloudConnected=false 时非空，用于遮罩正文展示） */
  cloudError: string
}>()

defineEmits<{
  (e: 'goSettings'): void
}>()
</script>

<template>
  <div
    class="flex flex-col items-center justify-center h-full rounded-xl bg-white shadow-sm p-8"
  >
    <CloudIcon class="w-12 h-12 text-gray-300 mb-4" />
    <p class="text-sm font-medium text-gray-900">云端连接失败</p>
    <p class="text-xs text-gray-500 mt-2 text-center max-w-sm">
      {{ cloudError || '与云端 API 连接失败，联机功能暂不可用。' }}
    </p>
    <p class="text-xs text-gray-400 mt-1">可在「联机设置」页尝试重新连接</p>
    <Button type="outline" size="small" class="mt-4" @click="$emit('goSettings')">
      <template #icon><Cog6ToothIcon class="w-4 h-4" /></template>
      打开联机设置
    </Button>
  </div>
</template>
