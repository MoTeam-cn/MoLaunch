<script setup lang="ts">
/**
 * 下载页面 - PCL2 风格
 * 侧边栏：加载类别
 * 主内容：版本列表
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
  {
    id: 'vanilla',
    label: '原版游戏',
    icon: CubeIcon,
    subCategories: [
      { id: 'release', label: '正式版', icon: StarIcon },
      { id: 'snapshot', label: '快照版', icon: BeakerIcon },
      { id: 'old', label: '远古版', icon: ClockIcon },
    ],
  },
  {
    id: 'modloaders',
    label: '模组加载器',
    icon: WrenchIcon,
    subCategories: [
      { id: 'forge', label: 'Forge', icon: WrenchIcon },
      { id: 'fabric', label: 'Fabric', icon: WrenchIcon },
      { id: 'neoforge', label: 'NeoForge', icon: WrenchIcon },
      { id: 'optifine', label: 'OptiFine', icon: WrenchIcon },
    ],
  },
  {
    id: 'modpacks',
    label: '整合包',
    icon: ArchiveBoxIcon,
    subCategories: [
      { id: 'curseforge', label: 'CurseForge', icon: ArchiveBoxIcon },
      { id: 'modrinth', label: 'Modrinth', icon: ArchiveBoxIcon },
    ],
  },
  {
    id: 'installed',
    label: '已安装',
    icon: CheckCircleIcon,
    subCategories: [],
  },
]

// 当前分类
const currentCategory = computed(() => {
  return categories.find(c => c.id === activeCategory.value)
})

// 显示的版本列表
const displayedVersions = computed(() => {
  if (activeCategory.value === 'installed') {
    return installedVersions.value.map(id => ({
      id,
      version_type: 'release',
      release_time: 0,
    }))
  }

  if (activeCategory.value === 'vanilla') {
    if (activeSubCategory.value === 'release') {
      return versionStore.getReleaseVersions()
    }
    if (activeSubCategory.value === 'snapshot') {
      return versionStore.getSnapshotVersions()
    }
    if (activeSubCategory.value === 'old') {
      return versionStore.versions.filter(
        v => v.version_type === 'old_beta' || v.version_type === 'old_alpha'
      )
    }
  }

  // 模组加载器和整合包暂时显示空
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

function handleCategoryClick(categoryId: string) {
  activeCategory.value = categoryId
  const category = categories.find(c => c.id === categoryId)
  if (category && category.subCategories.length > 0) {
    activeSubCategory.value = category.subCategories[0].id
  }
}

function handleSubCategoryClick(subCategoryId: string) {
  activeSubCategory.value = subCategoryId
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
      <!-- 分类列表 -->
      <div class="flex-1 overflow-y-auto py-4">
        <div
          v-for="category in categories"
          :key="category.id"
          class="mb-2"
        >
          <!-- 一级分类 -->
          <button
            class="w-full flex items-center px-4 py-2.5 text-sm font-medium transition-colors"
            :class="[
              activeCategory === category.id
                ? 'bg-primary-50 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300 border-r-2 border-primary-500'
                : 'text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-gray-700'
            ]"
            @click="handleCategoryClick(category.id)"
          >
            <component :is="category.icon" class="w-5 h-5 mr-3" />
            {{ category.label }}
          </button>

          <!-- 二级分类 -->
          <div v-if="activeCategory === category.id && category.subCategories.length > 0">
            <button
              v-for="sub in category.subCategories"
              :key="sub.id"
              class="w-full flex items-center pl-12 pr-4 py-2 text-xs transition-colors"
              :class="[
                activeSubCategory === sub.id
                  ? 'text-primary-600 dark:text-primary-400 bg-primary-50/50 dark:bg-primary-900/30'
                  : 'text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700/50'
              ]"
              @click="handleSubCategoryClick(sub.id)"
            >
              {{ sub.label }}
            </button>
          </div>
        </div>
      </div>

      <!-- 底部：打开游戏目录 -->
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
      <!-- 分类描述 -->
      <div class="px-6 py-3 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between">
          <div>
            <h2 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
              {{ currentCategory?.label }}
              <span v-if="currentCategory?.subCategories.length" class="font-normal text-gray-500">
                / {{ currentCategory?.subCategories.find(s => s.id === activeSubCategory)?.label }}
              </span>
            </h2>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
              {{ activeCategory === 'installed' ? '已下载的版本' : '选择版本进行下载' }}
            </p>
          </div>
          <div class="text-xs text-gray-500 dark:text-gray-400">
            共 {{ displayedVersions.length }} 个版本
          </div>
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
      <div class="flex-1 overflow-y-auto p-6">
        <!-- 加载状态 -->
        <div v-if="loading" class="flex items-center justify-center h-full">
          <div class="text-center">
            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600 mx-auto"></div>
            <p class="text-gray-600 dark:text-gray-400 mt-4">加载中...</p>
          </div>
        </div>

        <!-- 版本卡片 -->
        <div v-else-if="displayedVersions.length > 0" class="grid grid-cols-1 lg:grid-cols-2 gap-3">
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
                  class="w-10 h-10 rounded mr-3"
                />
                <div>
                  <h3 class="font-semibold text-gray-900 dark:text-gray-100 text-sm">
                    {{ version.id }}
                  </h3>
                  <p class="text-xs text-gray-500 dark:text-gray-400">
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
                  class="btn-primary text-xs px-3 py-1.5"
                >
                  启动
                </button>
                <button
                  v-else
                  class="btn-primary text-xs px-3 py-1.5"
                  :disabled="versionStore.downloading"
                  @click="handleDownload(version.id)"
                >
                  <ArrowDownTrayIcon v-if="versionStore.downloadingVersion !== version.id" class="w-3.5 h-3.5 mr-1" />
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
