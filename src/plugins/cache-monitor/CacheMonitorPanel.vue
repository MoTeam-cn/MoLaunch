<script setup lang="ts">
/**
 * 内置插件：缓存监控面板
 *
 * 在主页右侧内容区展示各缓存目录占用情况：
 * - 顶部概览：总占用 + 总文件数 + 可自动清理标识
 * - 按分类分组（运行缓存 / 临时缓存 / AppData），每个子目录显示名称、文件数、占用大小、TTL
 * - 手动刷新按钮（不轮询）
 *
 * 数据来源：pluginSdk.getCacheStats()
 */
import { ref, computed, onMounted, defineAsyncComponent } from 'vue'
import { pluginSdk } from '@/plugins/sdk'
import type { CacheStatsResult, CacheStatEntry } from '@/plugins/sdk'
import {
  CircleStackIcon,
  ArrowPathIcon,
  ClockIcon,
} from '@heroicons/vue/24/outline'
import { formatBytes } from '@/utils/format'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))

const cacheStats = ref<CacheStatsResult | null>(null)
const loading = ref(true)
const refreshing = ref(false)
const error = ref<string | null>(null)

async function loadStats() {
  try {
    cacheStats.value = await pluginSdk.getCacheStats()
    error.value = null
  } catch (e) {
    error.value = String(e)
    pluginSdk.log('error', `[CacheMonitor] 加载失败: ${e}`)
  } finally {
    loading.value = false
    refreshing.value = false
  }
}

async function refresh() {
  refreshing.value = true
  await loadStats()
}

/** 所有条目扁平化 */
const allEntries = computed<CacheStatEntry[]>(() => {
  if (!cacheStats.value) return []
  return [
    ...cacheStats.value.cache,
    ...cacheStats.value.cacheTemp,
    ...cacheStats.value.cacheApp,
  ]
})

/** 总占用大小 */
const totalSize = computed(() => allEntries.value.reduce((s, e) => s + e.totalSize, 0))

/** 总文件数 */
const totalFiles = computed(() => allEntries.value.reduce((s, e) => s + e.fileCount, 0))

/** 可自动清理的占用大小（TTL 非 null 的条目） */
const autoCleanableSize = computed(() =>
  allEntries.value.filter((e) => e.ttlHours !== null).reduce((s, e) => s + e.totalSize, 0),
)

/** 按类别分组 */
const groupedCategories = computed(() => {
  if (!cacheStats.value) return []
  return [
    { label: '运行缓存', entries: cacheStats.value.cache, color: 'blue' },
    { label: '临时缓存', entries: cacheStats.value.cacheTemp, color: 'yellow' },
    { label: 'AppData', entries: cacheStats.value.cacheApp, color: 'green' },
  ].filter((g) => g.entries.length > 0)
})

onMounted(() => loadStats())
</script>

<template>
  <div class="flex h-full flex-col p-6">
    <!-- 标题栏（固定） -->
    <div class="flex flex-none items-center justify-between mb-4">
      <div class="flex items-center gap-2">
        <CircleStackIcon class="h-5 w-5 text-primary-500" />
        <h3 class="text-base font-semibold text-gray-900">缓存监控</h3>
      </div>
      <Button
        type="ghost"
        size="mini"
        :disabled="refreshing"
        @click="refresh"
      >
        <template #icon>
          <ArrowPathIcon class="h-3.5 w-3.5" :class="{ 'animate-spin': refreshing }" />
        </template>
        刷新
      </Button>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="text-sm text-gray-500">加载中...</div>

    <!-- 错误 -->
    <div v-else-if="error" class="text-sm text-red-500">
      加载失败：{{ error }}
    </div>

    <!-- 内容 -->
    <template v-else>
      <!-- 概览卡片（固定） -->
      <div class="flex-none grid grid-cols-3 gap-3 mb-4">
        <div class="rounded-md border border-gray-200 p-3">
          <p class="text-[11px] text-gray-500">总占用</p>
          <p class="mt-1 text-lg font-semibold text-gray-900">{{ formatBytes(totalSize) }}</p>
        </div>
        <div class="rounded-md border border-gray-200 p-3">
          <p class="text-[11px] text-gray-500">文件总数</p>
          <p class="mt-1 text-lg font-semibold text-gray-900">{{ totalFiles }}</p>
        </div>
        <div class="rounded-md border border-gray-200 p-3">
          <p class="text-[11px] text-gray-500">可自动清理</p>
          <p class="mt-1 text-lg font-semibold text-yellow-600">{{ formatBytes(autoCleanableSize) }}</p>
        </div>
      </div>

      <!-- 按分类分组明细（可滚动） -->
      <div class="flex-1 space-y-4 overflow-y-auto pr-1">
        <div
          v-for="group in groupedCategories"
          :key="group.label"
          class="rounded-md border border-gray-200 p-4"
        >
          <div class="mb-3 flex items-center justify-between">
            <span class="text-sm font-medium text-gray-900">{{ group.label }}</span>
            <span class="text-xs text-gray-500">
              {{ group.entries.reduce((s, e) => s + e.fileCount, 0) }} 文件 ·
              {{ formatBytes(group.entries.reduce((s, e) => s + e.totalSize, 0)) }}
            </span>
          </div>
          <div class="space-y-2">
            <div
              v-for="entry in group.entries"
              :key="entry.path"
              class="flex items-center justify-between rounded bg-gray-50 px-3 py-2"
            >
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <span class="text-xs font-medium text-gray-900">{{ entry.name }}</span>
                  <span
                    v-if="entry.ttlHours !== null"
                    class="inline-flex items-center gap-0.5 rounded bg-yellow-50 px-1 py-0.5 text-[9px] font-medium text-yellow-700"
                  >
                    <ClockIcon class="h-2.5 w-2.5" />
                    24h
                  </span>
                  <span
                    v-else
                    class="inline-flex items-center rounded bg-gray-200 px-1 py-0.5 text-[9px] font-medium text-gray-500"
                  >
                    不清理
                  </span>
                </div>
                <p class="mt-0.5 truncate text-[10px] text-gray-400">{{ entry.path }}</p>
              </div>
              <div class="flex flex-none flex-col items-end ml-3">
                <span class="text-xs font-medium text-gray-900">{{ formatBytes(entry.totalSize) }}</span>
                <span class="text-[10px] text-gray-500">{{ entry.fileCount }} 文件</span>
              </div>
            </div>
          </div>
        </div>

        <p class="pt-1 text-xs text-gray-400">点击刷新按钮更新缓存统计</p>
      </div>
    </template>
  </div>
</template>
