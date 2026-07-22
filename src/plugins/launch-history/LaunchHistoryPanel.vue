<script setup lang="ts">
/**
 * 内置插件：启动历史
 *
 * 在主页右侧内容区显示最近启动过的版本列表，
 * 含启动时间、版本 ID、用户名、退出状态。
 *
 * 数据来源：后端 get_launch_history 命令（内存累积，重启后清空）。
 * 刷新策略：进入页面加载一次，监听 plugin:game-launch / plugin:game-exit 事件实时刷新。
 */
import { ref, onMounted, onUnmounted } from 'vue'
import { pluginSdk } from '@/plugins/sdk'
import {
  ClockIcon,
  CheckCircleIcon,
  XCircleIcon,
  ArrowPathIcon,
} from '@heroicons/vue/24/outline'

interface HistoryItem {
  version_id: string
  username: string
  launch_time: string
  pid: number
  exit_code: number | null
}

const history = ref<HistoryItem[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

async function loadHistory() {
  try {
    history.value = await pluginSdk.listLaunchHistory()
    error.value = null
  } catch (e) {
    error.value = String(e)
    pluginSdk.log('error', `[LaunchHistory] 加载失败: ${e}`)
  } finally {
    loading.value = false
  }
}

/** 格式化时间：RFC3339 → "MM-DD HH:mm" */
function formatTime(rfc3339: string): string {
  try {
    const d = new Date(rfc3339)
    if (isNaN(d.getTime())) return rfc3339
    const mm = String(d.getMonth() + 1).padStart(2, '0')
    const dd = String(d.getDate()).padStart(2, '0')
    const hh = String(d.getHours()).padStart(2, '0')
    const mi = String(d.getMinutes()).padStart(2, '0')
    return `${mm}-${dd} ${hh}:${mi}`
  } catch {
    return rfc3339
  }
}

/** 退出状态文案与样式 */
function getExitStatus(item: HistoryItem): {
  text: string
  cls: string
  iconCls: string
} {
  if (item.exit_code === null) {
    return { text: '运行中', cls: 'text-green-600', iconCls: 'text-green-500' }
  }
  if (item.exit_code === 0) {
    return { text: '正常退出', cls: 'text-gray-500', iconCls: 'text-green-500' }
  }
  return { text: `退出码 ${item.exit_code}`, cls: 'text-red-500', iconCls: 'text-red-500' }
}

function onGameLaunch() {
  // 延迟 1s 刷新，确保后端已写入历史
  setTimeout(loadHistory, 1000)
}
function onGameExit() {
  setTimeout(loadHistory, 500)
}

onMounted(() => {
  loadHistory()
  window.addEventListener('plugin:game-launch', onGameLaunch)
  window.addEventListener('plugin:game-exit', onGameExit)
})
onUnmounted(() => {
  window.removeEventListener('plugin:game-launch', onGameLaunch)
  window.removeEventListener('plugin:game-exit', onGameExit)
})
</script>

<template>
  <div class="flex h-full flex-col p-6">
    <!-- 标题栏 -->
    <div class="mb-4 flex items-center justify-between">
      <h3 class="text-base font-semibold text-gray-900">启动历史</h3>
      <button
        class="inline-flex items-center gap-1 rounded px-2 py-1 text-xs text-gray-500 hover:bg-gray-100 hover:text-gray-700"
        :disabled="loading"
        @click="loadHistory"
      >
        <ArrowPathIcon class="h-3.5 w-3.5" :class="{ 'animate-spin': loading }" />
        刷新
      </button>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="text-sm text-gray-500">加载中...</div>

    <!-- 错误 -->
    <div v-else-if="error" class="text-sm text-red-500">
      加载失败：{{ error }}
    </div>

    <!-- 空状态 -->
    <div
      v-else-if="history.length === 0"
      class="flex flex-1 flex-col items-center justify-center text-center"
    >
      <ClockIcon class="mb-3 h-10 w-10 text-gray-300" />
      <p class="text-sm text-gray-500">暂无启动历史</p>
      <p class="mt-1 text-xs text-gray-400">启动游戏后将在此显示最近记录</p>
    </div>

    <!-- 历史列表 -->
    <div v-else class="flex-1 space-y-2 overflow-y-auto pr-1">
      <div
        v-for="(item, idx) in history"
        :key="`${item.pid}-${idx}`"
        class="rounded-md border border-gray-200 px-3 py-2.5 hover:bg-gray-50"
      >
        <div class="flex items-center justify-between">
          <div class="min-w-0 flex-1">
            <p class="truncate text-sm font-medium text-gray-900">{{ item.version_id }}</p>
            <p class="mt-0.5 text-xs text-gray-500">
              {{ formatTime(item.launch_time) }} · {{ item.username }}
            </p>
          </div>
          <div class="ml-2 flex flex-none items-center gap-1 text-xs" :class="getExitStatus(item).cls">
            <CheckCircleIcon
              v-if="item.exit_code === null || item.exit_code === 0"
              class="h-3.5 w-3.5"
              :class="getExitStatus(item).iconCls"
            />
            <XCircleIcon v-else class="h-3.5 w-3.5" :class="getExitStatus(item).iconCls" />
            {{ getExitStatus(item).text }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
