<script setup lang="ts">
/**
 * Frp 运行日志
 *
 * 功能：
 * - 顶部：隧道 ID 下拉筛选（全部 / 指定隧道，来自 store.logFiles）+ 级别筛选 + 刷新 + 清空
 * - 中部：日志流区域（深色背景 + 垂直滚动），每行按级别着色（复用 log-display.ts）
 * - 底部：状态栏（当前隧道 + 行数 + 是否还有更多）
 * - 实时：监听 `frpc-log` Tauri event 追加日志，监听 `frp-tunnel-status` 在隧道停止时刷新历史日志
 *
 * 事件监听使用项目 composable `useTauriEvent`（自动 onUnmounted unlisten）。
 * 日志颜色复用 `logLineClass`（项目约定），禁止重复定义颜色 class。
 */
import { ref, computed, onMounted, watch } from 'vue'
import { useFrpStore } from '@/stores/frp'
import { useTauriEvent } from '@/composables/useTauriEvent'
import { parseLogLines, logLineClass, type LogLine } from '@/utils/log-display'
import Button from '@/components/common/Button.vue'
import Select from '@/components/common/Select.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import {
  ArrowPathIcon,
  TrashIcon,
  DocumentTextIcon,
} from '@heroicons/vue/24/outline'
import type { FrpcLogEvent, FrpTunnelStatusEvent } from '@/types/frp'

const store = useFrpStore()

/** 隧道筛选选项：全部 + 已有日志文件的隧道 */
const tunnelOptions = computed(() => [
  { label: '全部隧道', value: '' },
  ...store.logFiles.map(f => ({ label: f.tunnelId, value: f.tunnelId })),
])

/** 级别筛选 */
const levelOptions: { label: string; value: '' | LogLine['level'] }[] = [
  { label: '全部级别', value: '' },
  { label: 'ERROR', value: 'error' },
  { label: 'WARN', value: 'warn' },
  { label: 'INFO', value: 'info' },
  { label: 'DEBUG', value: 'debug' },
]

const selectedLevel = ref<'' | LogLine['level']>('')

/** 解析后的日志行（含级别） */
const parsedLogs = computed(() => parseLogLines(store.logs.join('\n')))

/** 按级别过滤后的日志行 */
const filteredLogs = computed(() => {
  const lv = selectedLevel.value
  return lv ? parsedLogs.value.filter(l => l.level === lv) : parsedLogs.value
})

/** 实时日志事件：按当前选中隧道过滤后追加到 store.logs */
function handleFrpcLog(e: FrpcLogEvent) {
  if (store.selectedLogTunnelId && e.tunnelId !== store.selectedLogTunnelId) return
  store.logs.push(e.line)
}

/** 隧道状态变更：停止时刷新历史日志（捕获最终退出日志） */
function handleTunnelStatus(e: FrpTunnelStatusEvent) {
  if (e.status === 'stopped') {
    void store.readLogs(store.selectedLogTunnelId)
  }
}

const { start: startLogListener } = useTauriEvent<FrpcLogEvent>('frpc-log', handleFrpcLog)
const { start: startStatusListener } = useTauriEvent<FrpTunnelStatusEvent>('frp-tunnel-status', handleTunnelStatus)

/** 切换隧道筛选时重新读取该隧道的历史日志 */
watch(() => store.selectedLogTunnelId, (id) => {
  void store.readLogs(id)
})

async function handleRefresh() {
  await Promise.all([store.loadLogFiles(), store.readLogs(store.selectedLogTunnelId)])
}

function handleClear() {
  store.clearLogs()
}

onMounted(() => {
  void store.loadLogFiles()
  void store.readLogs(store.selectedLogTunnelId)
  void startLogListener()
  void startStatusListener()
})
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- 顶部工具栏 -->
    <div class="flex items-center gap-2 flex-wrap">
      <div class="w-56">
        <Select v-model="store.selectedLogTunnelId" :options="tunnelOptions" />
      </div>
      <div class="w-36">
        <Select v-model="selectedLevel" :options="levelOptions" />
      </div>
      <Tooltip text="刷新日志">
        <Button type="ghost" size="small" :loading="store.logsLoading" @click="handleRefresh">
          <template #icon><ArrowPathIcon class="w-4 h-4" /></template>
        </Button>
      </Tooltip>
      <Tooltip text="清空当前显示">
        <Button type="ghost" size="small" @click="handleClear">
          <template #icon><TrashIcon class="w-4 h-4" /></template>
        </Button>
      </Tooltip>
    </div>

    <!-- 日志流 -->
    <div
      class="flex-1 mt-3 rounded-lg border border-gray-700 bg-gray-900 overflow-auto"
      style="max-height: 60vh;"
    >
      <div v-if="filteredLogs.length > 0" class="p-3 font-mono text-xs space-y-0.5">
        <div
          v-for="line in filteredLogs"
          :key="line.no"
          :class="logLineClass(line.level)"
          class="whitespace-pre-wrap break-all"
        >
          {{ line.text }}
        </div>
      </div>
      <div
        v-else-if="!store.logsLoading"
        class="flex flex-col items-center justify-center h-full py-16"
      >
        <DocumentTextIcon class="w-12 h-12 text-gray-600 mb-3" />
        <p class="text-sm font-medium text-gray-400">暂无日志</p>
        <p class="text-xs text-gray-500 mt-1">启动隧道后将显示 frpc 实时输出</p>
      </div>
      <div v-else class="flex items-center justify-center h-full py-16">
        <ArrowPathIcon class="w-6 h-6 text-gray-500 animate-spin" />
      </div>
    </div>

    <!-- 状态栏 -->
    <div class="mt-2 flex items-center justify-between text-xs text-gray-500">
      <span>
        {{ store.selectedLogTunnelId ? '隧道：' + store.selectedLogTunnelId : '全部隧道' }}
      </span>
      <span>
        共 {{ filteredLogs.length }} 行{{ store.logsHasMore ? '（仍有更早历史日志）' : '' }}
      </span>
    </div>
  </div>
</template>
