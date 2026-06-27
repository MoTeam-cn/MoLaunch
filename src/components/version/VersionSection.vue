<script setup lang="ts">
/**
 * 可展开的版本分类组件
 */

import { ref } from 'vue'
import { ChevronRightIcon, PlayIcon, TrashIcon, ArrowDownTrayIcon } from '@heroicons/vue/24/outline'

interface VersionItem {
  id: string
  version_type: string
  release_time: number
  tag?: string
}

interface Props {
  id: string
  label: string
  icon: object
  versions: VersionItem[]
  installedIds: string[]
  downloading: boolean
  defaultExpanded?: boolean
  formatDate: (ts: number) => string
  getVersionIcon: (id: string, type: string) => string
}

const props = withDefaults(defineProps<Props>(), {
  defaultExpanded: false,
})
const emit = defineEmits<{
  download: [versionId: string]
  uninstall: [versionId: string]
}>()

const expanded = ref(props.defaultExpanded)

function toggle() {
  expanded.value = !expanded.value
}

function isInstalled(id: string): boolean {
  return props.installedIds.includes(id)
}
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-200 overflow-hidden">
    <!-- 标题栏 -->
    <div
      class="flex items-center justify-between px-4 py-3 cursor-pointer hover:bg-gray-50 transition-colors"
      @click="toggle"
    >
      <div class="flex items-center">
        <component :is="icon" class="w-5 h-5 mr-3 text-gray-500" />
        <span class="font-medium text-gray-900">{{ label }}</span>
        <span class="ml-2 text-xs text-gray-500">{{ versions.length }} 个版本</span>
      </div>
      <ChevronRightIcon
        class="w-5 h-5 text-gray-400 transition-transform duration-200"
        :class="{ 'rotate-90': expanded }"
      />
    </div>

    <!-- 版本列表 -->
    <div
      class="grid transition-all duration-500 ease-in-out"
      :style="{ gridTemplateRows: expanded ? '1fr' : '0fr' }"
    >
      <div class="overflow-hidden min-h-0">
        <div class="border-t border-gray-100 divide-y divide-gray-100">
          <div
            v-for="version in versions"
            :key="version.id"
            class="flex items-center justify-between px-4 py-2.5 hover:bg-gray-50 transition-colors"
          >
            <div class="flex items-center pl-8">
              <img
                :src="getVersionIcon(version.id, version.version_type)"
                :alt="version.id"
                class="w-6 h-6 rounded mr-2"
              />
              <div>
                <div class="flex items-center">
                  <span class="text-sm text-gray-900">{{ version.id }}</span>
                  <span
                    v-if="version.tag"
                    class="ml-2 text-xs px-1.5 py-0.5 rounded-full"
                    :class="version.tag === '正式版'
                      ? 'bg-green-100 text-green-800'
                      : 'bg-yellow-100 text-yellow-800'"
                  >
                    {{ version.tag }}
                  </span>
                </div>
                <span class="text-xs text-gray-500">{{ formatDate(version.release_time) }}</span>
              </div>
            </div>

            <div class="flex items-center">
              <span
                v-if="isInstalled(version.id)"
                class="text-xs px-2 py-0.5 rounded-full bg-green-100 text-green-800 mr-2"
              >
                已安装
              </span>
              <button
                v-if="isInstalled(version.id)"
                class="flex items-center px-3 py-1 bg-primary-600 text-white text-xs rounded hover:bg-primary-700 transition-colors mr-1"
              >
                <PlayIcon class="w-3.5 h-3.5 mr-1" />
                启动
              </button>
              <button
                v-if="isInstalled(version.id)"
                class="flex items-center px-2 py-1 bg-red-100 text-red-700 text-xs rounded hover:bg-red-200 transition-colors"
                @click="emit('uninstall', version.id)"
              >
                <TrashIcon class="w-3.5 h-3.5" />
              </button>
              <button
                v-else
                class="flex items-center px-3 py-1 bg-primary-600 text-white text-xs rounded hover:bg-primary-700 transition-colors disabled:opacity-50"
                :disabled="downloading"
                @click="emit('download', version.id)"
              >
                <ArrowDownTrayIcon class="w-3.5 h-3.5 mr-1" />
                安装
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
