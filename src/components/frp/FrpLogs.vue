<script setup lang="ts">
/**
 * Frp 运行日志
 *
 * 功能：
 * - 顶部：隧道 ID 下拉筛选（全部 / 指定隧道）+ 级别筛选 + 翻译开关 + 刷新 + 清空
 * - 诊断面板：基于当前日志分析退出原因，给出中文说明和建议操作
 * - 中部：日志流区域（深色背景 + 垂直滚动），每行按级别着色（复用 log-display.ts）
 *        开启翻译后追加中文释义（复用 frp-log-translate.ts）
 * - 底部：状态栏（当前隧道 + 行数 + 是否还有更多）
 * - 实时：监听 `frpc-log` Tauri event 追加日志，监听 `frp-tunnel-status` 在隧道停止时刷新历史日志
 *
 * 事件监听使用项目 composable `useTauriEvent`（自动 onUnmounted unlisten）。
 * 日志颜色复用 `logLineClass`（项目约定），禁止重复定义颜色 class。
 * 翻译规则和诊断规则独立在 utils/ 下，便于维护和扩展。
 */
import { ref, computed, onMounted, watch } from 'vue'
import { useFrpStore } from '@/stores/frp'
import { useTauriEvent } from '@/composables/useTauriEvent'
import { parseLogLines, logLineClass, type LogLine } from '@/utils/log-display'
import { translateLogLine } from '@/utils/frp-log-translate'
import { diagnoseLogs, diagnoseBadgeClass, type DiagnoseResult } from '@/utils/frp-log-diagnose'
import { toastInfo } from '@/utils/toast'
import Button from '@/components/common/Button.vue'
import Select from '@/components/common/Select.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import {
  ArrowPathIcon,
  TrashIcon,
  DocumentTextIcon,
  LanguageIcon,
  CheckCircleIcon,
  WifiIcon,
  LockClosedIcon,
  Cog6ToothIcon,
  ServerIcon,
  QuestionMarkCircleIcon,
  ExclamationTriangleIcon,
  ChevronDownIcon,
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
/** 是否开启中文翻译（默认关闭，避免刷屏；用户主动开启后追加在行尾） */
const translateEnabled = ref(false)
/** 是否展开诊断面板（默认展开，异常退出时自动展开） */
const diagnoseExpanded = ref(true)

/** 解析后的日志行（含级别） */
const parsedLogs = computed(() => parseLogLines(store.logs.join('\n')))

/** 按级别过滤后的日志行 */
const filteredLogs = computed(() => {
  const lv = selectedLevel.value
  return lv ? parsedLogs.value.filter(l => l.level === lv) : parsedLogs.value
})

/** 诊断结果（基于当前日志分析退出原因） */
const diagnoseResult = computed<DiagnoseResult>(() => {
  // 用原始日志行（不带翻译）做诊断，避免翻译后关键词被破坏
  return diagnoseLogs(store.logs)
})

/** 诊断面板图标组件映射 */
const diagnoseIconMap = {
  normal: CheckCircleIcon,
  network: WifiIcon,
  auth: LockClosedIcon,
  config: Cog6ToothIcon,
  server: ServerIcon,
  unknown: QuestionMarkCircleIcon,
} as const

/** 实时日志事件：按当前选中隧道过滤后追加到 store.logs */
function handleFrpcLog(e: FrpcLogEvent) {
  if (store.selectedLogTunnelId && e.tunnelId !== store.selectedLogTunnelId) return
  store.logs.push(e.line)
}

/** 隧道状态变更：停止时刷新历史日志（捕获最终退出日志）+ 自动展开诊断面板 */
function handleTunnelStatus(e: FrpTunnelStatusEvent) {
  if (e.status === 'stopped') {
    // 异常退出（带 error）时自动展开诊断面板
    if (e.error) diagnoseExpanded.value = true
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
  toastInfo('日志已刷新')
}

function handleClear() {
  store.clearLogs()
  toastInfo('日志已清空')
}

/** 单行日志显示文本：开启翻译时追加中文释义 */
function displayText(line: LogLine): string {
  return translateEnabled.value ? translateLogLine(line.text) : line.text
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
    <!-- 顶部工具栏：隧道下拉框左侧，三按钮+级别筛选tag右侧 -->
    <div class="flex items-center gap-2 flex-wrap">
      <div class="w-56">
        <Select v-model="store.selectedLogTunnelId" :options="tunnelOptions" />
      </div>
      <div class="flex items-center gap-2 ml-auto">
        <Tooltip text="中文翻译（在行尾追加释义）">
          <Button
            type="ghost"
            size="small"
            :class="translateEnabled ? '!text-primary-600 !bg-primary-50' : ''"
            @click="translateEnabled = !translateEnabled"
          >
            <template #icon><LanguageIcon class="w-4 h-4" /></template>
          </Button>
        </Tooltip>
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
        <div class="w-36">
          <Select v-model="selectedLevel" :options="levelOptions" />
        </div>
      </div>
    </div>

    <!-- 诊断面板 -->
    <Transition
      enter-active-class="transition-all duration-300 ease-out origin-top"
      leave-active-class="transition-all duration-200 ease-in origin-top"
      enter-from-class="opacity-0 scale-y-95"
      leave-to-class="opacity-0 scale-y-95"
    >
      <div
        v-if="diagnoseExpanded && store.logs.length > 0"
        class="mt-3 rounded-lg border p-3"
        :class="diagnoseBadgeClass(diagnoseResult.category)"
      >
        <div class="flex items-start gap-3">
          <component
            :is="diagnoseIconMap[diagnoseResult.category]"
            class="w-5 h-5 shrink-0 mt-0.5"
          />
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 flex-wrap">
              <span class="text-sm font-semibold">{{ diagnoseResult.title }}</span>
              <span
                class="px-1.5 py-0.5 rounded text-xs font-medium uppercase"
                :class="diagnoseBadgeClass(diagnoseResult.category)"
              >
                {{ diagnoseResult.category }}
              </span>
            </div>
            <p class="mt-1 text-xs opacity-90">{{ diagnoseResult.detail }}</p>
            <p class="mt-1 text-xs font-medium">
              <ExclamationTriangleIcon class="w-3.5 h-3.5 inline mr-1" />
              建议：{{ diagnoseResult.suggestion }}
            </p>
            <details v-if="diagnoseResult.evidence.length > 0" class="mt-2">
              <summary class="text-xs cursor-pointer opacity-75 hover:opacity-100">
                查看关键日志（{{ diagnoseResult.evidence.length }} 行）
              </summary>
              <div class="mt-1 p-2 rounded bg-gray-900/50 font-mono text-xs space-y-0.5">
                <div
                  v-for="(line, i) in diagnoseResult.evidence"
                  :key="i"
                  class="text-gray-200 whitespace-pre-wrap break-all"
                >
                  {{ line }}
                </div>
              </div>
            </details>
          </div>
          <Tooltip text="收起诊断">
            <Button
              type="ghost"
              size="mini"
              @click="diagnoseExpanded = false"
            >
              <template #icon><ChevronDownIcon class="w-3.5 h-3.5 rotate-180" /></template>
            </Button>
          </Tooltip>
        </div>
      </div>
    </Transition>

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
          {{ displayText(line) }}
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
        <span v-if="translateEnabled" class="ml-2 text-primary-600">已开启翻译</span>
      </span>
      <span>
        共 {{ filteredLogs.length }} 行{{ store.logsHasMore ? '（仍有更早历史日志）' : '' }}
      </span>
    </div>
  </div>
</template>
