<script setup lang="ts">
/**
 * 开发者模式开关卡片
 *
 * 自包含组件：自行加载解锁状态与开关状态，开关变更时通过 window 自定义事件
 * `developer-mode-changed` 通知父级（Settings.vue）更新侧边菜单显隐。
 *
 * 解锁触发点在 CreditsTab.vue（更多 → 鸣谢 → 法律信息，连续点击版权声明中
 * 「MoTeam」字段 7 次）。
 *
 * 撤销入口在本卡片底部「撤销开发者模式」按钮，二次确认后调用后端
 * `lock_developer_mode`，同时重置 DeveloperUnlocked/DeveloperMode/IgnoreTls，
 * 并关闭已打开的 DevTools。
 *
 * 数据来源：get_config / apply_config（developerMode 字段），
 * 解锁状态通过 developerUnlocked 只读字段获取。
 */
import { ref, onMounted } from 'vue'
import { applyConfig, getConfigMap } from '@/utils/api/config'
import { lockDeveloperMode } from '@/utils/api/developer'
import { toastError, toastSuccess } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import Alert from '@/components/common/Alert.vue'
import Button from '@/components/common/Button.vue'
import Select from '@/components/common/Select.vue'
import { safeCall } from '@/utils/async'
import { ArrowUturnLeftIcon } from '@heroicons/vue/24/outline'

const devUnlocked = ref(false)
const devMode = ref(false)
const locking = ref(false)

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

/** 撤销开发者模式解锁（二次确认） */
function onLock() {
  if (locking.value) return
  showConfirm(
    '撤销开发者模式',
    '将彻底撤销开发者模式解锁，关闭 DevTools（若已打开）并重置 IgnoreTls。\n撤销后需重新在鸣谢法律信息中触发隐藏字段才能再次解锁。确定继续吗？',
    async () => {
      locking.value = true
      try {
        await lockDeveloperMode()
        // 同步本地状态
        devUnlocked.value = false
        devMode.value = false
        // 通知父组件隐藏侧边菜单「开发者」项
        window.dispatchEvent(new CustomEvent('developer-mode-changed', { detail: false }))
        toastSuccess('已撤销开发者模式')
      } catch (e) {
        toastError('撤销失败：' + e)
      } finally {
        locking.value = false
      }
    },
  )
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
  <!-- 仅在「鸣谢 → 法律信息」中触发隐藏字段解锁后显示 -->
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

      <!-- 撤销解锁 -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">撤销开发者模式</p>
            <p class="text-xs text-gray-500 mt-0.5">彻底撤销解锁，需重新触发隐藏字段才能再次解锁</p>
          </div>
          <Button
            type="outline"
            :loading="locking"
            @click="onLock"
          >
            <template #icon>
              <ArrowUturnLeftIcon class="h-4 w-4" />
            </template>
            撤销解锁
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
