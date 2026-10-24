<script setup lang="ts">
/**
 * 功能隐藏设置区
 *
 * 隐藏导航栏中的页面入口：被隐藏的页面不会出现在顶部导航，直接访问路径会回到首页。
 * 首页与设置页不可隐藏（设置页是恢复入口）。
 */
import { defineAsyncComponent } from 'vue'
import { useUiPrefsStore } from '@/stores/ui-prefs'
import { HIDEABLE_PAGES } from '@/types/ui-prefs'
const Checkbox = defineAsyncComponent(() => import('@/components/common/Checkbox.vue'))

const uiStore = useUiPrefsStore()
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">功能隐藏</h3>
    <div class="divide-y divide-gray-200">
      <div v-for="page in HIDEABLE_PAGES" :key="page.path" class="px-5 py-3">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">{{ page.label }}</p>
            <p class="text-xs text-gray-500 mt-0.5">{{ page.hint }}</p>
          </div>
          <div class="flex-none">
            <Checkbox
              :model-value="uiStore.isPageHidden(page.path)"
              @update:model-value="uiStore.toggleHiddenPage(page.path)"
            >
              隐藏
            </Checkbox>
          </div>
        </div>
      </div>
      <div class="px-5 py-3">
        <p class="text-xs text-gray-500">
          首页与设置页不可隐藏；隐藏只影响导航入口，不会删除任何功能。
        </p>
      </div>
    </div>
  </div>
</template>
