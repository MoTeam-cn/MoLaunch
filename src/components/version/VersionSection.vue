<script setup lang="ts">
/**
 * 可展开的版本分类组件
 */

import { ref, type Component } from 'vue'
import { ChevronRightIcon, TrashIcon, ArrowDownTrayIcon } from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Tag from '@/components/common/Tag.vue'

interface VersionItem {
  id: string
  version_type: string
  release_time: number
  tag?: string
  description?: string
}

interface Props {
  id: string
  label: string
  icon: Component
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
            class="flex items-center justify-between px-4 py-2.5 hover:bg-gray-50 transition-colors cursor-pointer"
            @click="!downloading && emit('download', version.id)"
          >
            <div class="flex items-center pl-8">
              <img
                :src="getVersionIcon(version.id, version.version_type)"
                :alt="version.id"
                class="w-6 h-6 rounded mr-2"
              />
              <div>
                <div class="flex items-center">
                  <span class="text-sm text-gray-900">{{ version.description || version.id }}</span>
                  <Tag
                    v-if="version.tag"
                    size="small"
                    class="ml-2"
                    :color="version.tag === '正式版' ? 'green' : 'gold'"
                  >
                    {{ version.tag }}
                  </Tag>
                </div>
                <span class="text-xs text-gray-500">{{ formatDate(version.release_time) }}</span>
              </div>
            </div>

            <div class="flex items-center">
              <Tag
                v-if="isInstalled(version.id)"
                size="small"
                color="green"
                class="mr-2"
              >
                已安装
              </Tag>
              <Button
                type="primary"
                size="small"
                :disabled="downloading"
                @click.stop="emit('download', version.id)"
              >
                <template #icon><ArrowDownTrayIcon class="w-3.5 h-3.5" /></template>
                安装
              </Button>
              <!-- 保留原生 button：卸载按钮（px-2 py-1 text-xs + @click.stop），
                   Button.vue 的 scoped size 类固定 padding 会破坏紧凑尺寸 -->
              <button
                v-if="isInstalled(version.id)"
                class="flex items-center px-2 py-1 bg-red-100 text-red-700 text-xs rounded hover:bg-red-200 transition-colors ml-1"
                @click.stop="emit('uninstall', version.id)"
              >
                <TrashIcon class="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
