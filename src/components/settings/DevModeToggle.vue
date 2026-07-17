<script setup lang="ts">
/**
 * 开发者模式开关卡片
 *
 * 自包含组件：自行加载解锁状态与开关状态，开关变更时通过 window 自定义事件
 * `developer-mode-changed` 通知父级（Settings.vue）更新侧边菜单显隐。
 *
 * 解锁触发点在 SettingsOther.vue（连续点击版本号 5 次）。
 */
import { ref, onMounted } from 'vue'
import * as tauri from '@/utils/tauri'
import { showError } from '@/utils/toast'
import Alert from '@/components/common/Alert.vue'

const devUnlocked = ref(false)
const devMode = ref(false)

async function toggleDevMode(v: boolean) {
  try {
    await tauri.setDeveloperMode(v)
    devMode.value = v
    // 通知 Settings.vue 父组件更新侧边菜单（dev 菜单项的显隐）
    window.dispatchEvent(new CustomEvent('developer-mode-changed', { detail: v }))
  } catch (e) {
    showError('设置开发者模式失败：' + e)
    // 回滚 UI 状态
    devMode.value = !v
  }
}

onMounted(async () => {
  try {
    devUnlocked.value = await tauri.isDeveloperUnlocked()
    if (devUnlocked.value) {
      devMode.value = await tauri.isDeveloperMode()
    }
  } catch (e) {
    console.error('Failed to load developer mode state:', e)
  }
})
</script>

<template>
  <!-- 仅在「其他」页连续点击版本号 5 次解锁后显示 -->
  <div v-if="devUnlocked" class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">开发者模式</h3>

    <!-- 提示框 -->
    <div class="mx-5 mb-4">
      <Alert
        type="info"
        :truncate="false"
        message="开启后侧边菜单将出现「开发者」项，可查看日志、缓存、存储信息与系统信息。"
      />
    </div>

    <div class="divide-y divide-gray-200">
      <!-- 开关 -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between mb-2">
          <div>
            <p class="text-sm font-medium text-gray-900">开启开发者模式</p>
            <p class="text-xs text-gray-500 mt-0.5">控制「开发者」菜单项的显示</p>
          </div>
        </div>
        <div class="flex gap-2">
          <button
            class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
            :class="devMode
              ? 'border-primary-500 bg-primary-50 text-primary-700'
              : 'border-gray-200 text-gray-600 hover:border-gray-300'"
            @click="toggleDevMode(true)"
          >
            已开启
          </button>
          <button
            class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
            :class="!devMode
              ? 'border-primary-500 bg-primary-50 text-primary-700'
              : 'border-gray-200 text-gray-600 hover:border-gray-300'"
            @click="toggleDevMode(false)"
          >
            已关闭
          </button>
        </div>
        <p class="text-xs text-gray-400 mt-2">
          <template v-if="devMode">已开启：侧边菜单显示「开发者」项</template>
          <template v-else>已关闭：侧边菜单不显示「开发者」项</template>
        </p>
      </div>
    </div>
  </div>
</template>
