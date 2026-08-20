<script setup lang="ts">
/**
 * 资源包编辑器 - 资源包列表（可折叠 + 版本隔离筛选）
 */
import { defineAsyncComponent } from 'vue'
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import { formatBytes } from '@/utils/format'
import { ChevronDownIcon, FolderOpenIcon } from '@heroicons/vue/24/outline'
import type { ResourcePackItem } from '@/utils/api/tools'

defineProps<{
  packs: ResourcePackItem[]
  listOpen: boolean
  selectedVersion: string
  versionOptions: { label: string; value: string }[]
  opening: boolean
}>()

const emit = defineEmits<{
  open: [path: string]
  'update:list-open': [open: boolean]
  'update:version': [version: string]
}>()
</script>

<template>
  <div class="px-5 py-4">
    <!-- 资源包列表 -->
    <div class="flex flex-wrap items-center gap-2">
      <button
        class="flex items-center gap-1 text-sm text-gray-600 hover:text-gray-800"
        @click="emit('update:list-open', !listOpen)"
      >
        <ChevronDownIcon
          class="h-4 w-4 transition-transform"
          :class="listOpen ? '' : '-rotate-90'"
        />
        资源包列表
        <span class="text-xs text-gray-400">（{{ packs.length }}）</span>
      </button>
      <div class="ml-auto flex items-center gap-1.5">
        <span class="text-xs text-gray-400">版本</span>
        <Tooltip text="选择资源包隔离目录（按 MC 版本）">
          <Select
            :model-value="selectedVersion"
            :options="versionOptions"
            class="w-40"
            @update:model-value="emit('update:version', String($event))"
          />
        </Tooltip>
      </div>
    </div>
    <div v-show="listOpen" class="mt-2 grid max-h-[132px] grid-cols-2 gap-2 overflow-y-auto md:grid-cols-3">
      <button
        v-for="p in packs"
        :key="p.path"
        class="flex items-center gap-2 rounded border border-gray-200 px-3 py-2 text-left text-sm text-gray-700 hover:border-blue-400 hover:bg-blue-50"
        :disabled="opening"
        @click="emit('open', p.path)"
      >
        <FolderOpenIcon class="h-4 w-4 shrink-0 text-gray-400" />
        <span class="truncate">{{ p.name }}</span>
        <span class="ml-auto shrink-0 text-[10px] text-gray-400">{{ formatBytes(p.size) }}</span>
      </button>
      <p v-if="!packs.length" class="col-span-full py-4 text-center text-sm text-gray-400">
        暂无资源包，可点击「打开 ZIP / 打开文件夹」载入
      </p>
    </div>
  </div>
</template>