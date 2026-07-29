<script setup lang="ts">
/**
 * 设置 - 开发者页面
 *
 * 展示日志查看、缓存目录路径、存储信息、系统信息等开发者专属内容。
 * 缓存统计已迁移到独立的「缓存管理」页面（SettingsCache.vue，普通用户可见）。
 */
import { ref, computed, onMounted } from 'vue'
import * as tauri from '@/utils/tauri'
import { applyConfig, getConfigMap } from '@/utils/api/config'
import { toastError } from '@/utils/toast'
import { formatBytes } from '@/utils/format'
import { osDisplay, archDisplay } from '@/utils/system-display'
import LogViewer from '@/components/settings/LogViewer.vue'
import HttpLogViewer from '@/components/settings/HttpLogViewer.vue'
import Button from '@/components/common/Button.vue'
import Select from '@/components/common/Select.vue'
import { safeCall } from '@/utils/async'
import {
  FolderOpenIcon,
  DocumentTextIcon,
} from '@heroicons/vue/24/outline'

// ==================== 实验性功能 ====================
const modrinthCdnRawEnabled = ref(false)

async function toggleModrinthCdnRaw(v: boolean) {
  try {
    await applyConfig({ modrinthCdnRawEnabled: v })
    modrinthCdnRawEnabled.value = v
  } catch (e) {
    toastError('设置 Modrinth CDN 直连失败：' + e)
    modrinthCdnRawEnabled.value = !v
  }
}

// ==================== 存储目录 ====================
const storageDirs = ref<tauri.StorageDirs | null>(null)

async function loadStorageDirs() {
  try {
    storageDirs.value = await tauri.getStorageDirs()
  } catch (e) {
    console.error('Failed to load storage dirs:', e)
    toastError('获取存储目录失败：' + e)
  }
}

async function openDir(path: string) {
  try {
    await tauri.openPath(path)
  } catch (e) {
    toastError('打开目录失败：' + e)
  }
}

// ==================== 系统信息 ====================
const systemInfo = ref<tauri.SystemInfo | null>(null)

async function loadSystemInfo() {
  try {
    systemInfo.value = await tauri.getSystemInfo()
  } catch (e) {
    console.error('Failed to load system info:', e)
    toastError('获取系统信息失败：' + e)
  }
}

/** 缓存卡片条目（运行路径缓存 / 临时目录 / 系统临时缓存 / AppData 缓存） */
const cacheEntries = computed<{ label: string; path: string }[]>(() => {
  if (!storageDirs.value) return []
  return [
    { label: '运行路径缓存', path: storageDirs.value.cache },
    { label: '运行路径临时', path: storageDirs.value.temp },
    { label: '系统临时缓存', path: storageDirs.value.cacheTemp },
    { label: 'AppData 缓存', path: storageDirs.value.cacheApp },
  ]
})

/** 存储信息卡片条目（数据根目录 / 配置文件 / 日志目录） */
const storageEntries = computed<{ label: string; path: string; locate?: boolean }[]>(() => {
  if (!storageDirs.value) return []
  return [
    { label: '数据根目录', path: storageDirs.value.base },
    { label: '配置文件', path: storageDirs.value.config, locate: true },
    { label: '日志目录', path: storageDirs.value.logs },
  ]
})

/** 系统信息卡片条目（key 用于 v-for 稳定 key） */
const systemEntries = computed<{ key: string; label: string; value: string }[]>(() => {
  if (!systemInfo.value) return []
  const s = systemInfo.value
  return [
    { key: 'appVersion', label: '应用版本', value: 'v' + s.appVersion },
    { key: 'os', label: '操作系统', value: osDisplay(s.os) },
    { key: 'arch', label: '架构', value: archDisplay(s.arch) },
    { key: 'bit', label: '位数', value: s.is64bit ? '64 位' : '32 位' },
    { key: 'total', label: '总内存', value: formatBytes(s.totalMemory) },
    { key: 'used', label: '已用内存', value: formatBytes(s.usedMemory) },
    { key: 'avail', label: '可用内存', value: formatBytes(s.availableMemory) },
    { key: 'usage', label: '内存使用率', value: s.memoryUsagePercent.toFixed(1) + '%' },
  ]
})

onMounted(async () => {
  await Promise.all([
    loadStorageDirs(),
    loadSystemInfo(),
    safeCall(async () => {
      const config = await getConfigMap()
      modrinthCdnRawEnabled.value = config.modrinthCdnRawEnabled
    }, 'load developer config'),
  ])
})
</script>

<template>
  <div class="space-y-6">
    <!-- 实验性功能 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">实验性功能</h3>
      <div class="divide-y divide-gray-200">
        <!-- Modrinth CDN 直连开关 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">Modrinth CDN 直连</p>
              <p class="text-xs text-gray-500 mt-0.5">
                将 cdn.modrinth.com 替换为 cdn-raw.modrinth.com（绕过中国大陆 cdn-alt 跳转）
              </p>
            </div>
            <div class="flex-none w-40">
              <Select
                :model-value="modrinthCdnRawEnabled ? 'true' : 'false'"
                :options="[
                  { label: '已开启', value: 'true' },
                  { label: '已关闭', value: 'false' },
                ]"
                @update:model-value="toggleModrinthCdnRaw($event === 'true')"
              />
            </div>
          </div>
          <p class="text-xs text-gray-400 mt-2">
            <template v-if="modrinthCdnRawEnabled">已开启：Modrinth 下载走 cdn-raw 直连</template>
            <template v-else>已关闭：Modrinth 下载走官方 CDN（可能跳转 cdn-alt）</template>
          </p>
        </div>
      </div>
    </div>

    <!-- HTTP 请求日志（联机 API 调用追踪，默认收起，展开才加载） -->
    <HttpLogViewer />

    <!-- 日志查看（自包含子组件，传入日志目录用于「打开目录」按钮） -->
    <LogViewer :logs-dir="storageDirs?.logs" />

    <!-- 缓存目录（仅展示父目录路径，便于整体定位；详细统计见「缓存管理」页） -->
    <div v-if="storageDirs" class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">缓存目录</h3>
      <div class="divide-y divide-gray-200">
        <div
          v-for="entry in cacheEntries"
          :key="entry.label"
          class="px-5 py-3 flex items-center justify-between"
        >
          <div>
            <p class="text-sm text-gray-500">{{ entry.label }}</p>
            <p class="text-xs text-gray-900 font-mono mt-1 break-all">{{ entry.path }}</p>
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

    <!-- 存储信息 -->
    <div v-if="storageDirs" class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">存储信息</h3>
      <div class="divide-y divide-gray-200">
        <div
          v-for="entry in storageEntries"
          :key="entry.label"
          class="px-5 py-3 flex items-center justify-between"
        >
          <div>
            <p class="text-sm text-gray-500">{{ entry.label }}</p>
            <p class="text-xs text-gray-900 font-mono mt-1 break-all">{{ entry.path }}</p>
          </div>
          <Button
            type="outline"
            size="small"
            class="shrink-0 ml-4"
            @click="openDir(entry.path)"
          >
            <template #icon><component :is="entry.locate ? DocumentTextIcon : FolderOpenIcon" class="w-3.5 h-3.5" /></template>
            {{ entry.locate ? '定位' : '打开' }}
          </Button>
        </div>
      </div>
    </div>

    <!-- 系统信息 -->
    <div v-if="systemInfo" class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">系统信息</h3>
      <div class="divide-y divide-gray-200">
        <div
          v-for="entry in systemEntries"
          :key="entry.key"
          class="px-5 py-3 flex items-center justify-between"
        >
          <span class="text-sm text-gray-500">{{ entry.label }}</span>
          <span class="text-sm text-gray-900 font-mono">{{ entry.value }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
