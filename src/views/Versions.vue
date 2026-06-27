<script setup lang="ts">
/**
 * 下载页面
 * 左侧侧边栏：加载类别
 * 右侧主页面：展开动画分类
 */

import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useVersionStore } from '@/stores/version'
import * as tauri from '@/utils/tauri'
import { showError, showConfirm } from '@/utils/modal'
import { showSuccess, showInfo } from '@/utils/toast'
import LoaderSelect from './LoaderSelect.vue'
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

import grassIcon from '@/assets/blocks/Grass.png'
import cobblestoneIcon from '@/assets/blocks/CobbleStone.png'
import commandBlockIcon from '@/assets/blocks/CommandBlock.png'
import goldBlockIcon from '@/assets/blocks/GoldBlock.png'

const versionStore = useVersionStore()

let pollTimer: ReturnType<typeof setInterval> | null = null
let lastPercentage = 0

const stageNames: Record<number, string> = {
  0: '版本清单', 1: '版本 JSON', 2: '客户端 JAR', 3: '库文件',
  4: '资源文件', 5: 'Natives', 6: '解压 Natives', 7: '模组', 8: '整合包',
}

function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec <= 0) return ''
  if (bytesPerSec >= 1024 * 1024) return (bytesPerSec / 1024 / 1024).toFixed(1) + ' MB/s'
  if (bytesPerSec >= 1024) return (bytesPerSec / 1024).toFixed(0) + ' KB/s'
  return bytesPerSec + ' B/s'
}

function formatBytes(bytes: number): string {
  if (bytes <= 0) return '0'
  if (bytes >= 1024 * 1024 * 1024) return (bytes / 1024 / 1024 / 1024).toFixed(1) + ' GB'
  if (bytes >= 1024 * 1024) return (bytes / 1024 / 1024).toFixed(0) + ' MB'
  if (bytes >= 1024) return (bytes / 1024).toFixed(0) + ' KB'
  return bytes + ' B'
}

function startPolling() {
  if (pollTimer) return
  lastPercentage = 0
  pollTimer = setInterval(async () => {
    try {
      const snapshot = await tauri.getDownloadProgress()
      console.log('[poll]', JSON.stringify(snapshot))
      let percentage = 0
      if (snapshot.bytes_total > 0) {
        percentage = (snapshot.bytes_downloaded / snapshot.bytes_total) * 100
      } else if (snapshot.total > 0) {
        percentage = (snapshot.current / snapshot.total) * 100
      }
      percentage = Math.max(percentage, lastPercentage)
      percentage = Math.min(percentage, 100)
      lastPercentage = percentage

      if (snapshot.is_active || percentage > 0) {
        versionStore.updateProgress({
          stage: stageNames[snapshot.stage] || `阶段 ${snapshot.stage}`,
          current: snapshot.current,
          total: snapshot.total,
          percentage,
          speed: snapshot.speed,
          bytesDownloaded: snapshot.bytes_downloaded,
          bytesTotal: snapshot.bytes_total,
          filesRemaining: snapshot.files_remaining,
        })
      }

      if (snapshot.is_complete) {
        stopPolling()
        await loadInstalledVersions()
        versionStore.finishDownload()
        showSuccess(`${versionStore.downloadingVersion} 下载完成`)
      } else if (snapshot.error_code !== 0) {
        stopPolling()
        showError('下载失败', `错误码: ${snapshot.error_code}`, '')
        versionStore.finishDownload()
      }
    } catch (e) {
      console.error('Failed to poll progress:', e)
    }
  }, 300)
}

function stopPolling() {
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
}

onMounted(async () => {
  loading.value = true
  await Promise.all([
    versionStore.fetchVersions(),
    loadInstalledVersions(),
  ])
  loading.value = false
})

onUnmounted(() => {
  stopPolling()
})

const loading = ref(false)
const installedVersions = ref<string[]>([])
const activeCategory = ref('vanilla')
const selectedVersion = ref<string | null>(null)

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

const categories = [
  { id: 'vanilla', label: '原版游戏', icon: CubeIcon },
  { id: 'modloaders', label: '模组加载器', icon: WrenchIcon },
  { id: 'modpacks', label: '整合包', icon: ArchiveBoxIcon },
  { id: 'installed', label: '已安装', icon: CheckCircleIcon },
]

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
  console.log('[download] 开始:', versionId)
  lastPercentage = 0
  versionStore.startDownload(versionId)
  showInfo(`开始下载 ${versionId}`)
  startPolling()
  try {
    console.log('[download] 调用 downloadVersion...')
    await tauri.downloadVersion(versionId)
    console.log('[download] downloadVersion 返回')
    stopPolling()
    await loadInstalledVersions()
    versionStore.finishDownload()
    showSuccess(`${versionId} 下载完成`)
  } catch (e) {
    stopPolling()
    console.error('[download] 失败:', e)
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
        showSuccess(`${versionId} 已卸载`)
      } catch (e) {
        console.error('Failed to uninstall version:', e)
        showError('卸载失败', `无法卸载版本 ${versionId}`, String(e))
      }
    }
  )
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
  <div class="flex h-full rounded-xl overflow-hidden bg-white shadow-sm">
    <!-- 左侧侧边栏 -->
    <aside class="w-48 bg-white border-r border-gray-200 flex flex-col shrink-0">
      <div class="flex-1 overflow-y-auto py-4">
        <button
          v-for="category in categories"
          :key="category.id"
          class="w-full flex items-center px-4 py-2.5 text-sm font-medium transition-colors"
          :class="[
            activeCategory === category.id
              ? 'bg-primary-50 text-primary-700 border-r-2 border-primary-500'
              : 'text-gray-700 hover:bg-gray-50'
          ]"
          @click="activeCategory = category.id"
        >
          <component :is="category.icon" class="w-5 h-5 mr-3" />
          {{ category.label }}
        </button>
      </div>

      <div class="p-3 border-t border-gray-200">
        <button
          class="w-full flex items-center justify-center px-3 py-2 text-xs text-gray-600 hover:bg-gray-50 rounded-lg transition-colors"
          @click="handleOpenGameDir"
        >
          <FolderOpenIcon class="w-4 h-4 mr-2" />
          打开游戏目录
        </button>
      </div>
    </aside>

    <!-- 右侧内容区 -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <div class="px-6 py-4 bg-white border-b border-gray-200 shrink-0">
        <h2 class="text-lg font-semibold text-gray-900">
          {{ activeCategory === 'installed' ? '已安装' : '原版游戏' }}
        </h2>
        <p class="text-xs text-gray-500 mt-1">
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
        <div v-if="versionStore.downloading" class="mx-6 mt-4 p-3 bg-white rounded-lg border border-gray-200">
          <!-- 状态行 -->
          <div class="flex items-center gap-3">
            <div class="animate-spin rounded-full h-4 w-4 border-2 border-gray-300 border-t-primary-600"></div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between">
                <span class="text-sm font-medium text-gray-900">
                  正在下载 {{ versionStore.downloadingVersion }}
                </span>
                <span class="text-xs text-gray-500">
                  {{ versionStore.downloadProgress?.stage || '准备中...' }}
                </span>
              </div>
            </div>
          </div>
          <!-- 进度条 -->
          <div class="mt-3">
            <!-- 有百分比进度时：确定进度条 -->
            <div v-if="(versionStore.downloadProgress?.percentage || 0) > 0" class="w-full bg-gray-100 rounded-full overflow-hidden" style="height: 6px">
              <div
                class="h-full bg-primary-500 rounded-full transition-all duration-300"
                :style="{ width: `${versionStore.downloadProgress.percentage}%` }"
              ></div>
            </div>
            <!-- 无百分比进度时：动画扫动进度条 -->
            <div v-else class="w-full bg-gray-100 rounded-full overflow-hidden" style="height: 6px">
              <div class="h-full bg-primary-400 rounded-full animate-sweep" style="width: 30%"></div>
            </div>
            <div class="flex items-center justify-between mt-2 text-xs text-gray-500">
              <span>
                <template v-if="versionStore.downloadProgress?.bytesTotal && versionStore.downloadProgress.bytesTotal > 0">
                  {{ formatBytes(versionStore.downloadProgress.bytesDownloaded) }} / {{ formatBytes(versionStore.downloadProgress.bytesTotal) }}
                </template>
                <template v-else-if="versionStore.downloadProgress?.total && versionStore.downloadProgress.total > 0">
                  {{ versionStore.downloadProgress.current }}/{{ versionStore.downloadProgress.total }} 文件
                </template>
                <template v-else>
                  正在处理...
                </template>
              </span>
              <span v-if="versionStore.downloadProgress?.speed && versionStore.downloadProgress.speed > 0">
                {{ formatSpeed(versionStore.downloadProgress.speed) }}
              </span>
            </div>
          </div>
        </div>
      </transition>

      <!-- 内容列表 -->
      <div class="flex-1 overflow-y-auto">
        <transition name="slide-right" mode="out-in">
          <!-- 加载器选择模式 -->
          <LoaderSelect
            v-if="selectedVersion"
            :key="'loader-' + selectedVersion"
            :mc-version="selectedVersion"
            @back="selectedVersion = null"
            @installing="selectedVersion = null"
          />

          <!-- 版本列表模式 -->
          <div v-else key="version-list" class="p-6">
          <!-- 加载状态 -->
        <div v-if="loading" class="flex items-center justify-center h-full">
          <div class="text-center">
            <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600 mx-auto"></div>
            <p class="text-gray-600 mt-4">加载中...</p>
          </div>
        </div>

        <!-- 原版游戏：展开分类 -->
        <div v-else-if="activeCategory === 'vanilla'" class="space-y-4">
          <div
            v-for="section in sections"
            :key="section.id"
            class="bg-white rounded-lg border border-gray-200 overflow-hidden"
          >
            <div
              class="flex items-center justify-between px-4 py-3 cursor-pointer hover:bg-gray-50 transition-colors"
              @click="toggleSection(section.id)"
            >
              <div class="flex items-center">
                <component :is="section.icon" class="w-5 h-5 mr-3 text-gray-500" />
                <span class="font-medium text-gray-900">{{ section.label }}</span>
                <span class="ml-2 text-xs text-gray-500">
                  {{ section.count }} 个版本
                </span>
              </div>
              <ChevronRightIcon
                class="w-5 h-5 text-gray-400 transition-transform duration-200"
                :class="{ 'rotate-90': isSectionExpanded(section.id) }"
              />
            </div>

            <div
              class="grid transition-all duration-500 ease-in-out"
              :style="{ gridTemplateRows: isSectionExpanded(section.id) ? '1fr' : '0fr' }"
            >
              <div class="overflow-hidden min-h-0">
                <div class="border-t border-gray-100 divide-y divide-gray-100">
                  <div
                    v-for="version in section.versions"
                    :key="version.id"
                    class="flex items-center justify-between px-4 py-2.5 hover:bg-gray-50 transition-colors"
                  >
                    <div class="flex items-center pl-8">
                      <img
                        :src="getVersionIcon(version.id, version.version_type)"
                        :alt="version.id"
                        class="w-6 h-6 rounded mr-2"
                      />
                      <div>
                        <div class="flex items-center">
                          <span class="text-sm text-gray-900">{{ version.id }}</span>
                          <span
                            v-if="(version as any).tag"
                            class="ml-2 text-xs px-1.5 py-0.5 rounded-full"
                            :class="(version as any).tag === '正式版'
                              ? 'bg-green-100 text-green-800'
                              : 'bg-yellow-100 text-yellow-800'"
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
                        class="text-xs px-2 py-0.5 rounded-full bg-green-100 text-green-800 mr-2"
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
                        class="flex items-center px-2 py-1 bg-red-100 text-red-700 text-xs rounded hover:bg-red-200 transition-colors"
                        @click="handleUninstall(version.id)"
                      >
                        <TrashIcon class="w-3.5 h-3.5" />
                      </button>
                      <button
                        v-else
                        class="flex items-center px-3 py-1 bg-primary-600 text-white text-xs rounded hover:bg-primary-700 transition-colors disabled:opacity-50"
                        :disabled="versionStore.downloading"
                        @click="selectedVersion = version.id"
                      >
                        <ArrowDownTrayIcon class="w-3.5 h-3.5 mr-1" />
                        安装
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 已安装列表 -->
        <div v-else-if="activeCategory === 'installed'" class="space-y-2">
          <div v-if="installedVersions.length === 0" class="flex items-center justify-center h-64">
            <div class="text-center">
              <CubeIcon class="w-16 h-16 text-gray-400 mx-auto" />
              <p class="text-gray-600 mt-4">暂未安装任何版本</p>
            </div>
          </div>
          <div
            v-for="versionId in installedVersions"
            :key="versionId"
            class="bg-white rounded-lg border border-gray-200 p-4 flex items-center justify-between"
          >
            <div class="flex items-center">
              <img
                :src="getVersionIcon(versionId, 'release')"
                :alt="versionId"
                class="w-10 h-10 rounded mr-3"
              />
              <div>
                <h3 class="font-semibold text-gray-900">{{ versionId }}</h3>
                <p class="text-xs text-gray-500">已安装</p>
              </div>
            </div>
            <div class="flex items-center space-x-2">
              <button class="flex items-center px-4 py-2 bg-primary-600 text-white text-sm rounded-lg hover:bg-primary-700 transition-colors">
                <PlayIcon class="w-4 h-4 mr-1" />
                启动
              </button>
              <button
                class="flex items-center px-3 py-2 bg-red-100 text-red-700 text-sm rounded-lg hover:bg-red-200 transition-colors"
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
            <p class="text-gray-600 mt-4">即将开放</p>
          </div>
        </div>
        </div>
        </transition>
      </div>
    </div>
  </div>
</template>

<style scoped>
.slide-right-enter-active {
  transition: transform 0.3s ease-out, opacity 0.2s ease-out;
}
.slide-right-leave-active {
  transition: transform 0.2s ease-in, opacity 0.15s ease-in;
}
.slide-right-enter-from {
  transform: translateX(100%);
  opacity: 0;
}
.slide-right-leave-to {
  transform: translateX(-30%);
  opacity: 0;
}
</style>
