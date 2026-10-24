<script setup lang="ts">
/**
 * 界面排版设置区：界面密度 + 减少动效
 *
 * 密度通过 <html> 根字号缩放实现（紧凑 = 14px），减少动效同时尊重系统 prefers-reduced-motion。
 */
import { defineAsyncComponent } from 'vue'
import { useUiPrefsStore } from '@/stores/ui-prefs'
const SegmentedButtons = defineAsyncComponent(() => import('@/components/common/SegmentedButtons.vue'))
const ToggleRow = defineAsyncComponent(() => import('@/components/settings/ToggleRow.vue'))

const uiStore = useUiPrefsStore()

const densityOptions = [
  { label: '标准', value: 'normal' },
  { label: '紧凑', value: 'compact' },
]
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">界面排版</h3>
    <div class="divide-y divide-gray-200">
      <!-- 界面密度 -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">界面密度</p>
            <p class="text-xs text-gray-500 mt-0.5">紧凑模式整体缩小字号与间距，同屏显示更多内容</p>
          </div>
          <div class="flex-none">
            <SegmentedButtons
              :model-value="uiStore.density"
              :options="densityOptions"
              @update:model-value="uiStore.setDensity($event as 'normal' | 'compact')"
            />
          </div>
        </div>
      </div>
      <!-- 减少动效 -->
      <ToggleRow
        :model-value="uiStore.reduceMotion"
        label="减少动效"
        description="关闭界面过渡与动画（系统开启「减少动态效果」时自动生效）"
        :hover="false"
        @update:model-value="uiStore.setReduceMotion($event)"
      />
    </div>
  </div>
</template>
