<script setup lang="ts">
/**
 * 外观 · 动态主色档位
 *
 * 就绪后提供若干主色流动档位，选中后立即作用于全局主色阶；
 * 「跟随主题色」表示回到设置中的静态主色。
 */
import { defineAsyncComponent } from 'vue'
import { useAccentDynamics } from '@/composables/useAccentDynamics'

const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const { activeId, options, selectProfile } = useAccentDynamics()
</script>

<template>
  <div class="px-5 py-4">
    <div class="flex items-center justify-between gap-4">
      <div class="min-w-0">
        <p class="text-sm font-medium text-gray-900">动态主色</p>
        <p class="text-xs text-gray-500 mt-0.5">主色阶沿色相连续流动，覆盖菜单栏、按钮与选中态</p>
      </div>
      <div class="flex-none w-40">
        <Select
          :model-value="activeId"
          :options="options"
          @update:model-value="selectProfile(String($event))"
        />
      </div>
    </div>
  </div>
</template>
