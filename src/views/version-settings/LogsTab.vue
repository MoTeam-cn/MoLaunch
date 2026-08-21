<script setup lang="ts">
/**
 * 版本设置 - 实例日志子页签
 *
 * 顶栏：日志文件下拉框（选择实例 logs/ 目录下文件）+ 刷新 + 分享（浮层二选一）
 * 内容：日志全文展示（暗色等宽，可滚动），读取前已由后端脱敏
 */
import { ref, computed, onMounted, watch, defineAsyncComponent } from 'vue'
import { useVersionSettings } from '@/composables/useVersionSettings'
import { listInstanceLogs, readInstanceLog } from '@/utils/api/version'
import type { InstanceLogFile } from '@/utils/api/version'
import { sanitizeShareLog, uploadLogShare, LOG_SHARE_PROVIDERS } from '@/utils/logShare'
import type { LogShareProvider } from '@/utils/logShare'
import { open } from '@tauri-apps/plugin-shell'
import { toastError, toastSuccess } from '@/utils/toast'
import { ArrowPathIcon, DocumentTextIcon, PaperAirplaneIcon } from '@heroicons/vue/24/outline'
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))

const { effectiveDir } = useVersionSettings()

const logFiles = ref<InstanceLogFile[]>([])
const selectedName = ref('')
const content = ref('')
const loadingList = ref(false)
const loadingContent = ref(false)
const shareMenuOpen = ref(false)
const sharing = ref<LogShareProvider | null>(null)

const options = computed(() =>
  logFiles.value.map((f) => ({ label: f.name, value: f.name })),
)

const selectedFile = computed(() => logFiles.value.find((f) => f.name === selectedName.value))

const lineCount = computed(() => (content.value ? content.value.split('\n').length : 0))

/** 加载日志文件列表，默认选中 latest.log */
async function loadList() {
  if (!effectiveDir.value) return
  loadingList.value = true
  try {
    logFiles.value = await listInstanceLogs(effectiveDir.value)
    const keep = selectedName.value && logFiles.value.some((f) => f.name === selectedName.value)
    if (!keep) {
      selectedName.value =
        logFiles.value.find((f) => f.name === 'latest.log')?.name ?? logFiles.value[0]?.name ?? ''
      if (selectedName.value) await loadContent()
    }
  } catch (e) {
    toastError('加载日志列表失败：' + String(e))
  } finally {
    loadingList.value = false
  }
}

/** 加载选中的日志文件内容 */
async function loadContent() {
  if (!effectiveDir.value || !selectedName.value) {
    content.value = ''
    return
  }
  loadingContent.value = true
  try {
    content.value = await readInstanceLog(effectiveDir.value, selectedName.value)
  } catch (e) {
    content.value = ''
    toastError('读取日志失败：' + String(e))
  } finally {
    loadingContent.value = false
  }
}

watch(selectedName, (v) => {
  if (v) loadContent()
})

onMounted(loadList)

/** 分享当前日志到云端服务：脱敏 → 上传 → 打开分享页 */
async function shareTo(provider: LogShareProvider) {
  if (!content.value) {
    toastError('当前没有可分享的日志内容')
    return
  }
  shareMenuOpen.value = false
  sharing.value = provider
  try {
    const url = await uploadLogShare(sanitizeShareLog(content.value), provider)
    await open(url)
    toastSuccess('日志已分享，已打开分享页面')
  } catch (e) {
    toastError('分享失败：' + String(e))
  } finally {
    sharing.value = null
  }
}

function formatSize(size: number): string {
  if (size < 1024) return `${size} B`
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
  return `${(size / 1024 / 1024).toFixed(2)} MB`
}

function formatTime(ts: number): string {
  if (!ts) return ''
  const d = new Date(ts * 1000)
  const p = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`
}
</script>

<template>
  <div class="flex h-full flex-col overflow-hidden">
    <!-- 顶栏：文件选择 + 信息 + 刷新 + 分享 -->
    <div class="flex flex-none items-center gap-3 border-b border-gray-200 bg-white px-4 py-3">
      <div class="w-64">
        <Select
          v-model="selectedName"
          :options="options"
          :disabled="loadingList"
          placeholder="选择日志文件"
        />
      </div>
      <div v-if="selectedFile" class="min-w-0 flex-1 truncate text-xs text-gray-400">
        <span class="mr-3">{{ formatSize(selectedFile.size) }}</span>
        <span>{{ formatTime(selectedFile.modified) }}</span>
        <span v-if="lineCount > 0" class="ml-3">{{ lineCount }} 行</span>
      </div>
      <div class="flex flex-none items-center gap-2">
        <Tooltip :text="loadingList ? '加载中...' : '刷新列表'" position="top">
          <Button type="ghost" size="small" :loading="loadingList" @click="loadList">
            <template #icon>
              <ArrowPathIcon class="h-4 w-4" />
            </template>
          </Button>
        </Tooltip>
        <div class="relative">
          <Button
            type="ghost"
            size="small"
            :loading="sharing !== null"
            :disabled="!content"
            @click="shareMenuOpen = !shareMenuOpen"
          >
            <template #icon>
              <PaperAirplaneIcon class="h-4 w-4" />
            </template>
            分享日志
          </Button>
          <div
            v-if="shareMenuOpen"
            class="absolute bottom-full right-0 z-20 mb-1 w-60 rounded-md border border-gray-200 bg-white p-1 shadow-lg"
            @mouseleave="shareMenuOpen = false"
          >
            <button
              v-for="item in LOG_SHARE_PROVIDERS"
              :key="item.value"
              class="flex w-full items-start gap-2 rounded px-2 py-1.5 text-left text-xs hover:bg-gray-50"
              @click="shareTo(item.value)"
            >
              <div class="min-w-0">
                <p class="font-medium text-gray-700">{{ item.label }}</p>
                <p class="text-gray-400">{{ item.desc }}</p>
              </div>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 内容区 -->
    <div class="min-h-0 flex-1 overflow-hidden bg-gray-900">
      <div v-if="loadingContent" class="flex h-full items-center justify-center text-xs text-gray-400">
        正在读取日志...
      </div>
      <div
        v-else-if="!content"
        class="flex h-full flex-col items-center justify-center gap-3 text-gray-400"
      >
        <DocumentTextIcon class="h-12 w-12" />
        <p class="text-sm">暂无日志内容</p>
        <p class="text-xs">请先启动游戏产生日志，或从上方下拉框选择日志文件</p>
      </div>
      <pre
        v-else
        class="h-full overflow-y-auto p-4 font-mono text-xs leading-5 whitespace-pre-wrap break-all text-gray-300"
      >{{ content }}</pre>
    </div>
  </div>
</template>
