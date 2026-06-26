<script setup lang="ts">
/**
 * 顶部导航布局组件
 */

import { useRouter, useRoute } from 'vue-router'
import { useSdkStore } from '@/stores/sdk'
import {
  HomeIcon,
  Cog6ToothIcon,
  CubeIcon,
} from '@heroicons/vue/24/outline'

const router = useRouter()
const route = useRoute()
const sdkStore = useSdkStore()

const navItems = [
  { name: '首页', path: '/', icon: HomeIcon },
  { name: '下载', path: '/versions', icon: CubeIcon },
  { name: '设置', path: '/settings', icon: Cog6ToothIcon },
]

function isActive(path: string): boolean {
  return route.path === path
}

function navigateTo(path: string) {
  router.push(path)
}
</script>

<template>
  <div class="flex flex-col h-screen overflow-hidden bg-gray-50 dark:bg-gray-900">
    <!-- 顶部导航栏 - 固定 -->
    <header class="fixed top-0 left-0 right-0 z-50 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 shadow-sm">
      <div class="max-w-7xl mx-auto px-6 h-14 flex items-center justify-between">
        <!-- 左侧：Logo -->
        <div class="flex items-center w-48">
          <h1 class="text-lg font-bold text-primary-600 dark:text-primary-400">
            MoLaunch
          </h1>
        </div>
        
        <!-- 中间：导航菜单 -->
        <nav class="flex items-center space-x-1">
          <button
            v-for="item in navItems"
            :key="item.path"
            class="flex items-center px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            :class="[
              isActive(item.path)
                ? 'bg-primary-50 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300'
                : 'text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700'
            ]"
            @click="navigateTo(item.path)"
          >
            <component :is="item.icon" class="w-4 h-4 mr-2" />
            {{ item.name }}
          </button>
        </nav>
        
        <!-- 右侧：SDK 状态 -->
        <div class="flex items-center w-48 justify-end">
          <div class="flex items-center text-xs">
            <div
              class="w-2 h-2 rounded-full mr-1.5"
              :class="sdkStore.isReady ? 'bg-green-500' : 'bg-yellow-500'"
            />
            <span class="text-gray-500 dark:text-gray-400">
              {{ sdkStore.isReady ? 'SDK 就绪' : 'SDK 加载中' }}
            </span>
          </div>
        </div>
      </div>
    </header>
    
    <!-- 主内容区 -->
    <main class="flex-1 overflow-hidden pt-14">
      <slot />
    </main>
  </div>
</template>
