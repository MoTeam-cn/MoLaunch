<script setup lang="ts">
/**
 * 外观设置区：主题色 + 启动器语言
 */
import { useSettingsStore } from '@/stores/settings'
import Select from '@/components/common/Select.vue'
import ColorPicker from '@/components/common/ColorPicker.vue'

const settingsStore = useSettingsStore()
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">外观</h3>
    <div class="divide-y divide-gray-200">
      <!-- 主题色（Arco Design 风格颜色选择器，控制所有 primary 蓝色区域） -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">主题色</p>
            <p class="text-xs text-gray-500 mt-0.5">控制菜单栏、按钮、选中态等所有主色区域</p>
          </div>
          <div class="flex-none w-40">
            <ColorPicker
              :model-value="settingsStore.primaryColor"
              @update:model-value="settingsStore.setPrimaryColor($event)"
            />
          </div>
        </div>
      </div>
      <!-- 启动器语言 -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">启动器语言</p>
            <p class="text-xs text-gray-500 mt-0.5">启动器界面语言，暂时仅支持简体中文</p>
          </div>
          <div class="flex-none w-40">
            <Select
              :model-value="settingsStore.language"
              :options="[
                { label: '简体中文', value: 'zh-CN' },
              ]"
              @update:model-value="settingsStore.setLanguage(String($event))"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
