<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))

interface Props {
  maxInputTokens: number
  maxOutputTokens: number
}

defineProps<Props>()
const emit = defineEmits<{
  'update:maxInputTokens': [value: number]
  'update:maxOutputTokens': [value: number]
}>()
</script>

<template>
  <div class="px-5 py-4">
    <p class="text-sm font-medium text-gray-900 mb-3">上下文窗口（Token）</p>
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label class="block text-xs text-gray-500 mb-1.5">输入上限（窗口）</label>
        <Input
          :model-value="maxInputTokens"
          type="number"
          min="2000"
          max="1000000"
          placeholder="184000"
          hint="接近此上限时自动压缩历史上下文"
          @update:model-value="emit('update:maxInputTokens', Number($event))"
        />
      </div>
      <div>
        <label class="block text-xs text-gray-500 mb-1.5">单次回复上限（输出）</label>
        <Input
          :model-value="maxOutputTokens"
          type="number"
          min="256"
          max="128000"
          placeholder="16000"
          hint="请求时作为 max_tokens 下发"
          @update:model-value="emit('update:maxOutputTokens', Number($event))"
        />
      </div>
    </div>
  </div>
</template>
