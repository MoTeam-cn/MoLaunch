<script setup lang="ts">
/**
 * 纹理 PNG 2D 预览（data URI）
 */
import { PhotoIcon } from '@heroicons/vue/24/outline'

defineProps<{
  src: string
  animated: boolean
  name: string
}>()
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center gap-2">
      <PhotoIcon class="h-4 w-4 text-gray-500" />
      <h4 class="text-sm font-medium text-gray-700">{{ name }}</h4>
      <span v-if="animated" class="rounded bg-purple-100 px-1.5 py-0.5 text-xs text-purple-600">
        动画纹理
      </span>
    </div>
    <div v-if="src" class="grid place-items-center rounded border border-gray-200 bg-gray-50 p-4">
      <img :src="src" class="max-h-96 max-w-full object-contain" alt="纹理预览" />
    </div>
    <p v-else class="text-sm text-gray-400">纹理加载失败</p>
  </div>
</template>
