<script setup lang="ts">
/**
 * 加载器卡片通用组件
 * 展开/收起动画 + 版本列表 + 说明 + 兼容性禁用
 */

import { ref } from 'vue'
import { XMarkIcon, CheckIcon } from '@heroicons/vue/24/outline'

interface VersionItem {
  key: string
  label: string
  tags?: string[]
}

interface Props {
  id: string
  name: string
  icon?: string
  color: string
  description: string
  versions: VersionItem[]
  selected: string | null
  disabled?: boolean
  disabledReason?: string
  showVersions?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  icon: '',
  disabled: false,
  disabledReason: '',
  showVersions: true,
})

const emit = defineEmits<{
  select: [key: string | null]
  clear: []
}>()

const expanded = ref(false)

function toggle() {
  if (props.disabled || !props.showVersions) return
  expanded.value = !expanded.value
}

function select(key: string) {
  emit('select', props.selected === key ? null : key)
  expanded.value = false
}

function clear(e: Event) {
  e.stopPropagation()
  emit('clear')
}
</script>

<template>
  <div
    class="bg-white rounded-lg border overflow-hidden transition-colors"
    :class="disabled ? 'border-gray-200 opacity-60' : 'border-gray-300'"
  >
    <!-- 标题栏 -->
    <div
      class="flex items-center justify-between px-4 py-3 transition-colors"
      :class="disabled || !showVersions ? 'cursor-not-allowed' : 'cursor-pointer hover:bg-gray-50'"
      @click="toggle"
    >
      <div class="flex items-center gap-2 min-w-0">
        <img v-if="icon" :src="icon" class="w-5 h-5 rounded shrink-0" />
        <span class="text-sm font-medium text-gray-900 shrink-0">{{ name }}</span>
        <span
          v-if="selected"
          class="flex items-center gap-1 text-xs px-2 py-0.5 rounded-full font-medium"
          :class="`bg-${color}-100 text-${color}-700`"
        >
          {{ selected }}
          <button
            class="p-0.5 -mr-1 rounded-full transition-colors"
            :class="`hover:bg-${color}-200`"
            @click="clear"
          >
            <XMarkIcon class="w-3 h-3" />
          </button>
        </span>
        <span v-if="disabled && disabledReason" class="text-xs text-red-500 ml-1">{{ disabledReason }}</span>
      </div>
      <svg
        v-if="showVersions && versions.length > 0"
        class="w-4 h-4 text-gray-500 transition-transform duration-300 shrink-0"
        :class="{ 'rotate-180': expanded }"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
      </svg>
    </div>

    <!-- 展开内容 -->
    <div
      v-if="showVersions && versions.length > 0"
      class="grid transition-all duration-300 ease-in-out"
      :style="{ gridTemplateRows: expanded ? '1fr' : '0fr' }"
    >
      <div class="overflow-hidden min-h-0">
        <div class="border-t border-gray-200">
          <!-- 版本列表 -->
          <div class="max-h-48 overflow-y-auto p-3 space-y-1.5">
            <div
              v-for="ver in versions"
              :key="ver.key"
              class="flex items-center justify-between px-3 py-2 rounded-lg border-2 transition-all cursor-pointer"
              :class="selected === ver.key
                ? `border-${color}-400 bg-${color}-50 shadow-sm`
                : `border-gray-200 hover:border-${color}-300 hover:bg-${color}-50/50`"
              @click="select(ver.key)"
            >
              <div class="flex items-center gap-2">
                <span
                  class="text-sm px-2 py-0.5 rounded-full"
                  :class="selected === ver.key
                    ? `bg-${color}-100 text-${color}-800 font-medium`
                    : 'bg-gray-100 text-gray-700'"
                >
                  {{ ver.label }}
                </span>
                <span
                  v-for="tag in ver.tags"
                  :key="tag"
                  class="text-xs px-1.5 py-0.5 rounded font-medium"
                  :class="tag === '推荐' || tag === '稳定版' || tag === '最新版'
                    ? 'bg-green-100 text-green-700'
                    : tag === '测试版' || tag === '预览版'
                    ? 'bg-yellow-100 text-yellow-700'
                    : 'bg-gray-100 text-gray-600'"
                >
                  {{ tag }}
                </span>
              </div>
              <CheckIcon v-if="selected === ver.key" class="w-4 h-4 shrink-0" :class="`text-${color}-500`" />
            </div>
          </div>
          <!-- 说明 -->
          <div
            class="px-3 py-2 border-t border-gray-100 text-xs"
            :class="`bg-${color}-50 text-${color}-700`"
          >
            {{ description }}
          </div>
        </div>
      </div>
    </div>

    <!-- 无版本时提示 -->
    <div v-if="versions.length === 0" class="px-4 pb-3 text-xs text-gray-400">
      暂无适用于此版本的 {{ name }}
    </div>
  </div>
</template>
