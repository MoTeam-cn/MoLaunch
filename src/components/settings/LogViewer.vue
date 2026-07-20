<script setup lang="ts">
/**
 * 日志查看器卡片
 *
 * 自包含组件：内部加载日志文件列表 / 读取日志内容 / 渲染虚拟滚动列表。
 * 父组件只需传入 `logsDir`（用于「打开目录」按钮），其余状态自行管理。
 */
import { ref, computed, onMounted } from 'vue'
import { RecycleScroller } from 'vue-virtual-scroller'
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'
import * as tauri from '@/utils/tauri'
import { showError, showSuccess } from '@/utils/toast'
import { parseLogLines, logLineClass, type LogLine } from '@/utils/log-display'
import Select from '@/components/common/Select.vue'
import {
  FolderOpenIcon,
  ArrowPathIcon,
} from '@heroicons/vue/24/outline'

interface Props {
  /** 日志目录路径（用于「打开目录」按钮，未传则不显示该按钮） */
  logsDir?: string
}
const props = defineProps<Props>()

const logFiles = ref<string[]>([])
const selectedLogFile = ref<string>('')
const logContent = ref<string>('')
const logLoading = ref(false)

async function loadLogFiles() {
  try {
    logFiles.value = await tauri.listLogFiles()
    // 默认选中今日日志（列表第一项，list_log_files 已按最新在前排序）
    if (logFiles.value.length > 0) {
      selectedLogFile.value = logFiles.value[0]
      await loadLogContent(logFiles.value[0])
    }
  } catch (e) {
    console.error('Failed to list log files:', e)
    showError('获取日志列表失败：' + e)
  }
}

async function loadLogContent(filename: string) {
  if (!filename) {
    logContent.value = ''
    return
  }
  logLoading.value = true
  try {
    logContent.value = await tauri.readLogFile(filename)
  } catch (e) {
    logContent.value = ''
    showError('读取日志失败：' + e)
  } finally {
    logLoading.value = false
  }
}

async function onLogSelect(filename: string) {
  selectedLogFile.value = filename
  await loadLogContent(filename)
}

async function refreshLogs() {
  await loadLogFiles()
  showSuccess('日志已刷新')
}

async function openLogsDir() {
  if (!props.logsDir) return
  try {
    await tauri.openPath(props.logsDir)
  } catch (e) {
    showError('打开目录失败：' + e)
  }
}

// Select 选项：日志文件列表
function logFileOptions() {
  return logFiles.value.map(f => ({ label: f, value: f }))
}

// 计算日志行数组（基于 logContent 响应式派生）
const logLines = computed<LogLine[]>(() => {
  if (!logContent.value) return []
  return parseLogLines(logContent.value)
})

onMounted(async () => {
  await loadLogFiles()
})
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <div class="px-5 pt-5 pb-3 flex items-center justify-between">
      <h3 class="text-sm font-semibold text-gray-900">日志</h3>
      <div class="flex items-center gap-2">
        <button
          class="inline-flex items-center gap-1 px-3 py-1.5 text-xs font-medium rounded-lg border border-gray-300 text-gray-700 hover:bg-gray-50 transition-colors"
          :disabled="logLoading"
          @click="refreshLogs"
        >
          <ArrowPathIcon class="w-3.5 h-3.5" :class="logLoading ? 'animate-spin' : ''" />
          刷新
        </button>
        <button
          v-if="logsDir"
          class="inline-flex items-center gap-1 px-3 py-1.5 text-xs font-medium rounded-lg border border-gray-300 text-gray-700 hover:bg-gray-50 transition-colors"
          @click="openLogsDir"
        >
          <FolderOpenIcon class="w-3.5 h-3.5" />
          打开目录
        </button>
      </div>
    </div>

    <div class="divide-y divide-gray-200">
      <!-- 日志文件选择 -->
      <div class="px-5 py-4">
        <p class="text-sm text-gray-500 mb-2">选择日志文件</p>
        <Select
          :model-value="selectedLogFile"
          :options="logFileOptions()"
          style="min-width: 280px"
          @update:model-value="onLogSelect(String($event))"
        />
        <p v-if="logFiles.length === 0" class="text-xs text-gray-400 mt-2">
          暂无日志文件
        </p>
      </div>

      <!-- 日志内容 -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between mb-2">
          <p class="text-sm text-gray-500">日志内容</p>
          <span v-if="logLoading" class="text-xs text-gray-400">加载中...</span>
          <span v-else-if="selectedLogFile" class="text-xs text-gray-400 font-mono">
            {{ selectedLogFile }}
          </span>
        </div>
        <div class="bg-gray-900 rounded-lg p-3 log-viewer">
          <RecycleScroller
            v-if="logLines.length > 0"
            :items="logLines"
            :item-size="20"
            key-field="no"
            v-slot="{ item }"
            style="height: 360px; overflow: auto;"
          >
            <div
              class="flex hover:bg-gray-800/50 px-1"
              style="min-height: 20px"
            >
              <span class="text-gray-600 select-none w-10 shrink-0 text-right pr-3 tabular-nums text-xs leading-5">{{ item.no }}</span>
              <span
                class="pl-3 border-l border-gray-800 whitespace-pre text-xs leading-5 font-mono"
                :class="logLineClass(item.level)"
              >{{ item.text || ' ' }}</span>
            </div>
          </RecycleScroller>
          <p v-else class="text-xs text-gray-500 font-mono">无内容</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 日志查看器：黑色背景 + 黑色滚动条 */

.log-viewer :deep(.vue-recycle-scroller) {
  background: #111827; /* gray-900 */
}

/* RecycleScroller 的 item-view 是 absolute 定位，默认 width:100% 会导致
   whitespace-pre 的长行内容溢出但容器不出现横向滚动条。
   修复：让 item-view 宽度适应内容（max-content），最小 100% */
.log-viewer :deep(.vue-recycle-scroller__item-view) {
  background: #111827;
  width: max-content !important;
  min-width: 100% !important;
}

/* 自定义滚动条：黑色背景上的深灰色滚动条（纵向 + 横向） */
.log-viewer :deep(.vue-recycle-scroller::-webkit-scrollbar) {
  width: 8px;
  height: 8px;
}

.log-viewer :deep(.vue-recycle-scroller::-webkit-scrollbar-track) {
  background: #1f2937; /* gray-800 */
}

.log-viewer :deep(.vue-recycle-scroller::-webkit-scrollbar-thumb) {
  background: #4b5563; /* gray-600 */
  border-radius: 4px;
}

.log-viewer :deep(.vue-recycle-scroller::-webkit-scrollbar-thumb:hover) {
  background: #6b7280; /* gray-500 */
}
</style>
