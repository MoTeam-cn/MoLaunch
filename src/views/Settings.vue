<script setup lang="ts">
/**
 * 设置页面
 */

import { ref, onMounted } from 'vue'
import { useSdkStore } from '@/stores/sdk'
import { useSettingsStore } from '@/stores/settings'
import type { LayoutMode, Theme } from '@/stores/settings'

const sdkStore = useSdkStore()
const settingsStore = useSettingsStore()

const gameDir = ref('.minecraft')
const maxThreads = ref(8)
const minMemory = ref(512)
const maxMemory = ref(2048)
const logLevel = ref(3)

const showSaveSuccess = ref(false)

onMounted(() => {
  // TODO: 从存储加载游戏设置
})

function handleSave() {
  // TODO: 保存游戏设置
  showSaveSuccess.value = true
  setTimeout(() => {
    showSaveSuccess.value = false
  }, 3000)
}

function handleReset() {
  gameDir.value = '.minecraft'
  maxThreads.value = 8
  minMemory.value = 512
  maxMemory.value = 2048
  logLevel.value = 3
  settingsStore.setLayoutMode('sidebar')
  settingsStore.setTheme('system')
}
</script>

<template>
  <div class="max-w-2xl mx-auto">
    <!-- 标题 -->
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
        设置
      </h1>
      <p class="text-gray-600 dark:text-gray-400 mt-1">
        配置启动器参数
      </p>
    </div>

    <!-- 保存成功提示 -->
    <transition
      enter-active-class="transition ease-out duration-300"
      enter-from-class="transform opacity-0 -translate-y-2"
      enter-to-class="transform opacity-100 translate-y-0"
      leave-active-class="transition ease-in duration-200"
      leave-from-class="transform opacity-100 translate-y-0"
      leave-to-class="transform opacity-0 -translate-y-2"
    >
      <div
        v-if="showSaveSuccess"
        class="mb-4 p-4 rounded-lg bg-green-50 dark:bg-green-900/50 border border-green-200 dark:border-green-800"
      >
        <div class="flex items-center">
          <svg class="w-5 h-5 text-green-600 dark:text-green-400 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span class="text-green-800 dark:text-green-200">设置已保存</span>
        </div>
      </div>
    </transition>

    <!-- 界面设置 -->
    <div class="card mb-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
        界面设置
      </h2>
      <div class="space-y-4">
        <!-- 布局模式 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            导航布局
          </label>
          <div class="grid grid-cols-2 gap-4">
            <button
              class="relative p-4 rounded-lg border-2 transition-all"
              :class="settingsStore.layoutMode === 'sidebar'
                ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/50'
                : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'
              "
              @click="settingsStore.setLayoutMode('sidebar')"
            >
              <div class="flex items-center space-x-3">
                <div class="w-10 h-10 rounded bg-gray-200 dark:bg-gray-600 flex">
                  <div class="w-3 h-full bg-gray-400 dark:bg-gray-500 rounded-l"></div>
                  <div class="flex-1"></div>
                </div>
                <div>
                  <p class="font-medium text-gray-900 dark:text-gray-100">侧边栏</p>
                  <p class="text-xs text-gray-500 dark:text-gray-400">左侧导航</p>
                </div>
              </div>
              <div
                v-if="settingsStore.layoutMode === 'sidebar'"
                class="absolute top-2 right-2 w-5 h-5 bg-primary-500 rounded-full flex items-center justify-center"
              >
                <svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                </svg>
              </div>
            </button>
            
            <button
              class="relative p-4 rounded-lg border-2 transition-all"
              :class="settingsStore.layoutMode === 'topnav'
                ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/50'
                : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'
              "
              @click="settingsStore.setLayoutMode('topnav')"
            >
              <div class="flex items-center space-x-3">
                <div class="w-10 h-10 rounded bg-gray-200 dark:bg-gray-600 flex flex-col">
                  <div class="h-2 bg-gray-400 dark:bg-gray-500 rounded-t"></div>
                  <div class="flex-1"></div>
                </div>
                <div>
                  <p class="font-medium text-gray-900 dark:text-gray-100">顶部栏</p>
                  <p class="text-xs text-gray-500 dark:text-gray-400">顶部导航</p>
                </div>
              </div>
              <div
                v-if="settingsStore.layoutMode === 'topnav'"
                class="absolute top-2 right-2 w-5 h-5 bg-primary-500 rounded-full flex items-center justify-center"
              >
                <svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                </svg>
              </div>
            </button>
          </div>
        </div>

        <!-- 主题 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            主题
          </label>
          <div class="flex gap-4">
            <label class="flex items-center">
              <input
                type="radio"
                :checked="settingsStore.theme === 'light'"
                class="mr-2"
                @change="settingsStore.setTheme('light')"
              />
              <span class="text-gray-700 dark:text-gray-300">浅色</span>
            </label>
            <label class="flex items-center">
              <input
                type="radio"
                :checked="settingsStore.theme === 'dark'"
                class="mr-2"
                @change="settingsStore.setTheme('dark')"
              />
              <span class="text-gray-700 dark:text-gray-300">深色</span>
            </label>
            <label class="flex items-center">
              <input
                type="radio"
                :checked="settingsStore.theme === 'system'"
                class="mr-2"
                @change="settingsStore.setTheme('system')"
              />
              <span class="text-gray-700 dark:text-gray-300">跟随系统</span>
            </label>
          </div>
        </div>
      </div>
    </div>

    <!-- 游戏设置 -->
    <div class="card mb-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
        游戏设置
      </h2>
      <div class="space-y-4">
        <!-- 游戏目录 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            游戏目录
          </label>
          <input
            v-model="gameDir"
            type="text"
            class="input"
            placeholder=".minecraft"
          />
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Minecraft 游戏数据存放目录
          </p>
        </div>

        <!-- 内存分配 -->
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              最小内存 (MB)
            </label>
            <input
              v-model.number="minMemory"
              type="number"
              class="input"
              min="256"
              max="16384"
              step="256"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
              最大内存 (MB)
            </label>
            <input
              v-model.number="maxMemory"
              type="number"
              class="input"
              min="256"
              max="16384"
              step="256"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- 下载设置 -->
    <div class="card mb-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
        下载设置
      </h2>
      <div class="space-y-4">
        <!-- 下载线程数 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            下载线程数
          </label>
          <input
            v-model.number="maxThreads"
            type="range"
            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700"
            min="1"
            max="16"
            step="1"
          />
          <div class="flex justify-between text-xs text-gray-500 dark:text-gray-400 mt-1">
            <span>1</span>
            <span>{{ maxThreads }}</span>
            <span>16</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 高级设置 -->
    <div class="card mb-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
        高级设置
      </h2>
      <div class="space-y-4">
        <!-- 日志级别 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            日志级别
          </label>
          <select v-model.number="logLevel" class="input">
            <option :value="0">关闭</option>
            <option :value="1">错误</option>
            <option :value="2">警告</option>
            <option :value="3">信息</option>
            <option :value="4">调试</option>
            <option :value="5">跟踪</option>
          </select>
        </div>
      </div>
    </div>

    <!-- SDK 信息 -->
    <div class="card mb-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
        SDK 信息
      </h2>
      <div class="space-y-2 text-sm">
        <div class="flex justify-between">
          <span class="text-gray-600 dark:text-gray-400">平台</span>
          <span class="text-gray-900 dark:text-gray-100">{{ sdkStore.status?.platform || '未知' }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-gray-600 dark:text-gray-400">版本</span>
          <span class="text-gray-900 dark:text-gray-100">{{ sdkStore.version || '未加载' }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-gray-600 dark:text-gray-400">状态</span>
          <span
            :class="sdkStore.isReady ? 'text-green-600 dark:text-green-400' : 'text-yellow-600 dark:text-yellow-400'"
          >
            {{ sdkStore.isReady ? '就绪' : '加载中' }}
          </span>
        </div>
        <div class="flex justify-between">
          <span class="text-gray-600 dark:text-gray-400">设备 ID</span>
          <span class="text-gray-900 dark:text-gray-100 font-mono text-xs">
            {{ sdkStore.deviceId || '未获取' }}
          </span>
        </div>
        <div class="flex justify-between">
          <span class="text-gray-600 dark:text-gray-400">库路径</span>
          <span class="text-gray-900 dark:text-gray-100 text-xs truncate ml-2 max-w-xs">
            {{ sdkStore.status?.library_path || '未知' }}
          </span>
        </div>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="flex justify-end gap-4">
      <button class="btn-secondary" @click="handleReset">
        重置默认
      </button>
      <button class="btn-primary" @click="handleSave">
        保存设置
      </button>
    </div>
  </div>
</template>
