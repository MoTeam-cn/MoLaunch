<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import SettingsLaunch from './settings/SettingsLaunch.vue'
import SettingsDownload from './settings/SettingsDownload.vue'
import SettingsPersonal from './settings/SettingsPersonal.vue'
import SettingsAdvanced from './settings/SettingsAdvanced.vue'
import SettingsPlugins from './settings/SettingsPlugins.vue'
import SettingsCache from './settings/SettingsCache.vue'
import SettingsDeveloper from './settings/SettingsDeveloper.vue'
import SettingsMore from './settings/SettingsMore.vue'
import NavSidebar from '@/components/common/NavSidebar.vue'
import { getConfigMap } from '@/utils/api/config'
import {
  RocketLaunchIcon,
  PaintBrushIcon,
  ArrowDownTrayIcon,
  CogIcon,
  CommandLineIcon,
  InformationCircleIcon,
  PuzzlePieceIcon,
  CircleStackIcon,
} from '@heroicons/vue/24/outline'
import { safeCall } from '@/utils/async'

const activeCategory = ref('launch')

// 基础菜单项（始终显示）
const baseCategories = [
  { id: 'launch', label: '游戏启动', icon: RocketLaunchIcon, desc: 'Java、内存、游戏目录等启动参数' },
  { id: 'download', label: '下载配置', icon: ArrowDownTrayIcon, desc: '下载源、限速、并发等下载配置' },
  { id: 'personal', label: '个性化', icon: PaintBrushIcon, desc: '主题、布局、语言、插件等外观设置' },
  { id: 'plugins', label: '插件', icon: PuzzlePieceIcon, desc: '管理启动器内置与外部插件' },
  { id: 'advanced', label: '进阶设置', icon: CogIcon, desc: '日志、代理、CurseForge、社区资源等' },
  { id: 'cache', label: '缓存管理', icon: CircleStackIcon, desc: '查看各缓存目录占用、文件数量与自动清理策略' },
  { id: 'about', label: '更多', icon: InformationCircleIcon, desc: '关于 MoLaunch、系统信息、鸣谢、教程、法律信息' },
]

// 开发者菜单项（仅在开发者模式开启时追加到末尾）
const developerCategory = {
  id: 'developer',
  label: '开发者',
  icon: CommandLineIcon,
  desc: '日志、缓存、存储信息、系统信息',
}

// 开发者模式开关状态（由「进阶设置」页开关控制）
const devModeEnabled = ref(false)

// 实际渲染的菜单列表：基础项 + （开发者模式开启时）开发者项
const categories = computed(() => {
  if (devModeEnabled.value) {
    return [...baseCategories, developerCategory]
  }
  return baseCategories
})

const activeDesc = () => categories.value.find(c => c.id === activeCategory.value)?.desc ?? ''

// 监听「进阶设置」页开发者模式开关变化，实时更新侧边菜单
// （DevModeToggle.vue 切换开关时派发 'developer-mode-changed' 自定义事件）
function onDevModeChanged(e: Event) {
  const detail = (e as CustomEvent).detail as boolean
  devModeEnabled.value = detail
  // 关闭开发者模式时若当前停留在 developer 分类，切回「更多」避免空白页
  if (!detail && activeCategory.value === 'developer') {
    activeCategory.value = 'about'
  }
}

onMounted(async () => {
  const config = await safeCall(() => getConfigMap(), 'check developer mode')
  if (config) devModeEnabled.value = config.developerMode
  window.addEventListener('developer-mode-changed', onDevModeChanged)
})

onUnmounted(() => {
  window.removeEventListener('developer-mode-changed', onDevModeChanged)
})
</script>

<template>
  <div class="flex h-full rounded-xl overflow-hidden bg-white shadow-sm">
    <!-- 左侧分类菜单（公共组件，tab 同步到 URL query） -->
    <NavSidebar v-model="activeCategory" :categories="categories" />

    <!-- 右侧内容区 -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <div class="px-6 py-4 bg-white border-b border-gray-200 shrink-0">
        <h2 class="text-lg font-semibold text-gray-900">
          {{ categories.find(c => c.id === activeCategory)?.label }}
        </h2>
        <p class="text-xs text-gray-500 mt-1">{{ activeDesc() }}</p>
      </div>

      <div
        class="flex-1 overflow-hidden"
        :class="[
          // about 子组件已自带 p-6 内边距，避免双重 padding
          activeCategory === 'about' ? '' : 'p-6',
          // cache 页面需要内部管理滚动（顶部固定+列表滑动），去掉 padding 让子组件自管理
          activeCategory === 'cache' ? '!p-0' : '',
          // 非 cache 页面统一由外部容器提供纵向滚动
          activeCategory !== 'cache' ? 'overflow-y-auto' : '',
        ]"
      >
        <SettingsLaunch v-if="activeCategory === 'launch'" />
        <SettingsDownload v-else-if="activeCategory === 'download'" />
        <SettingsPersonal v-else-if="activeCategory === 'personal'" />
        <SettingsPlugins v-else-if="activeCategory === 'plugins'" />
        <SettingsAdvanced v-else-if="activeCategory === 'advanced'" />
        <SettingsCache v-else-if="activeCategory === 'cache'" />
        <SettingsMore v-else-if="activeCategory === 'about'" />
        <SettingsDeveloper v-else-if="activeCategory === 'developer'" />
      </div>
    </div>
  </div>
</template>
