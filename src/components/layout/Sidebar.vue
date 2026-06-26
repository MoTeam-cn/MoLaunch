<script setup lang="ts">
/**
 * 侧边栏组件
 */

import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useSdkStore } from '@/stores/sdk'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const sdkStore = useSdkStore()

const navItems = [
  {
    name: '首页',
    path: '/',
    icon: 'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6',
  },
  {
    name: '版本管理',
    path: '/versions',
    icon: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10',
  },
  {
    name: '设置',
    path: '/settings',
    icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z',
  },
]

const isActive = (path: string) => route.path === path

const userInitial = computed(() => {
  return authStore.username.charAt(0).toUpperCase()
})
</script>

<template>
  <aside class="w-64 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col">
    <!-- Logo -->
    <div class="p-6 border-b border-gray-200 dark:border-gray-700">
      <h1 class="text-2xl font-bold text-primary-600 dark:text-primary-400">
        MoLaunch
      </h1>
      <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
        现代化 Minecraft 启动器
      </p>
    </div>
    
    <!-- 导航菜单 -->
    <nav class="flex-1 p-4 space-y-1">
      <router-link
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="flex items-center px-4 py-3 rounded-lg transition-colors duration-200"
        :class="[
          isActive(item.path)
            ? 'bg-primary-50 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300'
            : 'text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700'
        ]"
      >
        <svg
          class="w-5 h-5 mr-3"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            :d="item.icon"
          />
        </svg>
        <span>{{ item.name }}</span>
      </router-link>
    </nav>
    
    <!-- SDK 状态 -->
    <div class="p-4 border-t border-gray-200 dark:border-gray-700">
      <div class="flex items-center text-sm">
        <div
          class="w-2 h-2 rounded-full mr-2"
          :class="sdkStore.isReady ? 'bg-green-500' : 'bg-yellow-500'"
        />
        <span class="text-gray-600 dark:text-gray-400">
          SDK {{ sdkStore.isReady ? '就绪' : '加载中' }}
        </span>
      </div>
      <div v-if="sdkStore.version" class="text-xs text-gray-500 dark:text-gray-500 mt-1">
        v{{ sdkStore.version }}
      </div>
    </div>
    
    <!-- 用户信息 -->
    <div class="p-4 border-t border-gray-200 dark:border-gray-700">
      <div v-if="authStore.isLoggedIn" class="flex items-center">
        <div class="w-10 h-10 rounded-full bg-primary-100 dark:bg-primary-900 flex items-center justify-center">
          <span class="text-primary-600 dark:text-primary-400 font-semibold">
            {{ userInitial }}
          </span>
        </div>
        <div class="ml-3">
          <p class="text-sm font-medium text-gray-900 dark:text-gray-100">
            {{ authStore.username }}
          </p>
          <p class="text-xs text-gray-500 dark:text-gray-400">
            离线模式
          </p>
        </div>
      </div>
      <router-link
        v-else
        to="/login"
        class="btn-primary w-full text-center"
      >
        登录
      </router-link>
    </div>
  </aside>
</template>
