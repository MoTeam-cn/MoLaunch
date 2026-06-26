<script setup lang="ts">
/**
 * 版本管理页面
 */

import { ref, computed, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useVersionStore } from '@/stores/version'
import type { DownloadProgress } from '@/stores/version'
import * as tauri from '@/utils/tauri'

// 导入 Blocks 图片
import grassIcon from '@/assets/blocks/Grass.png'
import cobblestoneIcon from '@/assets/blocks/CobbleStone.png'
import commandBlockIcon from '@/assets/blocks/CommandBlock.png'
import goldBlockIcon from '@/assets/blocks/GoldBlock.png'

const versionStore = useVersionStore()

const searchQuery = ref('')
const filterType = ref<'all' | 'release' | 'snapshot'>('all')
const loading = ref(false)
const installedVersions = ref<string[]>([])
const activeTab = ref<'available' | 'installed'>('available')

// 版本类型对应的图标
const versionTypeIcons: Record<string, string> = {
  release: grassIcon,      // 正式版 - Grass
  snapshot: commandBlockIcon, // 快照版 - CommandBlock
  old_beta: cobblestoneIcon, // 远古版 - CobbleStone
  old_alpha: cobblestoneIcon, // 远古版 - CobbleStone
}

// 特殊版本图标
const specialIcons: Record<string, string> = {
  '23w13a_or_b': goldBlockIcon, // 愚人节版
  '20w14infinite': goldBlockIcon,
  '22w13oneblockatatime': goldBlockIcon,
  '24w14potato': goldBlockIcon,
  '25w14craftmine': goldBlockIcon,
}

// 监听下载进度事件
let unlistenProgress: (() => void) | null = null
let unlistenComplete: (() => void) | null = null

onMounted(async () => {
  loading.value = true
  await Promise.all([
    versionStore.fetchVersions(),
    loadInstalledVersions(),
  ])
  loading.value = false

  // 监听下载进度
  unlistenProgress = await listen<DownloadProgress>('download-progress', (event) => {
    versionStore.updateProgress(event.payload)
  })

  // 监听下载完成
  unlistenComplete = await listen<{ version_id: string }>('download-complete', (event) => {
    installedVersions.value.push(event.payload.version_id)
    versionStore.finishDownload()
  })
})

onUnmounted(() => {
  unlistenProgress?.()
  unlistenComplete?.()
})

async function loadInstalledVersions() {
  try {
    installedVersions.value = await tauri.listInstalledVersions()
  } catch (e) {
    console.error('Failed to load installed versions:', e)
  }
}

const filteredVersions = computed(() => {
  let versions = versionStore.versions

  if (filterType.value === 'release') {
    versions = versions.filter(v => v.version_type === 'release')
  } else if (filterType.value === 'snapshot') {
    versions = versions.filter(v => v.version_type === 'snapshot')
  }

  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    versions = versions.filter(v => v.id.toLowerCase().includes(query))
  }

  return versions
})

function isInstalled(versionId: string): boolean {
  return installedVersions.value.includes(versionId)
}

function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  })
}

function getVersionIcon(versionId: string, versionType: string): string {
  // 检查是否是特殊版本（愚人节等）
  if (specialIcons[versionId]) {
    return specialIcons[versionId]
  }
  // 根据版本类型返回对应图标
  return versionTypeIcons[versionType] || grassIcon
}

function getVersionTypeBadge(versionType: string) {
  switch (versionType) {
    case 'release':
      return {
        text: '正式版',
        class: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
      }
    case 'snapshot':
      return {
        text: '快照版',
        class: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
      }
    default:
      return {
        text: '旧版本',
        class: 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200',
      }
  }
}

async function handleDownload(versionId: string) {
  versionStore.startDownload(versionId)
  try {
    await tauri.downloadVersion(versionId)
  } catch (e) {
    console.error('Failed to download version:', e)
    alert(`下载失败: ${e}`)
    versionStore.finishDownload()
  }
}
</script>

<template>
  <div class="max-w-4xl mx-auto">
    <!-- 标题 -->
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100 flex items-center">
        <img :src="grassIcon" alt="Grass" class="w-8 h-8 mr-2" />
        版本管理
      </h1>
      <p class="text-gray-600 dark:text-gray-400 mt-1">
        浏览、下载和管理 Minecraft 版本
      </p>
    </div>

    <!-- 下载进度条 -->
    <transition
      enter-active-class="transition ease-out duration-200"
      enter-from-class="opacity-0 -translate-y-2"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 -translate-y-2"
    >
      <div v-if="versionStore.downloading && versionStore.downloadProgress" class="card mb-6">
        <div class="flex items-center justify-between mb-2">
          <div>
            <span class="font-medium text-gray-900 dark:text-gray-100">
              正在下载 {{ versionStore.downloadingVersion }}
            </span>
            <span class="ml-2 text-sm text-gray-500 dark:text-gray-400">
              {{ versionStore.downloadProgress.stage }}
            </span>
          </div>
          <span class="text-sm font-medium text-primary-600 dark:text-primary-400">
            {{ versionStore.downloadProgress.percentage.toFixed(1) }}%
          </span>
        </div>
        
        <!-- 进度条 -->
        <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2.5">
          <div
            class="bg-primary-600 h-2.5 rounded-full transition-all duration-300"
            :style="{ width: `${versionStore.downloadProgress.percentage}%` }"
          ></div>
        </div>
        
        <div class="flex justify-between mt-2 text-xs text-gray-500 dark:text-gray-400">
          <span>{{ versionStore.downloadProgress.current }} / {{ versionStore.downloadProgress.total }}</span>
        </div>
      </div>
    </transition>

    <!-- 标签页 -->
    <div class="flex space-x-1 mb-6 bg-gray-100 dark:bg-gray-800 p-1 rounded-lg">
      <button
        class="flex-1 py-2 px-4 rounded-md text-sm font-medium transition-colors"
        :class="activeTab === 'available'
          ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow'
          : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100'
        "
        @click="activeTab = 'available'"
      >
        可用版本
      </button>
      <button
        class="flex-1 py-2 px-4 rounded-md text-sm font-medium transition-colors"
        :class="activeTab === 'installed'
          ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow'
          : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100'
        "
        @click="activeTab = 'installed'"
      >
        已安装 ({{ installedVersions.length }})
      </button>
    </div>

    <!-- 搜索和过滤 (仅可用版本) -->
    <div v-if="activeTab === 'available'" class="card mb-6">
      <div class="flex flex-col md:flex-row gap-4">
        <div class="flex-1">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索版本..."
            class="input"
          />
        </div>
        <div class="flex gap-2">
          <button
            class="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            :class="filterType === 'all'
              ? 'bg-primary-600 text-white'
              : 'bg-gray-200 text-gray-700 hover:bg-gray-300 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600'
            "
            @click="filterType = 'all'"
          >
            全部
          </button>
          <button
            class="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            :class="filterType === 'release'
              ? 'bg-green-600 text-white'
              : 'bg-gray-200 text-gray-700 hover:bg-gray-300 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600'
            "
            @click="filterType = 'release'"
          >
            正式版
          </button>
          <button
            class="px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            :class="filterType === 'snapshot'
              ? 'bg-yellow-600 text-white'
              : 'bg-gray-200 text-gray-700 hover:bg-gray-300 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600'
            "
            @click="filterType = 'snapshot'"
          >
            快照版
          </button>
        </div>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="card text-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600 mx-auto"></div>
      <p class="text-gray-600 dark:text-gray-400 mt-4">加载版本列表...</p>
    </div>

    <!-- 可用版本列表 -->
    <div v-else-if="activeTab === 'available'">
      <div v-if="filteredVersions.length > 0" class="space-y-2">
        <div
          v-for="version in filteredVersions"
          :key="version.id"
          class="card flex items-center justify-between hover:shadow-md transition-shadow"
        >
          <div class="flex items-center">
            <!-- 版本图标 -->
            <img
              :src="getVersionIcon(version.id, version.version_type)"
              :alt="version.id"
              class="w-10 h-10 rounded mr-3"
            />
            <div>
              <div class="flex items-center">
                <span class="font-semibold text-gray-900 dark:text-gray-100">
                  {{ version.id }}
                </span>
                <span
                  class="ml-2 text-xs px-2 py-0.5 rounded-full"
                  :class="getVersionTypeBadge(version.version_type).class"
                >
                  {{ getVersionTypeBadge(version.version_type).text }}
                </span>
                <span
                  v-if="version.id === versionStore.latestRelease"
                  class="ml-2 text-xs px-2 py-0.5 rounded-full bg-primary-100 text-primary-800 dark:bg-primary-900 dark:text-primary-200"
                >
                  最新
                </span>
                <span
                  v-if="isInstalled(version.id)"
                  class="ml-2 text-xs px-2 py-0.5 rounded-full bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200"
                >
                  已安装
                </span>
              </div>
              <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
                发布于 {{ formatDate(version.release_time) }}
              </p>
            </div>
          </div>
          <button
            class="btn-primary text-sm"
            :disabled="versionStore.downloading || isInstalled(version.id)"
            @click="handleDownload(version.id)"
          >
            <span v-if="versionStore.downloadingVersion === version.id" class="flex items-center">
              <svg class="animate-spin -ml-1 mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              下载中...
            </span>
            <span v-else-if="isInstalled(version.id)">已安装</span>
            <span v-else class="flex items-center">
              <svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
              </svg>
              下载
            </span>
          </button>
        </div>
      </div>
      <div v-else class="card text-center py-12">
        <svg class="w-16 h-16 text-gray-400 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <p class="text-gray-600 dark:text-gray-400 mt-4">
          {{ searchQuery ? '未找到匹配的版本' : '暂无版本数据' }}
        </p>
      </div>
    </div>

    <!-- 已安装版本列表 -->
    <div v-else-if="activeTab === 'installed'">
      <div v-if="installedVersions.length > 0" class="space-y-2">
        <div
          v-for="versionId in installedVersions"
          :key="versionId"
          class="card flex items-center justify-between"
        >
          <div class="flex items-center">
            <img
              :src="getVersionIcon(versionId, 'release')"
              :alt="versionId"
              class="w-10 h-10 rounded mr-3"
            />
            <div>
              <span class="font-semibold text-gray-900 dark:text-gray-100">
                {{ versionId }}
              </span>
              <span class="ml-2 text-xs px-2 py-0.5 rounded-full bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
                已安装
              </span>
            </div>
          </div>
          <button class="btn-primary text-sm">
            <svg class="w-4 h-4 mr-1 inline" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            启动
          </button>
        </div>
      </div>
      <div v-else class="card text-center py-12">
        <svg class="w-16 h-16 text-gray-400 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
        </svg>
        <p class="text-gray-600 dark:text-gray-400 mt-4">
          暂未安装任何版本
        </p>
        <button
          class="btn-primary mt-4"
          @click="activeTab = 'available'"
        >
          浏览可用版本
        </button>
      </div>
    </div>
  </div>
</template>
