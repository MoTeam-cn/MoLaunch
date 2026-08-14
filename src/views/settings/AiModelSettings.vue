<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Checkbox = defineAsyncComponent(() => import('@/components/common/Checkbox.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))

interface Props {
  remoteModels: string[]
  enabledModels: string[]
  defaultModel: string
  loadingModels: boolean
  defaultOptions: { label: string; value: string }[]
}

defineProps<Props>()
const emit = defineEmits<{
  load: []
  toggle: [model: string]
  'update:defaultModel': [model: string]
}>()
</script>

<template>
  <div class="px-5 py-4">
    <div class="flex items-center justify-between mb-2">
      <p class="text-sm font-medium text-gray-900">模型管理</p>
      <Button type="outline" size="mini" :loading="loadingModels" @click="emit('load')">
        加载模型
      </Button>
    </div>
    <p class="text-xs text-gray-500 mb-3">
      从服务端加载模型列表后，勾选需要启用的模型；未勾选的模型不会被使用。
    </p>

    <div v-if="remoteModels.length > 0" data-inner-scroll class="border border-gray-200 rounded-md max-h-44 overflow-y-auto p-1.5 space-y-0.5 mb-3">
      <label
        v-for="model in remoteModels"
        :key="model"
        class="flex items-center justify-between px-2 py-1 rounded hover:bg-gray-50 cursor-pointer"
      >
        <Checkbox :checked="enabledModels.includes(model)" @change="emit('toggle', model)">
          {{ model }}
        </Checkbox>
        <Tag v-if="defaultModel === model" color="primary" size="small">默认</Tag>
      </label>
    </div>
    <p v-else class="text-xs text-gray-400 py-2 mb-3">
      点击「加载模型」从当前服务地址拉取可用模型
    </p>

    <template v-if="enabledModels.length > 0">
      <p class="text-sm font-medium text-gray-900 mb-2">默认模型</p>
      <Select
        :model-value="defaultModel"
        :options="defaultOptions"
        placeholder="请选择默认模型"
        @update:model-value="emit('update:defaultModel', String($event))"
      />
      <p class="text-xs text-gray-500 mt-1.5">未指定模型时，崩溃分析默认使用该模型</p>
    </template>
  </div>
</template>
