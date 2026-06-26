<script setup lang="ts">
/**
 * 下载页面
 * 左侧侧边栏：加载类别
 * 右侧主页面：展开动画分类
 */

import { ref, computed, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useVersionStore } from '@/stores/version'
import type { DownloadProgress } from '@/stores/version'
import * as tauri from '@/utils/tauri'
import { showError, showConfirm } from '@/utils/modal'
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
  ChevronRightIcon,
  PlayIcon,
  TrashIcon,
  SparklesIcon,
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

// 展开的分类（支持多个同时展开）
const expandedSections = ref<Set<string>>(new Set(['latest']))

function toggleSection(sectionId: string) {
  if (expandedSections.value.has(sectionId)) {
    expandedSections.value.delete(sectionId)
  } else {
    expandedSections.value.add(sectionId)
  }
}

function isSectionExpanded(sectionId: string): boolean {
  return expandedSections.value.has(sectionId)
}

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

// 最新版本
const latestVersions = computed(() => {
  const versions = []
  if (versionStore.latestRelease) {
    const v = versionStore.getVersionById(versionStore.latestRelease)
    if (v) versions.push({ ...v, tag: '正式版' })
  }
  if (versionStore.latestSnapshot) {
    const v = versionStore.getVersionById(versionStore.latestSnapshot)
    if (v) versions.push({ ...v, tag: '快照版' })
  }
  return versions
})

// 版本分类
const sections = computed(() => [
  {
    id: 'latest',
    label: '最新版本',
    icon: SparklesIcon,
    count: latestVersions.value.length,
    versions: latestVersions.value,
  },
  {
    id: 'release',
    label: '正式版',
    icon: StarIcon,
    count: versionStore.getReleaseVersions().length,
    versions: versionStore.getReleaseVersions(),
  },
  {
    id: 'snapshot',
    label: '快照版',
    icon: BeakerIcon,
    count: versionStore.getSnapshotVersions().length,
    versions: versionStore.getSnapshotVersions(),
  },
  {
    id: 'old',
    label: '远古版',
    icon: ClockIcon,
    count: versionStore.versions.filter(v => v.version_type === 'old_beta' || v.version_type === 'old_alpha').length,
    versions: versionStore.versions.filter(v => v.version_type === 'old_beta' || v.version_type === 'old_alpha'),
  },
])

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
    showError('下载失败', `无法下载版本 ${versionId}`, String(e))
    versionStore.finishDownload()
  }
}

async function handleUninstall(versionId: string) {
  showConfirm(
    '卸载版本',
    `确定要卸载版本 ${versionId} 吗？此操作不可撤销。`,
    async () => {
      try {
        await tauri.uninstallVersion(versionId)
        installedVersions.value = installedVersions.value.filter(v => v !== versionId)
      } catch (e) {
        console.error('Failed to uninstall version:', e)
        showError('卸载失败', `无法卸载版本 ${versionId}`, String(e))
      }
    }
  )
}

// 调试：显示版本目录信息
async function debugVersions() {
  try {
    const gameDir = await tauri.getGameDir()
    console.log('Game directory:', gameDir)
    console.log('Installed versions:', installedVersions.value)
  } catch (e) {
    console.error('Debug error:', e)
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
          {{ activeCategory === 'installed' ? '已下载的版本，可启动或卸载' : 'Minecraft Java Edition 官方版本' }}
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

      <!-- 内容列表 -->
      <div class="flex-1 overflow-y-auto p-6">
        <!-- 加载状态 -->
        <div v-if="loading" class="flex items-center justify-center h-full">
          <div class="text-center">
            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600 mx-auto"></div>
            <p class="text-gray-600 dark:text-gray-400 mt-4">加载中...</p>
          </div>
        </div>

        <!-- 原版游戏：展开分类 -->
        <div v-else-if="activeCategory === 'vanilla'" class="space-y-4">
          <div
            v-for="section in sections"
            :key="section.id"
            class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden"
          >
            <!-- 分类标题 -->
            <div
              class="flex items-center justify-between px-4 py-3 cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
              @click="toggleSection(section.id)"
            >
              <div class="flex items-center">
                <component :is="section.icon" class="w-5 h-5 mr-3 text-gray-500 dark:text-gray-400" />
                <span class="font-medium text-gray-900 dark:text-gray-100">{{ section.label }}</span>
                <span class="ml-2 text-xs text-gray-500 dark:text-gray-400">
                  {{ section.count }} 个版本
                </span>
              </div>
              <ChevronRightIcon
                class="w-5 h-5 text-gray-400 transition-transform duration-200"
                :class="{ 'rotate-90': isSectionExpanded(section.id) }"
              />
            </div>

            <!-- 展开内容 -->
            <transition
              enter-active-class="transition-all duration-300 ease-out"
              enter-from-class="max-h-0 opacity-0"
              enter-to-class="max-h-[5000px] opacity-100"
              leave-active-class="transition-all duration-200 ease-in"
              leave-from-class="max-h-[5000px] opacity-100"
              leave-to-class="max-h-0 opacity-0"
            >
              <div v-if="isSectionExpanded(section.id)" class="overflow-hidden">
                <div class="border-t border-gray-100 dark:border-gray-700 divide-y divide-gray-100 dark:divide-gray-700">
                  <div
                    v-for="version in section.versions"
                    :key="version.id"
                    class="flex items-center justify-between px-4 py-2.5 hover:bg-gray-50 dark:hover:bg-gray-700/30 transition-colors"
                  >
                    <div class="flex items-center pl-8">
                      <img
                        :src="getVersionIcon(version.id, version.version_type)"
                        :alt="version.id"
                        class="w-6 h-6 rounded mr-2"
                      />
                      <div>
                        <div class="flex items-center">
                          <span class="text-sm text-gray-900 dark:text-gray-100">{{ version.id }}</span>
                          <span
                            v-if="(version as any).tag"
                            class="ml-2 text-xs px-1.5 py-0.5 rounded-full"
                            :class="(version as any).tag === '正式版' 
                              ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                              : 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'"
                          >
                            {{ (version as any).tag }}
                          </span>
                        </div>
                        <span class="text-xs text-gray-500">{{ formatDate(version.release_time) }}</span>
                      </div>
                    </div>

                    <div class="flex items-center">
                      <span
                        v-if="isInstalled(version.id)"
                        class="text-xs px-2 py-0.5 rounded-full bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200 mr-2"
                      >
                        已安装
                      </span>
                      <button
                        v-if="isInstalled(version.id)"
                        class="flex items-center px-3 py-1 bg-primary-600 text-white text-xs rounded hover:bg-primary-700 transition-colors mr-1"
                      >
                        <PlayIcon class="w-3.5 h-3.5 mr-1" />
                        启动
                      </button>
                      <button
                        v-if="isInstalled(version.id)"
                        class="flex items-center px-2 py-1 bg-red-100 text-red-700 dark:bg-red-900/50 dark:text-red-400 text-xs rounded hover:bg-red-200 dark:hover:bg-red-900 transition-colors"
                        @click="handleUninstall(version.id)"
                      >
                        <TrashIcon class="w-3.5 h-3.5" />
                      </button>
                      <button
                        v-else
                        class="flex items-center px-3 py-1 bg-primary-600 text-white text-xs rounded hover:bg-primary-700 transition-colors disabled:opacity-50"
                        :disabled="versionStore.downloading"
                        @click="handleDownload(version.id)"
                      >
                        <ArrowDownTrayIcon v-if="versionStore.downloadingVersion !== version.id" class="w-3.5 h-3.5 mr-1" />
                        {{ versionStore.downloadingVersion === version.id ? '下载中...' : '下载' }}
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </transition>
          </div>
        </div>

        <!-- 已安装列表 -->
        <div v-else-if="activeCategory === 'installed'" class="space-y-2">
          <div v-if="installedVersions.length === 0" class="flex items-center justify-center h-64">
            <div class="text-center">
              <CubeIcon class="w-16 h-16 text-gray-400 mx-auto" />
              <p class="text-gray-600 dark:text-gray-400 mt-4">暂未安装任何版本</p>
            </div>
          </div>
          <div
            v-for="versionId in installedVersions"
            :key="versionId"
            class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 flex items-center justify-between"
          >
            <div class="flex items-center">
              <img
                :src="getVersionIcon(versionId, 'release')"
                :alt="versionId"
                class="w-10 h-10 rounded mr-3"
              />
              <div>
                <h3 class="font-semibold text-gray-900 dark:text-gray-100">{{ versionId }}</h3>
                <p class="text-xs text-gray-500">已安装</p>
              </div>
            </div>
            <div class="flex items-center space-x-2">
              <button class="flex items-center px-4 py-2 bg-primary-600 text-white text-sm rounded-lg hover:bg-primary-700 transition-colors">
                <PlayIcon class="w-4 h-4 mr-1" />
                启动
              </button>
              <button
                class="flex items-center px-3 py-2 bg-red-100 text-red-700 dark:bg-red-900/50 dark:text-red-400 text-sm rounded-lg hover:bg-red-200 dark:hover:bg-red-900 transition-colors"
                @click="handleUninstall(versionId)"
              >
                <TrashIcon class="w-4 h-4 mr-1" />
                卸载
              </button>
            </div>
          </div>
        </div>

        <!-- 其他分类（即将开放） -->
        <div v-else class="flex items-center justify-center h-full">
          <div class="text-center">
            <CubeIcon class="w-16 h-16 text-gray-400 mx-auto" />
            <p class="text-gray-600 dark:text-gray-400 mt-4">即将开放</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
