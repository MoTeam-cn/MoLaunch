<script setup lang="ts">
/**
 * HTTP 请求日志查看器（开发者模式）
 *
 * 展示联机 API 调用日志（`.Molaunch/logs/http_YYYY-MM-DD.log`），
 * 供开发者通过 `req_id` 追踪请求链路。
 *
 * 设计要点：
 * - 使用 CollapsibleCard 实现带动画的展开/收起
 * - 默认收起，首次展开时才加载日志（懒加载，避免页面卡顿）
 * - 日期选择使用项目 Select 组件，刷新使用项目 Button 组件
 * - 工具栏靠右对齐
 * - 表格展示：时间 / 方法 / req_id / 路径 / 状态码（倒序，最新在第一行）
 */
import { ref, computed } from 'vue'
import { ArrowPathIcon, DocumentTextIcon } from '@heroicons/vue/24/outline'
import CollapsibleCard from '@/components/common/CollapsibleCard.vue'
import Select from '@/components/common/Select.vue'
import Button from '@/components/common/Button.vue'
import { readHttpLogs, listHttpLogFiles, type HttpLogEntry } from '@/utils/api/developer'
import { toastError, toastSuccess } from '@/utils/toast'

const loading = ref(false)
const entries = ref<HttpLogEntry[]>([])
const logFiles = ref<string[]>([])
const selectedFile = ref<string>('')
/** 是否已首次加载（避免重复加载） */
const loaded = ref(false)

/** 倒序排列（最新请求在第一行，无需下滑查看） */
const reversedEntries = computed(() => [...entries.value].reverse())

/** 从文件名提取日期显示文本（`http_2026-07-29.log` → `2026-07-29`） */
const fileToDate = (f: string): string => f.replace(/^http_/, '').replace(/\.log$/, '')

/** 当前选中日期（用于 readHttpLogs 调用） */
const selectedDate = computed(() => {
  if (!selectedFile.value) return undefined
  return fileToDate(selectedFile.value)
})

/** Select 选项列表 */
const logFileOptions = computed(() =>
  logFiles.value.map(f => ({ label: fileToDate(f), value: f })),
)

/** 状态码颜色 */
function statusColor(status: number): string {
  if (status >= 200 && status < 300) return 'text-green-600'
  if (status >= 400 && status < 500) return 'text-yellow-600'
  if (status >= 500) return 'text-red-600'
  return 'text-gray-600'
}

/** 方法颜色 */
function methodColor(method: string): string {
  switch (method) {
    case 'GET': return 'text-blue-600'
    case 'POST': return 'text-green-600'
    case 'PUT': return 'text-orange-600'
    case 'DELETE': return 'text-red-600'
    default: return 'text-gray-600'
  }
}

async function loadLogFiles() {
  try {
    logFiles.value = await listHttpLogFiles()
    if (logFiles.value.length > 0 && !selectedFile.value) {
      selectedFile.value = logFiles.value[0]
    }
  } catch (e) {
    toastError('获取 HTTP 日志文件列表失败：' + e)
  }
}

async function loadEntries() {
  loading.value = true
  try {
    entries.value = await readHttpLogs(selectedDate.value, 200)
  } catch (e) {
    toastError('读取 HTTP 日志失败：' + e)
    entries.value = []
  } finally {
    loading.value = false
  }
}

/** CollapsibleCard 展开事件：首次展开时懒加载 */
async function onExpand() {
  if (loaded.value) return
  loaded.value = true
  await loadLogFiles()
  await loadEntries()
}

async function onRefresh() {
  if (!loaded.value) {
    loaded.value = true
    await loadLogFiles()
  }
  await loadEntries()
}

async function onFileChange(value: string) {
  selectedFile.value = value
  await loadEntries()
}

/** 点击 req_id 复制到剪贴板 */
async function copyReqId(reqId: string) {
  if (!reqId) return
  try {
    await navigator.clipboard.writeText(reqId)
    toastSuccess(`已复制：${reqId}`)
  } catch {
    toastError('复制失败')
  }
}
</script>

<template>
  <CollapsibleCard title="HTTP 请求日志" :default-open="false" @expand="onExpand">
    <!-- 工具栏：靠右对齐 -->
    <div class="flex items-center justify-end gap-3 px-5 py-3 bg-gray-50 border-b border-gray-200">
      <Select
        :model-value="selectedFile"
        :options="logFileOptions"
        style="min-width: 160px"
        @update:model-value="onFileChange(String($event))"
      />
      <Button
        type="outline"
        size="small"
        :disabled="loading"
        @click="onRefresh"
      >
        <template #icon>
          <ArrowPathIcon class="w-3.5 h-3.5" :class="loading ? 'animate-spin' : ''" />
        </template>
        刷新
      </Button>
    </div>

    <!-- 表格 -->
    <div class="overflow-x-auto max-h-96 overflow-y-auto">
      <table v-if="entries.length > 0" class="w-full text-xs">
        <thead class="sticky top-0 bg-gray-100 text-gray-600">
          <tr>
            <th class="text-left font-medium px-3 py-2 whitespace-nowrap">时间</th>
            <th class="text-left font-medium px-3 py-2 whitespace-nowrap">方法</th>
            <th class="text-left font-medium px-3 py-2 whitespace-nowrap">req_id</th>
            <th class="text-left font-medium px-3 py-2 whitespace-nowrap">路径</th>
            <th class="text-left font-medium px-3 py-2 whitespace-nowrap">状态</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100">
          <tr v-for="(entry, i) in reversedEntries" :key="i" class="hover:bg-gray-50">
            <td class="px-3 py-1.5 text-gray-500 whitespace-nowrap font-mono">
              {{ entry.timestamp }}
            </td>
            <td class="px-3 py-1.5 whitespace-nowrap font-mono font-medium" :class="methodColor(entry.method)">
              {{ entry.method }}
            </td>
            <td
              class="px-3 py-1.5 text-gray-400 font-mono whitespace-nowrap cursor-pointer hover:text-primary-600 transition-colors"
              :title="entry.reqId ? '点击复制' : ''"
              @click="copyReqId(entry.reqId)"
            >
              {{ entry.reqId || '-' }}
            </td>
            <td class="px-3 py-1.5 text-gray-700 font-mono whitespace-nowrap">
              {{ entry.path }}
            </td>
            <td class="px-3 py-1.5 whitespace-nowrap font-mono" :class="statusColor(entry.status)">
              {{ entry.status }}
            </td>
          </tr>
        </tbody>
      </table>

      <!-- 空状态 -->
      <div v-else-if="!loading" class="flex flex-col items-center justify-center py-12">
        <DocumentTextIcon class="w-10 h-10 text-gray-300 mb-3" />
        <p class="text-sm text-gray-500">暂无 HTTP 日志</p>
        <p class="text-xs text-gray-400 mt-1">联机 API 请求记录将在此显示</p>
      </div>

      <!-- 加载中 -->
      <div v-else class="flex items-center justify-center py-12">
        <ArrowPathIcon class="w-5 h-5 text-gray-400 animate-spin" />
      </div>
    </div>
  </CollapsibleCard>
</template>
