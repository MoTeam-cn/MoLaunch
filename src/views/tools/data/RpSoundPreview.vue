<script setup lang="ts">
/**
 * 声音 OGG 试听（data URI）
 */
import { SpeakerWaveIcon } from '@heroicons/vue/24/outline'

defineProps<{ src: string }>()
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center gap-2">
      <SpeakerWaveIcon class="h-4 w-4 text-gray-500" />
      <h4 class="text-sm font-medium text-gray-700">声音文件</h4>
      <span class="text-xs text-gray-400">OGG 音频试听</span>
    </div>
    <audio v-if="src" :src="src" controls class="w-full max-w-md" />
    <p v-else class="text-sm text-gray-400">音频加载失败</p>
  </div>
</template>
