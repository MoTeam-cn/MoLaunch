<script setup lang="ts">
import { useSettingsStore } from '@/stores/settings'
import Select from '@/components/common/Select.vue'

const settingsStore = useSettingsStore()
</script>

<template>
  <div class="space-y-6">
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">外观</h3>
      <div class="divide-y divide-gray-200">
        <!-- 主题 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">主题</p>
              <p class="text-xs text-gray-500 mt-0.5">选择启动器的颜色方案</p>
            </div>
            <div class="flex-none w-40">
              <Select
                :model-value="settingsStore.theme"
                :options="[
                  { label: '浅色', value: 'light' },
                  { label: '深色', value: 'dark' },
                  { label: '跟随系统', value: 'system' },
                ]"
                @update:model-value="settingsStore.setTheme($event as 'light' | 'dark' | 'system')"
              />
            </div>
          </div>
        </div>
        <!-- 语言 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">语言</p>
              <p class="text-xs text-gray-500 mt-0.5">启动器界面语言</p>
            </div>
            <div class="flex-none w-40">
              <Select
                v-model="settingsStore.language"
                :options="[
                  { label: '简体中文', value: 'zh-CN' },
                  { label: 'English', value: 'en-US' },
                ]"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
