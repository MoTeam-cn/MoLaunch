<script setup lang="ts">
/**
 * 版本选择入口
 *
 * 显示当前选中的版本（方块图标 + 版本名 + 类型），点击跳转到版本选择页。
 * 图标优先使用版本设置中自定义的 logo，fallback 到根据 ID 推断的类型图标。
 */

import { computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useVersionStore } from '@/stores/version'
import { useVersionSettings } from '@/composables/useVersionSettings'

const router = useRouter()
const versionStore = useVersionStore()
const { currentLogoIcon, currentMeta, loadPersonalization } = useVersionSettings()

const selectedId = computed(() => versionStore.selectedVersion)

// 选中版本变化时加载个性化设置（用于显示自定义 logo）
watch(selectedId, async (id) => {
  if (id) await loadPersonalization()
}, { immediate: true })

function goToSelect() {
  router.push('/apps/versions/select')
}
</script>

<template>
<!-- 保留原生 button：版本选择器（h-[35px] 自定义尺寸 + justify-between 布局），
     Button.vue 的 scoped size 类固定 height 无法被工具类覆盖 -->
  <button
    class="flex h-[35px] min-w-0 flex-1 items-center justify-between overflow-hidden rounded-[3px] border border-gray-300 bg-white/80 px-3 text-[13px] text-gray-600 transition-colors hover:border-primary-500 hover:text-primary-600 hover:bg-primary-50"
    @click="goToSelect"
  >
    <div class="flex min-w-0 flex-1 items-center gap-2 overflow-hidden">
      <img
        :src="currentLogoIcon || currentMeta.icon"
        class="h-4 w-4 flex-none rounded-sm"
        alt=""
      >
      <span v-if="selectedId" class="min-w-0 flex-1 truncate">{{ selectedId }}</span>
      <span v-else class="text-gray-400">无可用版本</span>
    </div>
    <div class="flex flex-none items-center gap-1.5">
      <span v-if="selectedId" class="text-xs text-gray-400">{{ currentMeta.label }}</span>
      <svg class="h-3.5 w-3.5 text-gray-400" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M7.3 14.7a1 1 0 010-1.4L11.6 9 7.3 4.7a1 1 0 011.4-1.4l5 5a1 1 0 010 1.4l-5 5a1 1 0 01-1.4 0z" clip-rule="evenodd" />
      </svg>
    </div>
  </button>
</template>
