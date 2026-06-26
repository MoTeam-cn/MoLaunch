<script setup lang="ts">
/**
 * 顶部导航布局组件
 */

import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useSdkStore } from '@/stores/sdk'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const sdkStore = useSdkStore()

const showMobileMenu = ref(false)
const showUserMenu = ref(false)

const navItems = [
  { name: '首页', path: '/' },
  { name: '版本管理', path: '/versions' },
  { name: '设置', path: '/settings' },
]

function isActive(path: string): boolean {
  return route.path === path
}

function navigateTo(path: string) {
  router.push(path)
  showMobileMenu.value = false
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
    <!-- 顶部导航栏 -->
    <header class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
      <div class="px-6 h-16 flex items-center justify-between">
        <!-- 左侧：Logo 和导航 -->
        <div class="flex items-center space-x-8">
          <!-- Logo -->
          <h1 class="text-xl font-bold text-primary-600 dark:text-primary-400">
            MoLaunch
          </h1>
          
          <!-- 导航菜单 -->
          <nav class="hidden md:flex items-center space-x-1">
            <router-link
              v-for="item in navItems"
              :key="item.path"
              :to="item.path"
              class="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
              :class="[
                isActive(item.path)
                  ? 'bg-primary-50 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300'
                  : 'text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700'
              ]"
            >
              {{ item.name }}
            </router-link>
          </nav>
        </div>
        
        <!-- 右侧：状态和用户 -->
        <div class="flex items-center space-x-4">
          <!-- SDK 状态 -->
          <div class="hidden sm:flex items-center text-sm">
            <div
              class="w-2 h-2 rounded-full mr-2"
              :class="sdkStore.isReady ? 'bg-green-500' : 'bg-yellow-500'"
            />
            <span class="text-gray-600 dark:text-gray-400">
              SDK {{ sdkStore.isReady ? '就绪' : '加载中' }}
            </span>
          </div>
          
          <!-- 用户菜单 -->
          <div v-if="authStore.isLoggedIn" class="relative">
            <button
              @click="toggleUserMenu"
              class="flex items-center space-x-2 p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
            >
              <div class="w-8 h-8 rounded-full bg-primary-100 dark:bg-primary-900 flex items-center justify-center">
                <span class="text-primary-600 dark:text-primary-400 font-semibold text-sm">
                  {{ authStore.username.charAt(0).toUpperCase() }}
                </span>
              </div>
              <span class="hidden sm:inline text-sm text-gray-700 dark:text-gray-300">
                {{ authStore.username }}
              </span>
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
                  @click="handleLogout"
                  class="w-full text-left px-4 py-2 text-sm text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/50"
                >
                  登出
                </button>
              </div>
            </transition>
          </div>
          
          <router-link
            v-else
            to="/login"
            class="btn-primary text-sm"
          >
            登录
          </router-link>
          
          <!-- 移动端菜单按钮 -->
          <button
            @click="showMobileMenu = !showMobileMenu"
            class="md:hidden p-2 rounded-lg text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                v-if="!showMobileMenu"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M4 6h16M4 12h16M4 18h16"
              />
              <path
                v-else
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>
      </div>
      
      <!-- 移动端菜单 -->
      <transition
        enter-active-class="transition ease-out duration-200"
        enter-from-class="opacity-0 -translate-y-2"
        enter-to-class="opacity-100 translate-y-0"
        leave-active-class="transition ease-in duration-150"
        leave-from-class="opacity-100 translate-y-0"
        leave-to-class="opacity-0 -translate-y-2"
      >
        <div
          v-if="showMobileMenu"
          class="md:hidden border-t border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800"
        >
          <nav class="px-4 py-2 space-y-1">
            <button
              v-for="item in navItems"
              :key="item.path"
              @click="navigateTo(item.path)"
              class="w-full text-left px-4 py-3 rounded-lg text-sm font-medium transition-colors"
              :class="[
                isActive(item.path)
                  ? 'bg-primary-50 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300'
                  : 'text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700'
              ]"
            >
              {{ item.name }}
            </button>
          </nav>
        </div>
      </transition>
    </header>
    
    <!-- 主内容区 -->
    <main class="flex-1 overflow-y-auto p-6">
      <slot />
    </main>
  </div>
</template>
