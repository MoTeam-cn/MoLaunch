<script setup lang="ts">
/**
 * 顶部导航布局组件
 */

import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useSdkStore } from '@/stores/sdk'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import {
  HomeIcon,
  Cog6ToothIcon,
  CubeIcon,
} from '@heroicons/vue/24/outline'
import * as tauri from '@/utils/tauri'
const appWindow = getCurrentWebviewWindow()

const router = useRouter()
const route = useRoute()
const sdkStore = useSdkStore()
const isMaximized = ref(false)
const unlistenResized = ref<(() => void) | null>(null)

onMounted(async () => {
  isMaximized.value = await appWindow.isMaximized()
  unlistenResized.value = await appWindow.onResized(async () => {
    isMaximized.value = await appWindow.isMaximized()
  })
})

onUnmounted(() => {
  if (unlistenResized.value) {
    unlistenResized.value()
    unlistenResized.value = null
  }
})

const navItems = [
  { name: '首页', path: '/', icon: HomeIcon },
  { name: '下载', path: '/versions', icon: CubeIcon, hasDblClick: true },
  { name: '设置', path: '/settings', icon: Cog6ToothIcon },
]

// 双击计时器
let lastClickTime = 0
let clickTimer: ReturnType<typeof setTimeout> | null = null

function isActive(path: string): boolean {
  return route.path === path
}

function navigateTo(path: string) {
  router.push(path)
}

function handleDownloadClick() {
  const now = Date.now()
  const timeDiff = now - lastClickTime
  
  if (timeDiff < 300) {
    // 双击：进入下载管理页面
    if (clickTimer) {
      clearTimeout(clickTimer)
      clickTimer = null
    }
    router.push('/downloads')
  } else {
    // 单击：延迟执行导航到版本页面
    clickTimer = setTimeout(() => {
      router.push('/versions')
      clickTimer = null
    }, 300)
  }
  
  lastClickTime = now
}

async function handleClose() {
  try {
    // 关闭窗口前先保存配置
    await tauri.saveConfigToFile()
  } catch (e) {
    console.error('Failed to save config before close:', e)
  }
  await appWindow.close()
}
</script>

<template>
  <div class="flex flex-col h-screen overflow-hidden">
    <!-- 顶部栏：蓝色背景 -->
    <header class="shrink-0 bg-primary-600 select-none">
      <div class="h-12 flex items-center">
        <!-- 左侧拖拽区域 -->
        <div data-tauri-drag-region class="flex items-center pl-4 pr-2 h-full">
          <span class="text-sm font-bold text-white pointer-events-none">MoLaunch</span>
        </div>

        <!-- 中间拖拽空隙 -->
        <div data-tauri-drag-region class="flex-1 h-full" />

        <!-- 导航菜单 -->
        <nav class="flex items-center space-x-1">
          <button
            v-for="item in navItems"
            :key="item.path"
            class="flex items-center px-4 py-1.5 rounded-md text-sm font-medium transition-colors"
            :class="[
              isActive(item.path) || (item.hasDblClick && isActive('/downloads'))
                ? 'bg-white/20 text-white'
                : 'text-white/70 hover:bg-white/10 hover:text-white'
            ]"
            @click="item.hasDblClick ? handleDownloadClick() : navigateTo(item.path)"
          >
            <component :is="item.icon" class="w-4 h-4 mr-1.5" />
            {{ item.name }}
          </button>
        </nav>

        <!-- 中间拖拽空隙 -->
        <div data-tauri-drag-region class="flex-1 h-full" />

        <!-- 右侧：SDK 状态 + 窗口控制 -->
        <div class="flex items-center h-full">
          <div data-tauri-drag-region class="flex items-center h-full px-2">
            <div class="flex items-center text-xs pointer-events-none">
              <div class="w-1.5 h-1.5 rounded-full mr-1.5" :class="sdkStore.isReady ? 'bg-green-400' : 'bg-yellow-300'" />
              <span class="text-white/60">{{ sdkStore.isReady ? '就绪' : '加载中' }}</span>
            </div>
          </div>

          <!-- 最小化 -->
          <button
            class="h-full w-11 flex items-center justify-center hover:bg-white/10 transition-colors group"
            @click="appWindow.minimize()"
          >
            <svg class="w-3.5 h-3.5 text-white/60 group-hover:text-white" viewBox="0 0 12 12" fill="none">
              <rect x="1" y="5.5" width="10" height="1" rx="0.5" fill="currentColor" />
            </svg>
          </button>

          <!-- 最大化/还原 -->
          <button
            class="h-full w-11 flex items-center justify-center hover:bg-white/10 transition-colors group"
            @click="appWindow.toggleMaximize()"
          >
            <svg v-if="!isMaximized" class="w-3.5 h-3.5 text-white/60 group-hover:text-white" viewBox="0 0 12 12" fill="none">
              <rect x="1.5" y="1.5" width="9" height="9" rx="1" stroke="currentColor" stroke-width="1" />
            </svg>
            <svg v-else class="w-3.5 h-3.5 text-white/60 group-hover:text-white" viewBox="0 0 12 12" fill="none">
              <rect x="3.5" y="0.5" width="7.5" height="7.5" rx="1" stroke="currentColor" stroke-width="1" />
              <path d="M1 3.5V10C1 10.5523 1.44772 11 2 11H8.5" stroke="currentColor" stroke-width="1" />
            </svg>
          </button>

          <!-- 关闭 -->
          <button
            class="h-full w-11 flex items-center justify-center hover:bg-red-500 transition-colors group"
            @click="handleClose"
          >
            <svg class="w-3.5 h-3.5 text-white/60 group-hover:text-white" viewBox="0 0 12 12" fill="none">
              <path d="M1.5 1.5L10.5 10.5M1.5 10.5L10.5 1.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
            </svg>
          </button>
        </div>
      </div>
    </header>
    
    <!-- 主内容区：淡蓝背景 -->
    <main class="flex-1 overflow-hidden p-2" style="background-color: #e0ecff">
      <slot />
    </main>
  </div>
</template>
