<script setup lang="ts">
/**
 * Mod 版本列表表格子组件
 *
 * 展示过滤后的版本列表，支持单选（点击行选中）。
 */
import Tooltip from '@/components/common/Tooltip.vue'
import { formatBytes, formatDate } from '@/utils/format'
import { releaseTypeClass } from '@/composables/useModUpdate'
import type { ResourceVersion } from '@/types/community'
import { CheckCircleIcon } from '@heroicons/vue/24/outline'

defineProps<{
  versions: ResourceVersion[]
  selectedId: string | null
}>()

const emit = defineEmits<{
  'update:selectedId': [val: string]
}>()

function selectRow(id: string) {
  emit('update:selectedId', id)
}
</script>

<template>
  <div class="border border-gray-200 rounded-lg overflow-hidden">
    <div class="max-h-80 overflow-y-auto">
      <table class="w-full text-sm">
        <thead class="sticky top-0 bg-gray-50 text-xs text-gray-500">
          <tr>
            <th class="w-8 px-2 py-2"></th>
            <th class="px-2 py-2 text-left">文件名</th>
            <th class="px-2 py-2 text-left">发布日期</th>
            <th class="px-2 py-2 text-left">类型</th>
            <th class="px-2 py-2 text-right">大小</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100">
          <tr
            v-for="ver in versions"
            :key="ver.id"
            class="cursor-pointer transition-colors"
            :class="selectedId === ver.id ? 'bg-blue-50' : 'hover:bg-gray-50'"
            @click="selectRow(ver.id)"
          >
            <td class="px-2 py-2 text-center">
              <CheckCircleIcon
                v-if="selectedId === ver.id"
                class="w-4 h-4 text-blue-500 inline-block"
              />
            </td>
            <td class="px-2 py-2">
              <Tooltip
                v-if="ver.file_name.length > 28"
                :text="ver.file_name"
                position="top"
                :delay="200"
              >
                <div class="text-gray-800 truncate max-w-[260px] cursor-help">{{ ver.file_name }}</div>
              </Tooltip>
              <div v-else class="text-gray-800 truncate max-w-[260px]">{{ ver.file_name }}</div>
            </td>
            <td class="px-2 py-2 text-xs text-gray-500">{{ formatDate(ver.release_date) }}</td>
            <td class="px-2 py-2">
              <span class="text-[10px] px-1.5 py-0.5 rounded font-medium" :class="releaseTypeClass(ver.release_type)">
                {{ ver.release_type }}
              </span>
            </td>
            <td class="px-2 py-2 text-xs text-gray-500 text-right">{{ formatBytes(ver.size, 1) }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
