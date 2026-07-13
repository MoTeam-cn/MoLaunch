<script setup lang="ts">
/**
 * 版本设置页（主容器）
 *
 * 左侧导航：概览 / 设置 / Mod 管理 / 导出
 * 右侧根据当前分类渲染对应子组件
 * 共享状态通过 useVersionSettings composable 单例管理
 */
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import {
  Squares2X2Icon,
  Cog6ToothIcon,
  PuzzlePieceIcon,
  ArrowUpTrayIcon,
} from '@heroicons/vue/24/outline'
import { useVersionSettings } from '@/composables/useVersionSettings'
import OverviewTab from './version-settings/OverviewTab.vue'
import SetupTab from './version-settings/SetupTab.vue'
import ModTab from './version-settings/ModTab.vue'

const router = useRouter()
const { selectedId, initContext } = useVersionSettings()

const activeCategory = ref('overview')

const categories = [
  { id: 'overview', label: '概览', icon: Squares2X2Icon, desc: '版本信息、文件夹快捷方式、高级管理' },
  { id: 'setup', label: '设置', icon: Cog6ToothIcon, desc: '版本独立的 Java、内存、窗口等启动参数' },
  { id: 'mod', label: 'Mod 管理', icon: PuzzlePieceIcon, desc: '管理当前版本的 Mod' },
  { id: 'export', label: '导出', icon: ArrowUpTrayIcon, desc: '导出整合包或版本' },
]

const currentCategory = () => categories.find(c => c.id === activeCategory.value)

function goBack() {
  router.push('/')
}

onMounted(initContext)
</script>

<template>
  <div class="flex h-full flex-col overflow-hidden">
    <!-- 顶部栏 -->
    <header class="flex flex-none items-center justify-between border-b border-gray-200 bg-white px-4 py-3">
      <div class="flex items-center gap-3">
        <button
          class="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-sm text-gray-500 transition-colors hover:bg-gray-100 hover:text-gray-700"
          @click="goBack"
        >
          <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M12.7 4.3a1 1 0 010 1.4L8.4 10l4.3 4.3a1 1 0 01-1.4 1.4l-5-5a1 1 0 010-1.4l5-5a1 1 0 011.4 0z" clip-rule="evenodd" />
          </svg>
          返回
        </button>
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
        <button
          class="rounded-lg bg-primary-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-700"
          @click="router.push('/select')"
        >
          去选择版本
        </button>
      </div>
    </div>

    <!-- 主体：左导航 + 右内容 -->
    <div v-else class="flex flex-1 overflow-hidden">
      <aside class="w-48 flex-none border-r border-gray-200 bg-white">
        <div class="flex-1 overflow-y-auto py-4">
          <button
            v-for="cat in categories"
            :key="cat.id"
            class="flex w-full items-center px-4 py-2.5 text-sm font-medium transition-colors"
            :class="[
              activeCategory === cat.id
                ? 'bg-primary-50 text-primary-700 border-r-2 border-primary-500'
                : 'text-gray-700 hover:bg-gray-50'
            ]"
            @click="activeCategory = cat.id"
          >
            <component :is="cat.icon" class="mr-3 h-5 w-5" />
            {{ cat.label }}
          </button>
        </div>
      </aside>

      <!-- 右侧内容区 -->
      <div class="flex flex-1 flex-col overflow-hidden">
        <div class="flex-none border-b border-gray-200 bg-white px-6 py-4">
          <h2 class="text-lg font-semibold text-gray-900">
            {{ currentCategory()?.label }}
          </h2>
          <p class="mt-1 text-xs text-gray-500">{{ currentCategory()?.desc }}</p>
        </div>

        <div class="flex-1 overflow-y-auto p-6">
          <OverviewTab v-if="activeCategory === 'overview'" />
          <SetupTab v-else-if="activeCategory === 'setup'" />
          <ModTab v-else-if="activeCategory === 'mod'" />
          <!-- 导出子页（占位） -->
          <div v-else class="flex h-full items-center justify-center">
            <div class="flex flex-col items-center gap-3 text-gray-400">
              <component :is="currentCategory()?.icon" class="h-10 w-10" />
              <p class="text-sm">功能开发中</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
