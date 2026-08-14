<script setup lang="ts">
/**
 * 设置 - 缓存管理页面
 *
 * 展示各缓存目录的统计信息（文件数、占用大小、TTL），支持打开目录。
 * 此页面为普通用户可见（不需要开发者模式），便于了解磁盘占用情况。
 *
 * 数据来源：get_cache_stats IPC 命令（后端 utils/cache_stats.rs）。
 */
import { ref, computed, onMounted, defineAsyncComponent } from 'vue'
import * as tauri from '@/utils/tauri'
import { toastError, toastSuccess } from '@/utils/toast'
import { formatBytes } from '@/utils/format'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Alert = defineAsyncComponent(() => import('@/components/common/Alert.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
import {
  FolderOpenIcon,
  ArrowPathIcon,
  CircleStackIcon,
} from '@heroicons/vue/24/outline'

const cacheStats = ref<tauri.CacheStatsResult | null>(null)
const loading = ref(false)

async function loadCacheStats() {
  loading.value = true
  try {
    cacheStats.value = await tauri.getCacheStats()
    toastSuccess('缓存统计已刷新')
  } catch (e) {
    console.error('Failed to load cache stats:', e)
    toastError('获取缓存统计失败：' + e)
  } finally {
    loading.value = false
  }
}

async function openDir(path: string) {
  try {
    await tauri.openPath(path)
  } catch (e) {
    toastError('打开目录失败：' + e)
  }
}

/** 缓存统计全部条目（合并三个类别） */
const allEntries = computed(() => {
  if (!cacheStats.value) return []
  return [
    ...cacheStats.value.cache,
    ...cacheStats.value.cacheTemp,
    ...cacheStats.value.cacheApp,
  ]
})

/** 缓存总占用 */
const totalSize = computed(() => allEntries.value.reduce((s, e) => s + e.totalSize, 0))

/** 缓存总文件数 */
const totalFiles = computed(() => allEntries.value.reduce((s, e) => s + e.fileCount, 0))

/** 可清理的缓存占用（有 TTL 的条目） */
const cleanableSize = computed(() =>
  allEntries.value.filter((e) => e.ttlHours !== null).reduce((s, e) => s + e.totalSize, 0),
)

onMounted(loadCacheStats)
</script>

<template>
  <div class="flex h-full flex-col gap-4 p-6">
    <!-- 顶部说明（固定） -->
    <Alert
      type="info"
      :truncate="false"
      message="启动器运行过程中会产生各类缓存（图片、安装包、SDK 动态库等）。标记「24h 自动清理」的目录会在文件超过 24 小时后自动删除；标记「不清理」的目录为重要资源，请勿手动删除。"
    />

    <!-- 总览卡片（固定） -->
    <div class="flex-none grid grid-cols-3 gap-4">
      <!-- 总占用 -->
      <div class="rounded-lg border border-gray-300 bg-white p-4">
        <div class="flex items-center gap-2 text-gray-500">
          <CircleStackIcon class="h-4 w-4" />
          <span class="text-xs">总占用</span>
        </div>
        <p class="mt-2 text-2xl font-semibold text-gray-900">
          {{ formatBytes(totalSize) }}
        </p>
        <p class="mt-1 text-[11px] text-gray-400">{{ totalFiles }} 个文件</p>
      </div>
      <!-- 可清理 -->
      <div class="rounded-lg border border-gray-300 bg-white p-4">
        <div class="flex items-center gap-2 text-gray-500">
          <ArrowPathIcon class="h-4 w-4" />
          <span class="text-xs">可自动清理</span>
        </div>
        <p class="mt-2 text-2xl font-semibold text-yellow-600">
          {{ formatBytes(cleanableSize) }}
        </p>
        <p class="mt-1 text-[11px] text-gray-400">24h 后自动清理</p>
      </div>
      <!-- 不可清理 -->
      <div class="rounded-lg border border-gray-300 bg-white p-4">
        <div class="flex items-center gap-2 text-gray-500">
          <CircleStackIcon class="h-4 w-4" />
          <span class="text-xs">重要资源</span>
        </div>
        <p class="mt-2 text-2xl font-semibold text-gray-700">
          {{ formatBytes(totalSize - cleanableSize) }}
        </p>
        <p class="mt-1 text-[11px] text-gray-400">SDK / Java Runtime 等</p>
      </div>
    </div>

    <!-- 详细列表（可滚动） -->
    <div class="flex-1 min-h-0 bg-white rounded-lg border border-gray-300 overflow-hidden flex flex-col">
      <div class="flex flex-none items-center justify-between px-5 pt-5 pb-3">
        <h3 class="text-sm font-semibold text-gray-900">缓存目录明细</h3>
        <Button
          type="ghost"
          size="mini"
          :disabled="loading"
          @click="loadCacheStats"
        >
          <template #icon>
            <ArrowPathIcon class="h-3.5 w-3.5" :class="{ 'animate-spin': loading }" />
          </template>
          刷新
        </Button>
      </div>

      <!-- 加载中 -->
      <div v-if="!cacheStats && loading" class="flex-1 px-5 py-12 text-center text-xs text-gray-400">
        正在统计缓存...
      </div>

      <!-- 空状态 -->
      <div
        v-else-if="allEntries.length === 0"
        class="flex flex-1 flex-col items-center justify-center px-5 py-12"
      >
        <CircleStackIcon class="mb-3 h-10 w-10 text-gray-300" />
        <p class="text-sm text-gray-500">暂无缓存数据</p>
      </div>

      <!-- 列表 -->
      <div v-else class="flex-1 overflow-y-auto divide-y divide-gray-200">
        <div
          v-for="entry in allEntries"
          :key="entry.category + '/' + entry.subDir"
          class="px-5 py-3 flex items-center justify-between"
        >
          <div class="min-w-0 flex-1">
            <div class="flex flex-wrap items-center gap-2">
              <p class="text-sm text-gray-900">{{ entry.name }}</p>
              <!-- TTL 标识 -->
              <Tag
                size="small"
                :color="entry.ttlHours ? 'gold' : 'gray'"
              >
                {{ entry.ttlHours ? `${entry.ttlHours}h 自动清理` : '不清理' }}
              </Tag>
              <Tag size="small" color="gray">{{ entry.category }}</Tag>
            </div>
            <p class="text-xs text-gray-500 mt-1">
              {{ entry.fileCount }} 个文件 · {{ formatBytes(entry.totalSize) }}
            </p>
            <p class="text-[11px] text-gray-400 font-mono mt-0.5 break-all">{{ entry.path }}</p>
          </div>
          <Button
            type="outline"
            size="small"
            class="shrink-0 ml-4"
            @click="openDir(entry.path)"
          >
            <template #icon><FolderOpenIcon class="w-3.5 h-3.5" /></template>
            打开
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
