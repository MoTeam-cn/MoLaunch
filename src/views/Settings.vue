<script setup lang="ts">
import { ref } from 'vue'
import SettingsLaunch from './settings/SettingsLaunch.vue'
import SettingsDownload from './settings/SettingsDownload.vue'
import SettingsPersonal from './settings/SettingsPersonal.vue'
import SettingsOther from './settings/SettingsOther.vue'
import {
  RocketLaunchIcon,
  PaintBrushIcon,
  EllipsisHorizontalIcon,
  ArrowDownTrayIcon,
} from '@heroicons/vue/24/outline'

const activeCategory = ref('launch')

const categories = [
  { id: 'launch', label: '游戏启动', icon: RocketLaunchIcon, desc: 'Java、内存、游戏目录等启动参数' },
  { id: 'download', label: '下载配置', icon: ArrowDownTrayIcon, desc: '下载源、限速、并发等下载配置' },
  { id: 'personal', label: '个性化', icon: PaintBrushIcon, desc: '主题、布局、语言等外观设置' },
  { id: 'other', label: '其他', icon: EllipsisHorizontalIcon, desc: '日志、SDK 信息' },
]

const activeDesc = () => categories.find(c => c.id === activeCategory.value)?.desc ?? ''
</script>

<template>
  <div class="flex h-full rounded-xl overflow-hidden bg-white shadow-sm">
    <!-- 左侧分类菜单 -->
    <aside class="w-48 bg-white border-r border-gray-200 flex flex-col shrink-0">
      <div class="flex-1 overflow-y-auto py-4">
        <button
          v-for="cat in categories"
          :key="cat.id"
          class="w-full flex items-center px-4 py-2.5 text-sm font-medium transition-colors"
          :class="[
            activeCategory === cat.id
              ? 'bg-primary-50 text-primary-700 border-r-2 border-primary-500'
              : 'text-gray-700 hover:bg-gray-50'
          ]"
          @click="activeCategory = cat.id"
        >
          <component :is="cat.icon" class="w-5 h-5 mr-3" />
          {{ cat.label }}
        </button>
      </div>
    </aside>

    <!-- 右侧内容区 -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <div class="px-6 py-4 bg-white border-b border-gray-200 shrink-0">
        <h2 class="text-lg font-semibold text-gray-900">
          {{ categories.find(c => c.id === activeCategory)?.label }}
        </h2>
        <p class="text-xs text-gray-500 mt-1">{{ activeDesc() }}</p>
      </div>

      <div class="flex-1 overflow-y-auto p-6">
        <SettingsLaunch v-if="activeCategory === 'launch'" />
        <SettingsDownload v-else-if="activeCategory === 'download'" />
        <SettingsPersonal v-else-if="activeCategory === 'personal'" />
        <SettingsOther v-else-if="activeCategory === 'other'" />
      </div>
    </div>
  </div>
</template>
