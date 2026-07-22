<script setup lang="ts">
/**
 * 内置插件：系统状态监控
 *
 * 在主页右侧内容区显示系统资源使用情况：
 * - 系统内存占用（进度条 + 数字）
 * - 缓存占用统计（总大小 + 文件数 + 分类明细）
 * - 当前游戏运行状态（PID、版本）
 * - SDK 初始化状态
 *
 * 数据来源：pluginSdk.getSystemMemory / getRunningGamePid / getConfig / getCacheStats。
 * 刷新策略：进入页面加载一次，之后每 3 秒轮询内存与运行状态（缓存统计不轮询，手动刷新）。
 */
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { pluginSdk } from '@/plugins/sdk'
import type { CacheStatsResult } from '@/plugins/sdk'
import {
  CpuChipIcon,
  CircleStackIcon,
  PlayCircleIcon,
  StopCircleIcon,
  ArrowPathIcon,
} from '@heroicons/vue/24/outline'
import { formatBytes } from '@/utils/format'

interface MemoryInfo {
  total: number
  used: number
  available: number
  usage_percent: number
}

const memory = ref<MemoryInfo | null>(null)
const runningPid = ref<number | null>(null)
const sdkReady = ref(false)
const cacheStats = ref<CacheStatsResult | null>(null)
const cacheLoading = ref(false)
const loading = ref(true)
const error = ref<string | null>(null)
let pollTimer: ReturnType<typeof setInterval> | null = null

async function loadAll() {
  try {
    const [mem, pid, cfg, cache] = await Promise.all([
      pluginSdk.getSystemMemory(),
      pluginSdk.getRunningGamePid(),
      pluginSdk.getConfig(),
      pluginSdk.getCacheStats(),
    ])
    memory.value = mem
    runningPid.value = pid
    // SDK 状态通过 device_id 是否存在判断（getConfig 已过滤敏感字段）
    sdkReady.value = Boolean(cfg.deviceId)
    cacheStats.value = cache
    error.value = null
  } catch (e) {
    error.value = String(e)
    pluginSdk.log('error', `[SystemMonitor] 加载失败: ${e}`)
  } finally {
    loading.value = false
  }
}

/** 手动刷新缓存统计 */
async function refreshCache() {
  cacheLoading.value = true
  try {
    cacheStats.value = await pluginSdk.getCacheStats()
  } catch (e) {
    pluginSdk.log('error', `[SystemMonitor] 缓存统计刷新失败: ${e}`)
  } finally {
    cacheLoading.value = false
  }
}



/** 内存进度条颜色（>=80% 红色，>=60% 黄色，否则绿色） */
const memoryBarColor = computed(() => {
  const pct = memory.value?.usage_percent ?? 0
  if (pct >= 80) return 'bg-red-500'
  if (pct >= 60) return 'bg-yellow-500'
  return 'bg-green-500'
})

/** 缓存总占用（所有条目大小之和） */
const cacheTotalSize = computed(() => {
  if (!cacheStats.value) return 0
  return [...cacheStats.value.cache, ...cacheStats.value.cacheTemp, ...cacheStats.value.cacheApp]
    .reduce((sum, e) => sum + e.totalSize, 0)
})

/** 缓存总文件数 */
const cacheTotalFiles = computed(() => {
  if (!cacheStats.value) return 0
  return [...cacheStats.value.cache, ...cacheStats.value.cacheTemp, ...cacheStats.value.cacheApp]
    .reduce((sum, e) => sum + e.fileCount, 0)
})

/** 缓存条目按类别分组 */
const cacheByCategory = computed(() => {
  if (!cacheStats.value) return []
  return [
    { label: '运行缓存', entries: cacheStats.value.cache },
    { label: '临时缓存', entries: cacheStats.value.cacheTemp },
    { label: 'AppData', entries: cacheStats.value.cacheApp },
  ].filter((g) => g.entries.length > 0)
})

onMounted(() => {
  loadAll()
  // 每 3 秒轮询内存与运行状态（不轮询配置和缓存，避免 IPC 重复读取）
  pollTimer = setInterval(async () => {
    try {
      memory.value = await pluginSdk.getSystemMemory()
      runningPid.value = await pluginSdk.getRunningGamePid()
    } catch (e) {
      // 轮询失败静默
    }
  }, 3000)
})

onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer)
})
</script>

<template>
  <div class="flex h-full flex-col p-6">
    <!-- 标题栏 -->
    <div class="mb-4 flex items-center justify-between">
      <h3 class="text-base font-semibold text-gray-900">系统状态</h3>
      <button
        class="inline-flex items-center gap-1 rounded px-2 py-1 text-xs text-gray-500 hover:bg-gray-100 hover:text-gray-700"
        :disabled="loading"
        @click="loadAll"
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

    <!-- 状态卡片 -->
    <div v-else class="flex-1 space-y-3 overflow-y-auto pr-1">
      <!-- 内存占用 -->
      <div class="rounded-md border border-gray-200 p-4">
        <div class="mb-2 flex items-center gap-2">
          <CircleStackIcon class="h-4 w-4 text-primary-500" />
          <span class="text-sm font-medium text-gray-900">系统内存</span>
        </div>
        <div v-if="memory" class="space-y-2">
          <!-- 进度条 -->
          <div class="h-2 w-full overflow-hidden rounded-full bg-gray-100">
            <div
              class="h-full transition-all duration-500"
              :class="memoryBarColor"
              :style="{ width: `${Math.min(100, memory.usage_percent)}%` }"
            />
          </div>
          <!-- 数字 -->
          <div class="flex items-center justify-between text-xs text-gray-500">
            <span>
              {{ formatBytes(memory.used) }} / {{ formatBytes(memory.total) }}
            </span>
            <span class="font-medium" :class="{
              'text-red-500': memory.usage_percent >= 80,
              'text-yellow-600': memory.usage_percent >= 60 && memory.usage_percent < 80,
              'text-green-600': memory.usage_percent < 60,
            }">
              {{ memory.usage_percent.toFixed(1) }}%
            </span>
          </div>
        </div>
        <p v-else class="text-xs text-gray-400">无法获取内存信息</p>
      </div>

      <!-- 缓存占用 -->
      <div class="rounded-md border border-gray-200 p-4">
        <div class="mb-2 flex items-center justify-between">
          <div class="flex items-center gap-2">
            <CircleStackIcon class="h-4 w-4 text-primary-500" />
            <span class="text-sm font-medium text-gray-900">缓存占用</span>
          </div>
          <button
            class="inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-[10px] text-gray-400 hover:bg-gray-100 hover:text-gray-600"
            :disabled="cacheLoading"
            @click="refreshCache"
          >
            <ArrowPathIcon class="h-3 w-3" :class="{ 'animate-spin': cacheLoading }" />
          </button>
        </div>
        <div v-if="cacheStats">
          <!-- 总览 -->
          <div class="mb-2 flex items-center justify-between text-xs">
            <span class="text-gray-500">{{ cacheTotalFiles }} 个文件</span>
            <span class="font-medium text-gray-900">{{ formatBytes(cacheTotalSize) }}</span>
          </div>
          <!-- 分类明细 -->
          <div class="space-y-1.5">
            <div
              v-for="group in cacheByCategory"
              :key="group.label"
              class="flex items-center justify-between text-[11px]"
            >
              <span class="text-gray-400">{{ group.label }}</span>
              <span class="text-gray-600">
                {{ group.entries.reduce((s, e) => s + e.fileCount, 0) }} 文件 ·
                {{ formatBytes(group.entries.reduce((s, e) => s + e.totalSize, 0)) }}
              </span>
            </div>
          </div>
        </div>
        <p v-else class="text-xs text-gray-400">无法获取缓存信息</p>
      </div>

      <!-- 游戏运行状态 -->
      <div class="rounded-md border border-gray-200 p-4">
        <div class="mb-2 flex items-center gap-2">
          <CpuChipIcon class="h-4 w-4 text-primary-500" />
          <span class="text-sm font-medium text-gray-900">游戏进程</span>
        </div>
        <div class="flex items-center gap-2">
          <template v-if="runningPid !== null">
            <PlayCircleIcon class="h-4 w-4 text-green-500" />
            <span class="text-sm text-green-600">运行中（PID: {{ runningPid }}）</span>
          </template>
          <template v-else>
            <StopCircleIcon class="h-4 w-4 text-gray-400" />
            <span class="text-sm text-gray-500">未运行</span>
          </template>
        </div>
      </div>

      <!-- SDK 状态 -->
      <div class="rounded-md border border-gray-200 p-4">
        <div class="mb-2 flex items-center gap-2">
          <CpuChipIcon class="h-4 w-4 text-primary-500" />
          <span class="text-sm font-medium text-gray-900">SDK 初始化</span>
        </div>
        <div class="flex items-center gap-2">
          <span
            class="inline-block h-2 w-2 rounded-full"
            :class="sdkReady ? 'bg-green-500' : 'bg-gray-300'"
          />
          <span class="text-sm" :class="sdkReady ? 'text-green-600' : 'text-gray-500'">
            {{ sdkReady ? '已就绪' : '未初始化' }}
          </span>
        </div>
      </div>

      <p class="pt-2 text-xs text-gray-400">内存与进程每 3 秒自动刷新</p>
    </div>
  </div>
</template>
