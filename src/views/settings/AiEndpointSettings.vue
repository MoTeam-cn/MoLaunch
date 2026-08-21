<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Slider = defineAsyncComponent(() => import('@/components/common/Slider.vue'))

interface Props {
  baseUrl: string
  apiKey: string
  timeoutSecs: number
  iconColorMode: 'color' | 'mono'
}

defineProps<Props>()
const emit = defineEmits<{
  'update:baseUrl': [value: string]
  'update:apiKey': [value: string]
  'update:timeoutSecs': [value: number]
  'update:iconColorMode': [value: 'color' | 'mono']
}>()

const iconModeOptions = [
  { label: '彩色', value: 'color' },
  { label: '黑白', value: 'mono' },
]
</script>

<template>
  <div class="divide-y divide-gray-200">
    <div class="px-5 py-4">
      <p class="text-sm font-medium text-gray-900 mb-2">服务地址</p>
      <Input
        :model-value="baseUrl"
        placeholder="http://127.0.0.1:11434/v1"
        hint="OpenAI 兼容 API 地址，例如 Ollama 默认 http://127.0.0.1:11434/v1"
        @update:model-value="emit('update:baseUrl', String($event))"
      />
    </div>

    <div class="px-5 py-4">
      <p class="text-sm font-medium text-gray-900 mb-2">API Key</p>
      <Input
        :model-value="apiKey"
        type="password"
        placeholder="留空表示无需认证"
        hint="写入时经 SDK 加密存储（config.ini），本地 Ollama 通常无需填写"
        @update:model-value="emit('update:apiKey', String($event))"
      />
    </div>

    <div class="px-5 py-4">
      <div class="flex items-center justify-between mb-2">
        <p class="text-sm font-medium text-gray-900">请求超时</p>
        <span class="text-sm font-medium text-primary-600">{{ timeoutSecs }} 秒</span>
      </div>
      <div class="flex items-center gap-3">
        <span class="text-xs text-gray-400">10</span>
        <Slider
          :model-value="timeoutSecs"
          :min="10"
          :max="300"
          :step="10"
          class="flex-1"
          @update:model-value="emit('update:timeoutSecs', $event)"
        />
        <span class="text-xs text-gray-400">300</span>
      </div>
      <p class="text-xs text-gray-500 mt-1.5">模型分析耗时可较长，默认 60 秒</p>
    </div>

    <div class="px-5 py-4">
      <p class="text-sm font-medium text-gray-900 mb-2">模型图标</p>
      <Select
        :model-value="iconColorMode"
        :options="iconModeOptions"
        @update:model-value="emit('update:iconColorMode', $event as 'color' | 'mono')"
      />
      <p class="text-xs text-gray-500 mt-1.5">
        彩色为品牌官方配色；黑白为单色图标。未识别的模型统一使用 HuggingFace 图标。
      </p>
    </div>
  </div>
</template>
