<script setup lang="ts">
/**
 * 顶部导航布局组件 - PCL2 风格
 */

import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useSdkStore } from '@/stores/sdk'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const sdkStore = useSdkStore()

const showUserMenu = ref(false)

const navItems = [
  { name: '首页', path: '/' },
  { name: '版本', path: '/versions' },
  { name: '设置', path: '/settings' },
]

function isActive(path: string): boolean {
  return route.path === path
}

function navigateTo(path: string) {
  router.push(path)
}

function toggleUserMenu() {
  showUserMenu.value = !showUserMenu.value
}

function handleLogout() {
  authStore.logout()
  showUserMenu.value = false
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
            class="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            :class="[
              isActive(item.path)
                ? 'bg-primary-50 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300'
                : 'text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700'
            ]"
            @click="navigateTo(item.path)"
          >
            {{ item.name }}
          </button>
        </nav>
        
        <!-- 右侧：状态和用户 -->
        <div class="flex items-center space-x-4 w-48 justify-end">
          <!-- SDK 状态 -->
          <div class="hidden sm:flex items-center text-xs">
            <div
              class="w-2 h-2 rounded-full mr-1.5"
              :class="sdkStore.isReady ? 'bg-green-500' : 'bg-yellow-500'"
            />
            <span class="text-gray-500 dark:text-gray-400">
              {{ sdkStore.isReady ? '就绪' : '加载中' }}
            </span>
          </div>
          
          <!-- 用户菜单 -->
          <div v-if="authStore.isLoggedIn" class="relative">
            <button
              class="flex items-center space-x-2 p-1.5 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
              @click="toggleUserMenu"
            >
              <div class="w-7 h-7 rounded-full bg-primary-100 dark:bg-primary-900 flex items-center justify-center">
                <span class="text-primary-600 dark:text-primary-400 font-semibold text-xs">
                  {{ authStore.username.charAt(0).toUpperCase() }}
                </span>
              </div>
              <svg class="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
              </svg>
            </button>
            
            <!-- 下拉菜单 -->
            <transition
              enter-active-class="transition ease-out duration-100"
              enter-from-class="transform opacity-0 scale-95"
              enter-to-class="transform opacity-100 scale-100"
              leave-active-class="transition ease-in duration-75"
              leave-from-class="transform opacity-100 scale-100"
              leave-to-class="transform opacity-0 scale-95"
            >
              <div
                v-if="showUserMenu"
                class="absolute right-0 mt-2 w-48 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-50"
              >
                <div class="px-4 py-2 border-b border-gray-200 dark:border-gray-700">
                  <p class="text-sm font-medium text-gray-900 dark:text-gray-100">
                    {{ authStore.username }}
                  </p>
                  <p class="text-xs text-gray-500 dark:text-gray-400">
                    离线模式
                  </p>
                </div>
                <button
                  class="w-full text-left px-4 py-2 text-sm text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/50"
                  @click="handleLogout"
                >
                  登出
                </button>
              </div>
            </transition>
          </div>
          
          <router-link
            v-else
            to="/login"
            class="btn-primary text-xs px-3 py-1.5"
          >
            登录
          </router-link>
        </div>
      </div>
    </header>
    
    <!-- 主内容区 -->
    <main class="flex-1 overflow-hidden pt-14">
      <slot />
    </main>
  </div>
</template>
