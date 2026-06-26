<script setup lang="ts">
/**
 * 下载页面
 * 左侧侧边栏：加载类别
 * 右侧主页面：分类列表，点击展开
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
  ChevronRightIcon,
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

// 展开的分类
const expandedGroups = ref<Set<string>>(new Set())

function toggleGroup(groupId: string) {
  if (expandedGroups.value.has(groupId)) {
    expandedGroups.value.delete(groupId)
  } else {
    expandedGroups.value.add(groupId)
  }
}

function isGroupExpanded(groupId: string): boolean {
  return expandedGroups.value.has(groupId)
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

// 按大版本号分组
function groupByVersion(versions: { id: string; version_type: string; release_time: number }[]) {
  const groupMap = new Map<string, typeof versions>()

  for (const v of versions) {
    const parts = v.id.split('.')
    const major = parts.length >= 2 ? `${parts[0]}.${parts[1]}` : v.id

    if (!groupMap.has(major)) {
      groupMap.set(major, [])
    }
    groupMap.get(major)!.push(v)
  }

  return Array.from(groupMap.entries())
    .sort((a, b) => b[0].localeCompare(a[0]))
    .map(([major, versions]) => ({
      id: major,
      label: `${major}.x`,
      versions,
    }))
}

// 正式版分组
const releaseGroups = computed(() => {
  return groupByVersion(versionStore.getReleaseVersions())
})

// 快照版分组
const snapshotGroups = computed(() => {
  return groupByVersion(versionStore.getSnapshotVersions())
})

// 远古版本
const oldVersions = computed(() => {
  return versionStore.versions.filter(
    v => v.version_type === 'old_beta' || v.version_type === 'old_alpha'
  )
})

// 当前显示的内容
const currentContent = computed(() => {
  switch (activeCategory.value) {
    case 'vanilla':
      return {
        title: '原版游戏',
        description: 'Minecraft Java Edition 官方版本',
        type: 'groups' as const,
        groups: [
          { id: 'release', label: '正式版', icon: StarIcon, groups: releaseGroups.value },
          { id: 'snapshot', label: '快照版', icon: BeakerIcon, groups: snapshotGroups.value },
          { id: 'old', label: '远古版', icon: ClockIcon, versions: oldVersions.value.slice(0, 50) },
        ],
      }
    case 'modloaders':
      return {
        title: '模组加载器',
        description: '即将开放',
        type: 'empty' as const,
      }
    case 'modpacks':
      return {
        title: '整合包',
        description: '即将开放',
        type: 'empty' as const,
      }
    case 'installed':
      return {
        title: '已安装',
        description: '已下载的版本',
        type: 'list' as const,
        versions: installedVersions.value.map(id => ({
          id,
          version_type: 'release',
          release_time: 0,
        })),
      }
    default:
      return { title: '', description: '', type: 'empty' as const }
  }
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
          {{ currentContent.title }}
        </h2>
        <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
          {{ currentContent.description }}
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

        <!-- 分组内容 -->
        <div v-else-if="currentContent.type === 'groups'" class="space-y-4">
          <div v-for="section in currentContent.groups" :key="section.id" class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
            <!-- 分组标题 -->
            <div
              class="flex items-center justify-between px-4 py-3 cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
              @click="toggleGroup(section.id)"
            >
              <div class="flex items-center">
                <component :is="section.icon" class="w-5 h-5 mr-3 text-gray-500 dark:text-gray-400" />
                <span class="font-medium text-gray-900 dark:text-gray-100">{{ section.label }}</span>
                <span class="ml-2 text-xs text-gray-500 dark:text-gray-400">
                  {{ section.groups ? section.groups.length + ' 个版本组' : (section.versions?.length || 0) + ' 个版本' }}
                </span>
              </div>
              <ChevronRightIcon
                class="w-5 h-5 text-gray-400 transition-transform duration-200"
                :class="{ 'rotate-90': isGroupExpanded(section.id) }"
              />
            </div>

            <!-- 展开内容 -->
            <transition
              enter-active-class="transition-all duration-300 ease-out"
              enter-from-class="max-h-0 opacity-0"
              enter-to-class="max-h-[2000px] opacity-100"
              leave-active-class="transition-all duration-200 ease-in"
              leave-from-class="max-h-[2000px] opacity-100"
              leave-to-class="max-h-0 opacity-0"
            >
              <div v-if="isGroupExpanded(section.id)" class="overflow-hidden">
                <!-- 版本组 -->
                <div v-if="section.groups" class="border-t border-gray-100 dark:border-gray-700">
                  <div v-for="group in section.groups" :key="group.id" class="border-b border-gray-100 dark:border-gray-700 last:border-b-0">
                    <!-- 版本组标题 -->
                    <div
                      class="flex items-center justify-between px-4 py-2 pl-12 cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700/30 transition-colors"
                      @click="toggleGroup(`${section.id}-${group.id}`)"
                    >
                      <span class="text-sm text-gray-700 dark:text-gray-300">{{ group.label }}</span>
                      <div class="flex items-center">
                        <span class="text-xs text-gray-500 mr-2">{{ group.versions.length }}</span>
                        <ChevronRightIcon
                          class="w-4 h-4 text-gray-400 transition-transform duration-200"
                          :class="{ 'rotate-90': isGroupExpanded(`${section.id}-${group.id}`) }"
                        />
                      </div>
                    </div>

                    <!-- 版本列表 -->
                    <transition
                      enter-active-class="transition-all duration-300 ease-out"
                      enter-from-class="max-h-0 opacity-0"
                      enter-to-class="max-h-[1000px] opacity-100"
                      leave-active-class="transition-all duration-200 ease-in"
                      leave-from-class="max-h-[1000px] opacity-100"
                      leave-to-class="max-h-0 opacity-0"
                    >
                      <div v-if="isGroupExpanded(`${section.id}-${group.id}`)" class="overflow-hidden">
                        <div class="px-4 py-2 pl-16 space-y-1">
                          <div
                            v-for="version in group.versions"
                            :key="version.id"
                            class="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/30 transition-colors"
                          >
                            <div class="flex items-center">
                              <img
                                :src="getVersionIcon(version.id, version.version_type)"
                                :alt="version.id"
                                class="w-6 h-6 rounded mr-2"
                              />
                              <span class="text-sm text-gray-900 dark:text-gray-100">{{ version.id }}</span>
                              <span class="ml-2 text-xs text-gray-500">{{ formatDate(version.release_time) }}</span>
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
                                class="text-xs px-3 py-1 bg-primary-600 text-white rounded hover:bg-primary-700 transition-colors"
                              >
                                <PlayIcon class="w-3.5 h-3.5 inline mr-1" />
                                启动
                              </button>
                              <button
                                v-else
                                class="text-xs px-3 py-1 bg-primary-600 text-white rounded hover:bg-primary-700 transition-colors disabled:opacity-50"
                                :disabled="versionStore.downloading"
                                @click="handleDownload(version.id)"
                              >
                                <ArrowDownTrayIcon v-if="versionStore.downloadingVersion !== version.id" class="w-3.5 h-3.5 inline mr-1" />
                                {{ versionStore.downloadingVersion === version.id ? '下载中...' : '下载' }}
                              </button>
                            </div>
                          </div>
                        </div>
                      </div>
                    </transition>
                  </div>
                </div>

                <!-- 直接版本列表 -->
                <div v-else-if="section.versions" class="border-t border-gray-100 dark:border-gray-700 px-4 py-2 pl-12 space-y-1">
                  <div
                    v-for="version in section.versions"
                    :key="version.id"
                    class="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/30 transition-colors"
                  >
                    <div class="flex items-center">
                      <img
                        :src="getVersionIcon(version.id, version.version_type)"
                        :alt="version.id"
                        class="w-6 h-6 rounded mr-2"
                      />
                      <span class="text-sm text-gray-900 dark:text-gray-100">{{ version.id }}</span>
                      <span class="ml-2 text-xs text-gray-500">{{ formatDate(version.release_time) }}</span>
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
                        class="text-xs px-3 py-1 bg-primary-600 text-white rounded hover:bg-primary-700 transition-colors"
                      >
                        <PlayIcon class="w-3.5 h-3.5 inline mr-1" />
                        启动
                      </button>
                      <button
                        v-else
                        class="text-xs px-3 py-1 bg-primary-600 text-white rounded hover:bg-primary-700 transition-colors disabled:opacity-50"
                        :disabled="versionStore.downloading"
                        @click="handleDownload(version.id)"
                      >
                        <ArrowDownTrayIcon v-if="versionStore.downloadingVersion !== version.id" class="w-3.5 h-3.5 inline mr-1" />
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
        <div v-else-if="currentContent.type === 'list' && currentContent.versions" class="space-y-2">
          <div
            v-for="version in currentContent.versions"
            :key="version.id"
            class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 flex items-center justify-between"
          >
            <div class="flex items-center">
              <img
                :src="getVersionIcon(version.id, version.version_type)"
                :alt="version.id"
                class="w-10 h-10 rounded mr-3"
              />
              <div>
                <h3 class="font-semibold text-gray-900 dark:text-gray-100">{{ version.id }}</h3>
                <p class="text-xs text-gray-500">已安装</p>
              </div>
            </div>
            <button class="text-sm px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors">
              <PlayIcon class="w-4 h-4 inline mr-1" />
              启动
            </button>
          </div>
        </div>

        <!-- 空状态 -->
        <div v-else class="flex items-center justify-center h-full">
          <div class="text-center">
            <CubeIcon class="w-16 h-16 text-gray-400 mx-auto" />
            <p class="text-gray-600 dark:text-gray-400 mt-4">
              即将开放
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
