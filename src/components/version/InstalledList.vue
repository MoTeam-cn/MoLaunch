<script setup lang="ts">
/**
 * 已安装版本列表组件
 */

import { CubeIcon, PlayIcon, TrashIcon } from '@heroicons/vue/24/outline'

function inferVersionType(id: string): string {
  if (/^\d{2}w\d{2}[a-z]/.test(id)) return 'snapshot'
  return 'release'
}

interface Props {
  versions: string[]
  getVersionIcon: (id: string, type: string) => string
}

const props = defineProps<Props>()
const emit = defineEmits<{
  uninstall: [versionId: string]
}>()
</script>

<template>
  <div class="space-y-2">
    <div v-if="versions.length === 0" class="flex items-center justify-center h-64">
      <div class="text-center">
        <CubeIcon class="w-16 h-16 text-gray-400 mx-auto" />
        <p class="text-gray-600 mt-4">暂未安装任何版本</p>
      </div>
    </div>
    <div
      v-for="versionId in versions"
      :key="versionId"
      class="bg-white rounded-lg border border-gray-200 p-4 flex items-center justify-between"
    >
      <div class="flex items-center">
        <img
          :src="getVersionIcon(versionId, inferVersionType(versionId))"
          :alt="versionId"
          class="w-10 h-10 rounded mr-3"
        />
        <div>
          <h3 class="font-semibold text-gray-900">{{ versionId }}</h3>
          <p class="text-xs text-gray-500">已安装</p>
        </div>
      </div>
      <div class="flex items-center space-x-2">
        <button class="flex items-center px-4 py-2 bg-primary-600 text-white text-sm rounded-lg hover:bg-primary-700 transition-colors">
          <PlayIcon class="w-4 h-4 mr-1" />
          启动
        </button>
        <button
          class="flex items-center px-3 py-2 bg-red-100 text-red-700 text-sm rounded-lg hover:bg-red-200 transition-colors"
          @click="emit('uninstall', versionId)"
        >
          <TrashIcon class="w-4 h-4 mr-1" />
          卸载
        </button>
      </div>
    </div>
  </div>
</template>
