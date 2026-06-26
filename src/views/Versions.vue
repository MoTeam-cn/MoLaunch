<script setup lang="ts">
/**
 * 版本管理页面 - PCL2 风格
 */

import { ref, computed, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useVersionStore } from '@/stores/version'
import type { DownloadProgress } from '@/stores/version'
import type { SidebarItem } from '@/components/common/Sidebar.vue'
import Sidebar from '@/components/common/Sidebar.vue'
import * as tauri from '@/utils/tauri'

// 导入 Blocks 图片
import grassIcon from '@/assets/blocks/Grass.png'
import cobblestoneIcon from '@/assets/blocks/CobbleStone.png'
import commandBlockIcon from '@/assets/blocks/CommandBlock.png'
import goldBlockIcon from '@/assets/blocks/GoldBlock.png'

const versionStore = useVersionStore()

const loading = ref(false)
const installedVersions = ref<string[]>([])
const selectedCategory = ref('latest-release')
const selectedVersion = ref<string | null>(null)

// 版本类型图标
const typeIcons: Record<string, string> = {
  release: grassIcon,
  snapshot: commandBlockIcon,
  old_beta: cobblestoneIcon,
  old_alpha: cobblestoneIcon,
}

// 特殊版本图标
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

// 侧边栏分类
const sidebarItems = computed<SidebarItem[]>(() => {
  const items: SidebarItem[] = []

  // 最新版本置顶
  if (versionStore.latestRelease) {
    items.push({
      id: 'latest-release',
      label: '最新正式版',
      description: versionStore.latestRelease,
      icon: '🟢',
    })
  }
  if (versionStore.latestSnapshot) {
    items.push({
      id: 'latest-snapshot',
      label: '最新快照版',
      description: versionStore.latestSnapshot,
      icon: '🟡',
    })
  }

  // 已安装版本
  if (installedVersions.value.length > 0) {
    items.push({
      id: 'installed',
      label: '已安装',
      badge: installedVersions.value.length,
      icon: '📦',
      children: installedVersions.value.map(v => ({
        id: `installed-${v}`,
        label: v,
      })),
    })
  }

  // 正式版分类
  const releaseVersions = versionStore.getReleaseVersions()
  if (releaseVersions.length > 0) {
    // 按大版本号分组
    const groups = groupVersions(releaseVersions)
    items.push({
      id: 'release',
      label: '正式版',
      badge: releaseVersions.length,
      icon: '🎮',
      children: groups,
    })
  }

  // 快照版分类
  const snapshotVersions = versionStore.getSnapshotVersions()
  if (snapshotVersions.length > 0) {
    const groups = groupVersions(snapshotVersions)
    items.push({
      id: 'snapshot',
      label: '快照版',
      badge: snapshotVersions.length,
      icon: '🧪',
      children: groups,
    })
  }

  // 远古版本
  const oldVersions = versionStore.versions.filter(
    v => v.version_type === 'old_beta' || v.version_type === 'old_alpha'
  )
  if (oldVersions.length > 0) {
    items.push({
      id: 'old',
      label: '远古版本',
      badge: oldVersions.length,
      icon: '📜',
      children: oldVersions.slice(0, 20).map(v => ({
        id: `old-${v.id}`,
        label: v.id,
      })),
    })
  }

  return items
})

// 按大版本号分组
function groupVersions(versions: { id: string; version_type: string }[]): SidebarItem[] {
  const groupMap = new Map<string, SidebarItem>()

  for (const v of versions) {
    // 提取主版本号 (如 1.20.1 -> 1.20)
    const parts = v.id.split('.')
    const major = parts.length >= 2 ? `${parts[0]}.${parts[1]}` : v.id

    if (!groupMap.has(major)) {
      groupMap.set(major, {
        id: `group-${major}`,
        label: `${major}.x`,
        children: [],
      })
    }

    groupMap.get(major)!.children!.push({
      id: `version-${v.id}`,
      label: v.id,
    })
  }

  return Array.from(groupMap.values()).reverse()
}

// 当前显示的版本列表
const displayedVersions = computed(() => {
  if (selectedCategory.value === 'latest-release') {
    const v = versionStore.getVersionById(versionStore.latestRelease)
    return v ? [v] : []
  }
  if (selectedCategory.value === 'latest-snapshot') {
    const v = versionStore.getVersionById(versionStore.latestSnapshot)
    return v ? [v] : []
  }
  if (selectedCategory.value === 'installed') {
    return installedVersions.value.map(id => ({
      id,
      version_type: 'release',
      release_time: 0,
    }))
  }
  if (selectedCategory.value.startsWith('installed-')) {
    const id = selectedCategory.value.replace('installed-', '')
    return [{ id, version_type: 'release', release_time: 0 }]
  }
  if (selectedCategory.value.startsWith('version-')) {
    const id = selectedCategory.value.replace('version-', '')
    const v = versionStore.getVersionById(id)
    return v ? [v] : []
  }
  if (selectedCategory.value.startsWith('old-')) {
    const id = selectedCategory.value.replace('old-', '')
    const v = versionStore.getVersionById(id)
    return v ? [v] : []
  }
  return []
})

// 侧边栏描述
const sidebarDescription = computed(() => {
  if (selectedCategory.value === 'latest-release') return '最新稳定版本'
  if (selectedCategory.value === 'latest-snapshot') return '测试新功能'
  if (selectedCategory.value === 'installed') return '已下载的版本'
  return '选择版本进行下载'
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

function handleSidebarSelect(id: string) {
  selectedCategory.value = id
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
    <!-- 左侧侧边栏 -->
    <Sidebar
      :items="sidebarItems"
      :active-id="selectedCategory"
      title="版本列表"
      description="选择要下载或启动的版本"
      @select="handleSidebarSelect"
    />

    <!-- 右侧内容区 -->
    <div class="flex-1 flex flex-col overflow-hidden">
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
      <div class="flex-1 overflow-y-auto p-6">
        <!-- 加载状态 -->
        <div v-if="loading" class="flex items-center justify-center h-full">
          <div class="text-center">
            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600 mx-auto"></div>
            <p class="text-gray-600 dark:text-gray-400 mt-4">加载版本列表...</p>
          </div>
        </div>

        <!-- 版本卡片 -->
        <div v-else-if="displayedVersions.length > 0" class="space-y-3">
          <div
            v-for="version in displayedVersions"
            :key="version.id"
            class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 hover:shadow-md transition-shadow"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center">
                <img
                  :src="getVersionIcon(version.id, version.version_type)"
                  :alt="version.id"
                  class="w-12 h-12 rounded mr-4"
                />
                <div>
                  <h3 class="font-semibold text-gray-900 dark:text-gray-100">
                    {{ version.id }}
                  </h3>
                  <p class="text-sm text-gray-500 dark:text-gray-400">
                    {{ formatDate(version.release_time) }}
                  </p>
                </div>
              </div>

              <div class="flex items-center space-x-2">
                <span
                  v-if="isInstalled(version.id)"
                  class="text-xs px-2 py-1 rounded-full bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200"
                >
                  已安装
                </span>
                <button
                  v-if="isInstalled(version.id)"
                  class="btn-primary text-sm"
                >
                  启动
                </button>
                <button
                  v-else
                  class="btn-primary text-sm"
                  :disabled="versionStore.downloading"
                  @click="handleDownload(version.id)"
                >
                  <span v-if="versionStore.downloadingVersion === version.id">下载中...</span>
                  <span v-else>下载</span>
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- 空状态 -->
        <div v-else class="flex items-center justify-center h-full">
          <div class="text-center">
            <img :src="grassIcon" alt="Grass" class="w-16 h-16 mx-auto opacity-50" />
            <p class="text-gray-600 dark:text-gray-400 mt-4">
              从左侧选择版本分类
            </p>
          </div>
        </div>
      </div>

      <!-- 底部操作栏 -->
      <div class="p-4 border-t border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
        <button
          class="btn-secondary w-full flex items-center justify-center"
          @click="handleOpenGameDir"
        >
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z" />
          </svg>
          打开游戏目录
        </button>
      </div>
    </div>
  </div>
</template>
