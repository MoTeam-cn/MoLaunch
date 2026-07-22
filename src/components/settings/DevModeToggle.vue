<script setup lang="ts">
/**
 * 开发者模式开关卡片
 *
 * 自包含组件：自行加载解锁状态与开关状态，开关变更时通过 window 自定义事件
 * `developer-mode-changed` 通知父级（Settings.vue）更新侧边菜单显隐。
 *
 * 解锁触发点在 SettingsOther.vue（连续点击版本号 5 次）。
 *
 * 数据来源：get_config / apply_config（developerMode 字段），
 * 解锁状态通过 developerUnlocked 只读字段获取。
 */
import { ref, onMounted } from 'vue'
import { applyConfig, getConfigMap } from '@/utils/api/config'
import { toastError } from '@/utils/toast'
import Alert from '@/components/common/Alert.vue'
import Select from '@/components/common/Select.vue'
import { safeCall } from '@/utils/async'

const devUnlocked = ref(false)
const devMode = ref(false)

async function toggleDevMode(v: boolean) {
  try {
    await applyConfig({ developerMode: v })
    devMode.value = v
    // 通知 Settings.vue 父组件更新侧边菜单（dev 菜单项的显隐）
    window.dispatchEvent(new CustomEvent('developer-mode-changed', { detail: v }))
  } catch (e) {
    toastError('设置开发者模式失败：' + e)
    // 回滚 UI 状态
    devMode.value = !v
  }
}

onMounted(async () => {
  const config = await safeCall(() => getConfigMap(), 'load developer mode state')
  if (config) {
    devUnlocked.value = config.developerUnlocked
    devMode.value = config.developerMode
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
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">开启开发者模式</p>
            <p class="text-xs text-gray-500 mt-0.5">控制「开发者」菜单项的显示</p>
          </div>
          <div class="flex-none w-40">
            <Select
              :model-value="devMode ? 'true' : 'false'"
              :options="[
                { label: '已开启', value: 'true' },
                { label: '已关闭', value: 'false' },
              ]"
              @update:model-value="toggleDevMode($event === 'true')"
            />
          </div>
        </div>
        <p class="text-xs text-gray-400 mt-2">
          <template v-if="devMode">已开启：侧边菜单显示「开发者」项</template>
          <template v-else>已关闭：侧边菜单不显示「开发者」项</template>
        </p>
      </div>
    </div>
  </div>
</template>
