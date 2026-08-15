<script setup lang="ts">
/**
 * 语言文件键值表格（解析 lang/*.json 原文）
 */
import { computed } from 'vue'
import { LanguageIcon } from '@heroicons/vue/24/outline'

const props = defineProps<{ content: string }>()

interface LangEntry {
  key: string
  value: string
}

const entries = computed<LangEntry[]>(() => {
  if (!props.content) return []
  try {
    const obj = JSON.parse(props.content) as Record<string, unknown>
    return Object.entries(obj).map(([k, v]) => ({
      key: k,
      value: typeof v === 'string' ? v : JSON.stringify(v),
    }))
  } catch {
    return []
  }
})
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center gap-2">
      <LanguageIcon class="h-4 w-4 text-gray-500" />
      <h4 class="text-sm font-medium text-gray-700">语言文件</h4>
      <span class="text-xs text-gray-400">{{ entries.length }} 条键值</span>
    </div>
    <div class="max-h-[440px] overflow-y-auto rounded border border-gray-200">
      <table class="w-full text-left text-xs">
        <thead class="sticky top-0 bg-gray-50 text-gray-500">
          <tr>
            <th class="px-3 py-2 font-medium">键</th>
            <th class="px-3 py-2 font-medium">值</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100">
          <tr v-for="e in entries" :key="e.key" class="align-top">
            <td class="px-3 py-1.5 font-mono text-gray-600">{{ e.key }}</td>
            <td class="px-3 py-1.5 text-gray-700">{{ e.value }}</td>
          </tr>
        </tbody>
      </table>
      <p v-if="!entries.length" class="px-3 py-6 text-center text-gray-400">
        解析失败或无内容
      </p>
    </div>
  </div>
</template>
