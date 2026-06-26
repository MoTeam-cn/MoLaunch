<script setup lang="ts">
/**
 * 下载页面
 * 左侧侧边栏：加载类别
 * 右侧主页面：版本列表，最新在前
 */

import { ref, computed, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useVersionStore } from '@/stores/version'
import type { DownloadProgress } from '@/stores/version'
import * as tauri from '@/utils/tauri'
import {
  CubeIcon,
  WrenchIcon,
  ArchiveBoxIcon,
  CheckCircleIcon,
  ArrowDownTrayIcon,
  FolderOpenIcon,
  StarIcon,
  BeakerIcon,
  ClockIcon,
  PlayIcon,
} from '@heroicons/vue/24/outline'

// 导入 Blocks 图片
import grassIcon from '@/assets/blocks/Grass.png'
import cobblestoneIcon from '@/assets/blocks/CobbleStone.png'
import commandBlockIcon from '@/assets/blocks/CommandBlock.png'
import goldBlockIcon from '@/assets/blocks/GoldBlock.png'

const versionStore = useVersionStore()

const loading = ref(false)
const installedVersions = ref<string[]>([])
const activeCategory = ref('vanilla')
const activeSubCategory = ref('release')

// 版本类型图标
const typeIcons: Record<string, string> = {
  release: grassIcon,
  snapshot: commandBlockIcon,
  old_beta: cobblestoneIcon,
  old_alpha: cobblestoneIcon,
}

const specialIcons: Record<string, string> = {
  '23w13a_or_b': goldBlockIcon,
  '20w14infinite': goldBlockIcon,
  '22w13oneblockatatime': goldBlockIcon,
  '24w14potato': goldBlockIcon,
  '25w14craftmine': goldBlockIcon,
}

function getVersionIcon(versionId: string, versionType: string): string {
  if (specialIcons[versionId]) return specialIcons[versionId]
  return typeIcons[versionType] || grassIcon
}

// 加载类别
const categories = [
  { id: 'vanilla', label: '原版游戏', icon: CubeIcon },
  { id: 'modloaders', label: '模组加载器', icon: WrenchIcon },
  { id: 'modpacks', label: '整合包', icon: ArchiveBoxIcon },
  { id: 'installed', label: '已安装', icon: CheckCircleIcon },
]

// 子分类
const subCategories = [
  { id: 'release', label: '正式版', icon: StarIcon },
  { id: 'snapshot', label: '快照版', icon: BeakerIcon },
  { id: 'old', label: '远古版', icon: ClockIcon },
]

// 按类型分类的版本（已排序，最新在前）
const releaseVersions = computed(() => {
  return versionStore.getReleaseVersions()
})

const snapshotVersions = computed(() => {
  return versionStore.getSnapshotVersions()
})

const oldVersions = computed(() => {
  return versionStore.versions.filter(
    v => v.version_type === 'old_beta' || v.version_type === 'old_alpha'
  )
})

// 当前显示的版本
const currentVersions = computed(() => {
  if (activeCategory.value === 'installed') {
    return installedVersions.value.map(id => ({
      id,
      version_type: 'release',
      release_time: 0,
    }))
  }

  if (activeCategory.value === 'vanilla') {
    switch (activeSubCategory.value) {
      case 'release':
        return releaseVersions.value
      case 'snapshot':
        return snapshotVersions.value
      case 'old':
        return oldVersions.value
    }
  }

  return []
})

// 监听下载进度
let unlistenProgress: (() => void) | null = null
let unlistenComplete: (() => void) | null = null

onMounted(async () => {
  loading.value = true
  await Promise.all([
    versionStore.fetchVersions(),
    loadInstalledVersions(),
  ])
  loading.value = false

  unlistenProgress = await listen<DownloadProgress>('download-progress', (event) => {
    versionStore.updateProgress(event.payload)
  })

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

function formatDate(timestamp: number): string {
  if (!timestamp) return '未知'
  return new Date(timestamp * 1000).toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  })
}

function isInstalled(versionId: string): boolean {
  return installedVersions.value.includes(versionId)
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

async function handleOpenGameDir() {
  try {
    await tauri.openGameDir()
  } catch (e) {
    console.error('Failed to open game directory:', e)
  }
}
</script>

<template>
  <div class="flex h-full">
    <!-- 左侧侧边栏：加载类别 -->
    <aside class="w-48 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col">
      <div class="flex-1 overflow-y-auto py-4">
        <button
          v-for="category in categories"
          :key="category.id"
          class="w-full flex items-center px-4 py-2.5 text-sm font-medium transition-colors"
          :class="[
            activeCategory === category.id
              ? 'bg-primary-50 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300 border-r-2 border-primary-500'
              : 'text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700'
          ]"
          @click="activeCategory = category.id"
        >
          <component :is="category.icon" class="w-5 h-5 mr-3" />
          {{ category.label }}
        </button>
      </div>

      <div class="p-3 border-t border-gray-200 dark:border-gray-700">
        <button
          class="w-full flex items-center justify-center px-3 py-2 text-xs text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
          @click="handleOpenGameDir"
        >
          <FolderOpenIcon class="w-4 h-4 mr-2" />
          打开游戏目录
        </button>
      </div>
    </aside>

    <!-- 右侧内容区 -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <!-- 分类标题 -->
      <div class="px-6 py-4 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
          {{ activeCategory === 'installed' ? '已安装' : '原版游戏' }}
        </h2>
        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
          {{ activeCategory === 'installed' ? '已下载的版本' : 'Minecraft Java Edition 官方版本' }}
        </p>
      </div>

      <!-- 子分类标签 (仅原版游戏) -->
      <div v-if="activeCategory === 'vanilla'" class="px-6 py-3 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
        <div class="flex space-x-2">
          <button
            v-for="sub in subCategories"
            :key="sub.id"
            class="flex items-center px-3 py-1.5 rounded-lg text-sm transition-colors"
            :class="[
              activeSubCategory === sub.id
                ? 'bg-primary-100 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300'
                : 'text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700'
            ]"
            @click="activeSubCategory = sub.id"
          >
            <component :is="sub.icon" class="w-4 h-4 mr-1.5" />
            {{ sub.label }}
            <span class="ml-1.5 text-xs opacity-75">
              {{ sub.id === 'release' ? releaseVersions.length : sub.id === 'snapshot' ? snapshotVersions.length : oldVersions.length }}
            </span>
          </button>
        </div>
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
        <div v-if="versionStore.downloading && versionStore.downloadProgress" class="mx-6 mt-4 p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
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
          <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
            <div
              class="bg-primary-600 h-2 rounded-full transition-all duration-300"
              :style="{ width: `${versionStore.downloadProgress.percentage}%` }"
            ></div>
          </div>
        </div>
      </transition>

      <!-- 版本列表 -->
      <div class="flex-1 overflow-y-auto">
        <!-- 加载状态 -->
        <div v-if="loading" class="flex items-center justify-center h-full">
          <div class="text-center">
            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600 mx-auto"></div>
            <p class="text-gray-600 dark:text-gray-400 mt-4">加载中...</p>
          </div>
        </div>

        <!-- 版本列表 -->
        <div v-else-if="currentVersions.length > 0" class="divide-y divide-gray-100 dark:divide-gray-700">
          <div
            v-for="version in currentVersions"
            :key="version.id"
            class="flex items-center justify-between px-6 py-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
          >
            <div class="flex items-center">
              <img
                :src="getVersionIcon(version.id, version.version_type)"
                :alt="version.id"
                class="w-8 h-8 rounded mr-3"
              />
              <div>
                <div class="flex items-center">
                  <span class="font-medium text-gray-900 dark:text-gray-100 text-sm">
                    {{ version.id }}
                  </span>
                  <span
                    v-if="version.id === versionStore.latestRelease && activeSubCategory === 'release'"
                    class="ml-2 text-xs px-1.5 py-0.5 rounded-full bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200"
                  >
                    最新
                  </span>
                  <span
                    v-if="version.id === versionStore.latestSnapshot && activeSubCategory === 'snapshot'"
                    class="ml-2 text-xs px-1.5 py-0.5 rounded-full bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200"
                  >
                    最新
                  </span>
                </div>
                <span class="text-xs text-gray-500 dark:text-gray-400">
                  {{ formatDate(version.release_time) }}
                </span>
              </div>
            </div>

            <div class="flex items-center">
              <span
                v-if="isInstalled(version.id)"
                class="text-xs px-2 py-1 rounded-full bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200 mr-3"
              >
                已安装
              </span>
              <button
                v-if="isInstalled(version.id)"
                class="flex items-center px-3 py-1.5 bg-primary-600 text-white text-sm rounded-lg hover:bg-primary-700 transition-colors"
              >
                <PlayIcon class="w-4 h-4 mr-1" />
                启动
              </button>
              <button
                v-else
                class="flex items-center px-3 py-1.5 bg-primary-600 text-white text-sm rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50"
                :disabled="versionStore.downloading"
                @click="handleDownload(version.id)"
              >
                <ArrowDownTrayIcon v-if="versionStore.downloadingVersion !== version.id" class="w-4 h-4 mr-1" />
                {{ versionStore.downloadingVersion === version.id ? '下载中...' : '下载' }}
              </button>
            </div>
          </div>
        </div>

        <!-- 空状态 -->
        <div v-else class="flex items-center justify-center h-full">
          <div class="text-center">
            <CubeIcon class="w-16 h-16 text-gray-400 mx-auto" />
            <p class="text-gray-600 dark:text-gray-400 mt-4">
              {{ activeCategory === 'modloaders' || activeCategory === 'modpacks' ? '即将开放' : '暂无版本' }}
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
