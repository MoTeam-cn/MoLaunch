<script setup lang="ts">
/**
 * 模组翻译 - 翻译设置区（右侧操作区）
 */
import { defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Checkbox = defineAsyncComponent(() => import('@/components/common/Checkbox.vue'))

defineProps<{
  modelOptions: { label: string; value: string }[]
}>()

const model = defineModel<string>('model', { default: '' })
const batchSize = defineModel<number>('batch-size', { default: 40 })
const generateModName = defineModel<boolean>('generate-mod-name', { default: true })
const repairEnabled = defineModel<boolean>('repair-enabled', { default: true })
const classTextEnabled = defineModel<boolean>('class-text-enabled', { default: true })
const emit = defineEmits<{ start: [] }>()

const batchOptions = [
  { label: '20 条/批（更稳）', value: 20 },
  { label: '40 条/批（推荐）', value: 40 },
  { label: '80 条/批（更快）', value: 80 },
]
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 p-5">
    <h3 class="text-sm font-semibold text-gray-900 mb-3">3. 翻译设置</h3>
    <div class="space-y-3">
      <div class="flex items-center gap-3">
        <span class="text-sm text-gray-500 w-16 shrink-0">模型</span>
        <Select v-model="model" :options="modelOptions" placeholder="选择翻译模型" />
      </div>
      <div class="flex items-center gap-3">
        <span class="text-sm text-gray-500 w-16 shrink-0">批次</span>
        <Select v-model="batchSize" :options="batchOptions" />
      </div>
      <div class="flex items-center gap-3">
        <span class="text-sm text-gray-500 w-16 shrink-0">选项</span>
        <div class="flex flex-col gap-2">
          <Checkbox v-model="generateModName">生成中文名</Checkbox>
          <Checkbox v-model="repairEnabled">质量回修</Checkbox>
          <Checkbox v-model="classTextEnabled">class 文本</Checkbox>
        </div>
      </div>
      <div class="pt-2">
        <Button type="primary" class="w-full" @click="emit('start')">开始翻译</Button>
      </div>
    </div>
  </div>
</template>