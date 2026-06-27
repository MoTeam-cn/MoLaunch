<script setup lang="ts">
import { useSettingsStore } from '@/stores/settings'

const settingsStore = useSettingsStore()
</script>

<template>
  <div class="space-y-6">
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">外观</h3>
      <div class="divide-y divide-gray-200">
        <!-- 主题 -->
        <div class="px-5 py-4 flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-900">主题</p>
            <p class="text-xs text-gray-500 mt-0.5">选择启动器的颜色方案</p>
          </div>
          <div class="flex gap-2">
            <button
              v-for="opt in [
                { value: 'light' as const, label: '浅色' },
                { value: 'system' as const, label: '跟随系统' },
              ]"
              :key="opt.value"
              class="px-4 py-1.5 text-sm font-medium rounded-lg border transition-colors"
              :class="settingsStore.theme === opt.value
                ? 'border-primary-500 bg-primary-50 text-primary-700'
                : 'border-gray-300 text-gray-600 hover:border-gray-400'"
              @click="settingsStore.setTheme(opt.value)"
            >
              {{ opt.label }}
            </button>
          </div>
        </div>
        <!-- 语言 -->
        <div class="px-5 py-4 flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-900">语言</p>
            <p class="text-xs text-gray-500 mt-0.5">启动器界面语言</p>
          </div>
          <div class="flex gap-2">
            <button
              v-for="opt in [
                { value: 'zh-CN' as const, label: '简体中文' },
                { value: 'en-US' as const, label: 'English' },
              ]"
              :key="opt.value"
              class="px-4 py-1.5 text-sm font-medium rounded-lg border transition-colors"
              :class="settingsStore.language === opt.value
                ? 'border-primary-500 bg-primary-50 text-primary-700'
                : 'border-gray-300 text-gray-600 hover:border-gray-400'"
              @click="settingsStore.language = opt.value"
            >
              {{ opt.label }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
