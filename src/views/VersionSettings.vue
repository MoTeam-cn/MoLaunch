<script setup lang="ts">
/**
 * 版本设置页（主容器）
 *
 * 左侧导航：概览 / 设置 / Mod 管理 / 资源包 / 光影 / 导出
 * 右侧根据当前分类渲染对应子组件
 * 共享状态通过 useVersionSettings composable 单例管理
 */
import { ref, onMounted, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import {
  Squares2X2Icon,
  Cog6ToothIcon,
  PuzzlePieceIcon,
  PaintBrushIcon,
  SparklesIcon,
  ArrowUpTrayIcon,
} from '@heroicons/vue/24/outline'
import { useVersionStore } from '@/stores/version'
import { useVersionSettings } from '@/composables/useVersionSettings'
import Button from '@/components/common/Button.vue'
import NavSidebar from '@/components/common/NavSidebar.vue'
import OverviewTab from './version-settings/OverviewTab.vue'
import SetupTab from './version-settings/SetupTab.vue'
import ModTab from './version-settings/ModTab.vue'
import PackTab from './version-settings/PackTab.vue'
import ExportTab from './version-settings/ExportTab.vue'

const router = useRouter()
const route = useRoute()
const versionStore = useVersionStore()
const { selectedId, initContext } = useVersionSettings()

const activeCategory = ref('overview')

const categories = [
  { id: 'overview', label: '概览', icon: Squares2X2Icon, desc: '版本信息、文件夹快捷方式、高级管理' },
  { id: 'setup', label: '设置', icon: Cog6ToothIcon, desc: '版本独立的 Java、内存、窗口等启动参数' },
  { id: 'mod', label: 'Mod 管理', icon: PuzzlePieceIcon, desc: '管理当前版本的 Mod' },
  { id: 'resourcepack', label: '资源包', icon: PaintBrushIcon, desc: '管理当前版本的资源包' },
  { id: 'shader', label: '光影', icon: SparklesIcon, desc: '管理当前版本的光影' },
  { id: 'export', label: '导出', icon: ArrowUpTrayIcon, desc: '导出整合包或版本' },
]

const currentCategory = () => categories.find(c => c.id === activeCategory.value)

function goBack() {
  router.push('/apps')
}

// 路由 query 中的版本 ID 与 store 双向同步：
// - 进入页面时：优先用 query.id，其次用 store.selectedVersion，最后从 config.ini 恢复
// - store 变化时：同步到 query（刷新页面后 URL 仍带 id，可直接恢复）
watch(selectedId, (val) => {
  const currentQueryId = route.query.id as string | undefined
  if (val && val !== currentQueryId) {
    router.replace({ query: { ...route.query, id: val } })
  } else if (!val && currentQueryId) {
    // selectedVersion 被清空时移除 query.id
    const { id: _id, ...rest } = route.query
    router.replace({ query: rest })
  }
}, { immediate: true })

onMounted(async () => {
  // 1. 优先从 URL query 读取版本 ID（支持刷新页面、分享链接）
  const queryId = route.query.id as string | undefined
  if (queryId && queryId !== versionStore.selectedVersion) {
    versionStore.selectedVersion = queryId
  }

  // 2. 若仍无选中版本，尝试从 config.ini 恢复（刷新 /apps/versions/setup 时不会经过 Home.vue）
  if (!versionStore.selectedVersion) {
    await versionStore.restoreSelectedVersion()
  }

  // 3. 初始化版本上下文（gameDir / effectiveDir / personalization 等）
  await initContext()
})
</script>

<template>
  <div class="flex h-full flex-col overflow-hidden">
    <!-- 顶部栏 -->
    <header class="flex flex-none items-center justify-between border-b border-gray-200 bg-white px-4 py-3">
      <div class="flex items-center gap-3">
        <Button
          type="ghost"
          size="small"
          @click="goBack"
        >
          <template #icon>
            <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M12.7 4.3a1 1 0 010 1.4L8.4 10l4.3 4.3a1 1 0 01-1.4 1.4l-5-5a1 1 0 010-1.4l5-5a1 1 0 011.4 0z" clip-rule="evenodd" />
            </svg>
          </template>
          返回
        </Button>
        <h1 class="text-base font-semibold text-gray-800">
          版本设置<span v-if="selectedId" class="text-gray-400"> - {{ selectedId }}</span>
        </h1>
      </div>
    </header>

    <!-- 未选择版本 -->
    <div v-if="!selectedId" class="flex flex-1 items-center justify-center">
      <div class="flex flex-col items-center gap-3 text-gray-400">
        <svg class="h-12 w-12" viewBox="0 0 24 24" fill="currentColor">
          <path d="M4 4a2 2 0 012-2h12a2 2 0 012 2v16a2 2 0 01-2 2H6a2 2 0 01-2-2V4zm2 0v4h12V4H6zm0 6v4h12v-4H6zm0 6v4h12v-4H6z" />
        </svg>
        <p class="text-sm">请先在主页选择一个版本</p>
        <Button
          type="primary"
          @click="router.push('/apps/versions/select')"
        >
          去选择版本
        </Button>
      </div>
    </div>

    <!-- 主体：左导航 + 右内容 -->
    <div v-else class="flex flex-1 overflow-hidden">
      <NavSidebar v-model="activeCategory" :categories="categories" />

      <!-- 右侧内容区 -->
      <div class="flex flex-1 flex-col overflow-hidden">
        <div class="flex-none border-b border-gray-200 bg-white px-6 py-4">
          <h2 class="text-lg font-semibold text-gray-900">
            {{ currentCategory()?.label }}
          </h2>
          <p class="mt-1 text-xs text-gray-500">{{ currentCategory()?.desc }}</p>
        </div>

        <!-- Mod 管理 / 资源包 / 光影 / 导出页由各自组件自管布局（固定工具栏/底栏 + 内部滚动），
             其他 tab 共用外层滚动容器 -->
        <div
          class="flex-1 overflow-hidden"
          :class="(activeCategory === 'mod' || activeCategory === 'resourcepack'
            || activeCategory === 'shader' || activeCategory === 'export') ? 'flex flex-col' : 'overflow-y-auto p-6'"
        >
          <OverviewTab v-if="activeCategory === 'overview'" />
          <SetupTab v-else-if="activeCategory === 'setup'" />
          <ModTab v-else-if="activeCategory === 'mod'" />
          <PackTab v-else-if="activeCategory === 'resourcepack'" kind="resourcepack" />
          <PackTab v-else-if="activeCategory === 'shader'" kind="shader" />
          <ExportTab v-else-if="activeCategory === 'export'" />
        </div>
      </div>
    </div>
  </div>
</template>
