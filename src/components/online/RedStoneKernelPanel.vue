<script setup lang="ts">
/**
 * 红石联机 - 内核状态
 *
 * 上半：内核运行状态卡（进程存活 + tunnel.ini 解析结果 + 释放目录说明）
 * 下半：内核日志阅读（logs/ 目录按日期文件，深色终端风格，复用 log-display 配色与解析）
 * 内核按需释放到系统临时目录 `<temp>/MoLaunch/hongshi/`，日志位于其下 logs/。
 */
import { ref, computed, watch, onMounted, onActivated, defineAsyncComponent } from 'vue'
import {
  ArrowPathIcon,
  DocumentTextIcon,
  ShieldCheckIcon,
  FolderOpenIcon,
  CheckCircleIcon,
} from '@heroicons/vue/24/outline'
import { redstoneLogFiles, redstoneReadLog, redstoneStatus } from '@/utils/api/redstone'
import { parseLogLines, logLineClass } from '@/utils/log-display'
import { toastError } from '@/utils/toast'
import { formatBytes } from '@/utils/format'
import type { RedStoneLogFileInfo, RedStoneStatusResult } from '@/types/redstone'
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const Alert = defineAsyncComponent(() => import('@/components/common/Alert.vue'))

/** 内核状态（redstone_status 返回） */
const status = ref<RedStoneStatusResult | null>(null)
/** 日志文件列表（按修改时间倒序，最新在前） */
const logFiles = ref<RedStoneLogFileInfo[]>([])
/** 当前选中的日志文件名 */
const selectedFile = ref('')
/** 当前日志全部行（原始文本） */
const rawLogs = ref<string[]>([])
/** 日志是否还有更早的历史行未读取（当前仅读尾部 maxLines） */
const logsHasMore = ref(false)
const statusLoading = ref(false)
const logsLoading = ref(false)

/** 解析后的日志行（含级别着色） */
const parsedLogs = computed(() => parseLogLines(rawLogs.value.join('\n')))

const fileOptions = computed(() =>
  logFiles.value.map((f) => ({
    label: `${f.fileName}（${formatBytes(f.sizeBytes, 1)}）`,
    value: f.fileName,
  })),
)

/** 加载内核状态（进程存活 + tunnel.ini 解析） */
async function loadStatus() {
  statusLoading.value = true
  try {
    status.value = await redstoneStatus()
  } catch (e) {
    toastError('查询内核状态失败：' + (e instanceof Error ? e.message : String(e)))
  } finally {
    statusLoading.value = false
  }
}

/** 加载日志文件列表，选中最新的日志文件（没有选中项时） */
async function loadLogFiles() {
  try {
    logFiles.value = (await redstoneLogFiles()).files
    if (!logFiles.value.some((f) => f.fileName === selectedFile.value)) {
      selectedFile.value = logFiles.value[0]?.fileName ?? ''
    }
  } catch (e) {
    toastError('获取日志列表失败：' + (e instanceof Error ? e.message : String(e)))
  }
}

/** 读取当前选中日志文件内容 */
async function loadLog() {
  if (!selectedFile.value) {
    rawLogs.value = []
    logsHasMore.value = false
    return
  }
  logsLoading.value = true
  try {
    const res = await redstoneReadLog(selectedFile.value)
    rawLogs.value = res.content.lines
    logsHasMore.value = res.content.hasMore
  } catch (e) {
    toastError('读取日志失败：' + (e instanceof Error ? e.message : String(e)))
  } finally {
    logsLoading.value = false
  }
}

/** 切换日志文件时重新读取 */
watch(selectedFile, () => {
  void loadLog()
})

/** 手动刷新：状态 + 文件列表 + 当前日志 */
async function handleRefresh() {
  await Promise.all([loadStatus(), loadLogFiles()])
  void loadLog()
}

onMounted(() => {
  void handleRefresh()
})

/** keep-alive 下从其他菜单切回时静默刷新（首次激活紧跟 onMounted，跳过） */
let activatedCount = 0
onActivated(() => {
  activatedCount += 1
  if (activatedCount > 1) void handleRefresh()
})
</script>

<template>
  <div class="space-y-4">
    <Card title="内核状态">
      <div v-if="!statusLoading && status" class="space-y-3">
        <div class="flex items-center gap-2">
          <template v-if="status.running">
            <Tag color="green" size="small">
              <template #icon><CheckCircleIcon class="w-3 h-3" /></template>
              运行中
            </Tag>
          </template>
          <template v-else>
            <Tag color="gray" size="small">
              <template #icon><ShieldCheckIcon class="w-3 h-3" /></template>
              已停止
            </Tag>
          </template>
          <Tag v-if="status.status === 'open'" color="green" size="small">隧道已建立</Tag>
          <Tag v-else-if="status.status === 'closed'" color="red" size="small">隧道已关闭</Tag>
          <Tag v-else color="gray" size="small">未创建隧道</Tag>
        </div>
        <div class="grid grid-cols-2 gap-3 text-sm">
          <div>
            <div class="text-xs text-gray-500">联机地址</div>
            <div class="font-mono mt-0.5 break-all">
              {{ status.status === 'open' ? `${status.server}:${status.port}` : '—' }}
            </div>
          </div>
          <div>
            <div class="text-xs text-gray-500">中转服务器</div>
            <div class="font-mono mt-0.5 break-all">{{ status.server ?? '—' }}</div>
          </div>
          <div>
            <div class="text-xs text-gray-500">本地端口</div>
            <div class="mt-0.5">{{ status.port ?? '—' }}</div>
          </div>
          <div>
            <div class="text-xs text-gray-500">建立时间</div>
            <div class="mt-0.5">{{ status.created ?? '—' }}</div>
          </div>
        </div>
        <Alert variant="soft" type="info" message="红石内核按需释放到系统临时目录（&lt;temp&gt;/MoLaunch/hongshi/），退出即随系统清理；日志位于内核目录 logs/ 下，由内核与控制台同步输出" />
      </div>
      <div v-else-if="!statusLoading" class="flex flex-col items-center justify-center py-10 gap-2 text-gray-400">
        <ShieldCheckIcon class="w-8 h-8" />
        <span class="text-sm">暂未启动红石内核</span>
      </div>
    </Card>

    <Card title="内核日志">
      <template #extra>
        <Button type="ghost" size="small" :loading="logsLoading || statusLoading" @click="handleRefresh">
          <template #icon><ArrowPathIcon class="w-3.5 h-3.5" /></template>
          刷新
        </Button>
      </template>

      <div class="flex items-center gap-2 mb-3">
        <div class="w-64">
          <Select v-model="selectedFile" :options="fileOptions" placeholder="选择日志文件" />
        </div>
        <span class="text-xs text-gray-500 flex items-center gap-1">
          <FolderOpenIcon class="w-3.5 h-3.5" />
          日志目录：&lt;temp&gt;/MoLaunch/hongshi/logs/
        </span>
      </div>

      <div
        data-inner-scroll
        class="rounded-lg border border-gray-700 bg-gray-900 overflow-auto"
        style="max-height: 60vh;"
      >
        <div v-if="parsedLogs.length > 0" class="p-3 font-mono text-xs space-y-0.5">
          <div
            v-for="line in parsedLogs"
            :key="line.no"
            :class="logLineClass(line.level)"
            class="whitespace-pre-wrap break-all"
          >
            {{ line.text }}
          </div>
        </div>
        <div
          v-else-if="!logsLoading"
          class="flex flex-col items-center justify-center py-16"
        >
          <DocumentTextIcon class="w-12 h-12 text-gray-600 mb-3" />
          <p class="text-sm font-medium text-gray-400">暂无日志</p>
          <p class="text-xs text-gray-500 mt-1">启动红石内核后将在该目录生成日志</p>
        </div>
        <div v-else class="flex items-center justify-center py-16">
          <ArrowPathIcon class="w-6 h-6 text-gray-500 animate-spin" />
        </div>
      </div>

      <div class="mt-2 flex items-center justify-between text-xs text-gray-500">
        <span>{{ selectedFile || '未选择日志文件' }}</span>
        <span>
          共 {{ parsedLogs.length }} 行{{ logsHasMore ? '（仍在读取末尾，更早日志未显示）' : '' }}
        </span>
      </div>
    </Card>
  </div>
</template>