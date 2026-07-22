<script setup lang="ts">
/**
 * 已下载文件列表（icon + 文件名 + 大小/时间 + 删除按钮）
 */
import type { ExternalDownloadEntry } from '@/utils/api/tools'
import { formatBytes } from '@/utils/format'
import Tooltip from '@/components/common/Tooltip.vue'
import Button from '@/components/common/Button.vue'
import { DocumentIcon, ExclamationCircleIcon, TrashIcon } from '@heroicons/vue/24/outline'

defineProps<{
  files: ExternalDownloadEntry[]
}>()

const emit = defineEmits<{
  delete: [name: string]
}>()

function formatTime(unix: number): string {
  if (!unix) return ''
  const d = new Date(unix * 1000)
  const month = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  const hours = String(d.getHours()).padStart(2, '0')
  const mins = String(d.getMinutes()).padStart(2, '0')
  return `${month}-${day} ${hours}:${mins}`
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center justify-between px-5 pt-5 pb-3">
      <h3 class="text-sm font-semibold text-gray-900">已下载文件</h3>
      <span class="rounded-full bg-gray-100 px-2 py-0.5 text-xs font-medium text-gray-500">
        {{ files.length }}
      </span>
    </div>

    <!-- 空状态 -->
    <div v-if="files.length === 0" class="flex h-32 items-center justify-center">
      <div class="flex flex-col items-center gap-2 text-gray-400">
        <ExclamationCircleIcon class="h-8 w-8" />
        <span class="text-xs">暂无已下载文件</span>
      </div>
    </div>

    <!-- 文件列表 -->
    <ul v-else class="divide-y divide-gray-100">
      <li
        v-for="file in files"
        :key="file.name"
        class="flex items-center gap-3 px-5 py-3 transition-colors hover:bg-gray-50"
      >
        <DocumentIcon class="h-5 w-5 flex-none text-gray-400" />
        <div class="min-w-0 flex-1">
          <div class="truncate text-sm font-medium text-gray-900">{{ file.name }}</div>
          <div class="mt-0.5 text-xs text-gray-400">
            {{ formatBytes(file.size) }} · {{ formatTime(file.modified) }}
          </div>
        </div>
        <Tooltip text="删除">
          <Button
            type="ghost"
            size="mini"
            class="!h-7 !w-7 !p-0 text-gray-400 hover:!text-red-500"
            @click="emit('delete', file.name)"
          >
            <TrashIcon class="h-4 w-4" />
          </Button>
        </Tooltip>
      </li>
    </ul>
  </section>
</template>
