<script setup lang="ts">
/**
 * pack.mcmeta 元信息只读表单（pack_format / 适用版本 / 描述）
 */
import { computed } from 'vue'
import { CubeIcon } from '@heroicons/vue/24/outline'

const props = defineProps<{
  packFormat: number | null
  mcVersion: string | null
  description: string | null
}>()

const rows = computed(() => [
  {
    label: 'pack_format',
    value: props.packFormat != null ? String(props.packFormat) : '（缺失）',
  },
  { label: '适用版本', value: props.mcVersion ?? '未知' },
  { label: '描述', value: props.description ?? '（缺失）' },
])
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center gap-2">
      <CubeIcon class="h-4 w-4 text-gray-500" />
      <h4 class="text-sm font-medium text-gray-700">pack.mcmeta</h4>
      <span class="text-xs text-gray-400">包元信息</span>
    </div>
    <div class="overflow-hidden rounded border border-gray-200">
      <div
        v-for="r in rows"
        :key="r.label"
        class="flex border-b border-gray-200 last:border-b-0"
      >
        <div class="w-32 shrink-0 bg-gray-50 px-3 py-2 text-xs text-gray-500">{{ r.label }}</div>
        <div class="px-3 py-2 text-sm text-gray-700">{{ r.value }}</div>
      </div>
    </div>
  </div>
</template>
