<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import SettingsLaunch from './settings/SettingsLaunch.vue'
import SettingsDownload from './settings/SettingsDownload.vue'
import SettingsPersonal from './settings/SettingsPersonal.vue'
import SettingsAdvanced from './settings/SettingsAdvanced.vue'
import SettingsOther from './settings/SettingsOther.vue'
import SettingsDeveloper from './settings/SettingsDeveloper.vue'
import * as tauri from '@/utils/tauri'
import {
  RocketLaunchIcon,
  PaintBrushIcon,
  EllipsisHorizontalIcon,
  ArrowDownTrayIcon,
  CogIcon,
  CommandLineIcon,
} from '@heroicons/vue/24/outline'

const activeCategory = ref('launch')

// 基础菜单项（始终显示）
const baseCategories = [
  { id: 'launch', label: '游戏启动', icon: RocketLaunchIcon, desc: 'Java、内存、游戏目录等启动参数' },
  { id: 'download', label: '下载配置', icon: ArrowDownTrayIcon, desc: '下载源、限速、并发等下载配置' },
  { id: 'personal', label: '个性化', icon: PaintBrushIcon, desc: '主题、布局、语言等外观设置' },
  { id: 'advanced', label: '高阶配置', icon: CogIcon, desc: '代理、高级参数等' },
  { id: 'other', label: '其他', icon: EllipsisHorizontalIcon, desc: '日志、SDK 信息' },
]

// 开发者菜单项（仅在开发者模式开启时追加到末尾）
const developerCategory = {
  id: 'developer',
  label: '开发者',
  icon: CommandLineIcon,
  desc: '日志、缓存、存储信息、系统信息',
}

// 开发者模式开关状态（由「高阶配置」开关控制）
const devModeEnabled = ref(false)

// 实际渲染的菜单列表：基础项 + （开发者模式开启时）开发者项
const categories = computed(() => {
  if (devModeEnabled.value) {
    return [...baseCategories, developerCategory]
  }
  return baseCategories
})

const activeDesc = () => categories.value.find(c => c.id === activeCategory.value)?.desc ?? ''

// 监听「高阶配置」页开发者模式开关变化，实时更新侧边菜单
// （SettingsAdvanced.vue 切换开关时派发 'developer-mode-changed' 自定义事件）
function onDevModeChanged(e: Event) {
  const detail = (e as CustomEvent).detail as boolean
  devModeEnabled.value = detail
  // 关闭开发者模式时若当前停留在 developer 分类，切回「其他」避免空白页
  if (!detail && activeCategory.value === 'developer') {
    activeCategory.value = 'other'
  }
}

onMounted(async () => {
  try {
    devModeEnabled.value = await tauri.isDeveloperMode()
  } catch (e) {
    console.error('Failed to check developer mode:', e)
  }
  window.addEventListener('developer-mode-changed', onDevModeChanged)
})

onUnmounted(() => {
  window.removeEventListener('developer-mode-changed', onDevModeChanged)
})
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
        <SettingsAdvanced v-else-if="activeCategory === 'advanced'" />
        <SettingsOther v-else-if="activeCategory === 'other'" />
        <SettingsDeveloper v-else-if="activeCategory === 'developer'" />
      </div>
    </div>
  </div>
</template>
