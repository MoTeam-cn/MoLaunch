<script setup lang="ts">
/**
 * 实验性功能开关卡片
 *
 * 自包含组件：加载 `experimentalEnabled` 配置，切换时通过 applyConfig 持久化，
 * 并派发 `experimental-mode-changed` 事件通知顶部导航实时显示「实验性」入口。
 *
 * 开启后：首次使用时惰性初始化 SQLite 聊天库（`.Molaunch/experimental/chat.db`）；
 * 关闭后仅隐藏入口，不删除已有数据。
 */
import { ref, onMounted, defineAsyncComponent } from 'vue'
import { applyConfig, getConfigMap } from '@/utils/api/config'
import { toastError, toastInfo } from '@/utils/toast'
import { safeCall } from '@/utils/async'
const Alert = defineAsyncComponent(() => import('@/components/common/Alert.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
import { EXPERIMENTAL_CHANGED_EVENT } from '@/composables/useExperimental'

const experimentalEnabled = ref(false)

async function toggleExperimental(v: boolean) {
  try {
    await applyConfig({ experimentalEnabled: v })
    experimentalEnabled.value = v
    window.dispatchEvent(new CustomEvent(EXPERIMENTAL_CHANGED_EVENT, { detail: v }))
    toastInfo(v ? '实验性功能已开启' : '实验性功能已关闭')
  } catch (e) {
    toastError('设置实验性功能失败：' + e)
    experimentalEnabled.value = !v
  }
}

onMounted(async () => {
  const config = await safeCall(() => getConfigMap(), 'load experimental flag')
  if (config) {
    experimentalEnabled.value = config.experimentalEnabled
  }
})
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">实验性功能</h3>

    <div class="mx-5 mb-4">
      <Alert
        type="warning"
        :truncate="false"
        message="实验性功能仍在开发中，界面与行为可能随时调整。开启后顶部导航显示「实验性」入口，并在首次使用时创建本地 SQLite 聊天数据库；关闭后仅隐藏入口，不会删除已有数据。"
      />
    </div>

    <div class="divide-y divide-gray-200">
      <div class="px-5 py-4">
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">启用实验性功能</p>
            <p class="text-xs text-gray-500 mt-0.5">包含 AI 聊天（Agent 工具）、日志分析等功能</p>
          </div>
          <div class="flex-none w-40">
            <Select
              :model-value="experimentalEnabled ? 'true' : 'false'"
              :options="[
                { label: '已启用', value: 'true' },
                { label: '未启用', value: 'false' },
              ]"
              @update:model-value="toggleExperimental($event === 'true')"
            />
          </div>
        </div>
        <p class="text-xs text-gray-400 mt-2">
          <template v-if="experimentalEnabled">已启用：导航显示「实验性」入口，聊天记录存储于本地 SQLite</template>
          <template v-else>未启用：导航不显示「实验性」入口，不创建数据库文件</template>
        </p>
      </div>
    </div>
  </div>
</template>