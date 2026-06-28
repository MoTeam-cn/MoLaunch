<script setup lang="ts">
/**
 * 下载页面
 */

import { ref, computed, onMounted } from 'vue'
import { useVersionStore } from '@/stores/version'
import * as tauri from '@/utils/tauri'
import { showError, showConfirm } from '@/utils/modal'
import { showSuccess, showInfo } from '@/utils/toast'
import { useDownloadPolling } from '@/composables/useDownloadPolling'
import Tooltip from '@/components/common/Tooltip.vue'
import LoaderSelect from './LoaderSelect.vue'
import VersionSection from '@/components/version/VersionSection.vue'
import InstalledList from '@/components/version/InstalledList.vue'
import {
  CubeIcon, WrenchIcon, ArchiveBoxIcon, CheckCircleIcon,
  FolderOpenIcon, StarIcon, BeakerIcon, ClockIcon, SparklesIcon,
  ArrowPathIcon,
} from '@heroicons/vue/24/outline'

import grassIcon from '@/assets/blocks/Grass.png'
import cobblestoneIcon from '@/assets/blocks/CobbleStone.png'
import commandBlockIcon from '@/assets/blocks/CommandBlock.png'
import goldBlockIcon from '@/assets/blocks/GoldBlock.png'

const versionStore = useVersionStore()
const { startPolling, stopPolling } = useDownloadPolling()

const loading = ref(false)
const installedVersions = ref<string[]>([])
const activeCategory = ref('vanilla')
const selectedVersion = ref<string | null>(null)

const typeIcons: Record<string, string> = {
  release: grassIcon, snapshot: commandBlockIcon,
  old_beta: cobblestoneIcon, old_alpha: cobblestoneIcon,
}
const specialIcons: Record<string, string> = {
  '23w13a_or_b': goldBlockIcon, '20w14infinite': goldBlockIcon,
  '22w13oneblockatatime': goldBlockIcon, '24w14potato': goldBlockIcon,
  '25w14craftmine': goldBlockIcon,
}

function getVersionIcon(id: string, type: string): string {
  return specialIcons[id] || typeIcons[type] || grassIcon
}

function formatDate(ts: number): string {
  if (!ts) return '未知'
  return new Date(ts * 1000).toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' })
}

const categories = [
  { id: 'vanilla', label: '原版游戏', icon: CubeIcon },
  { id: 'modloaders', label: '模组加载器', icon: WrenchIcon },
  { id: 'modpacks', label: '整合包', icon: ArchiveBoxIcon },
  { id: 'installed', label: '已安装', icon: CheckCircleIcon },
]

const latestVersions = computed(() => {
  const list = []
  if (versionStore.latestRelease) {
    const v = versionStore.getVersionById(versionStore.latestRelease)
    if (v) list.push({ ...v, tag: '正式版' })
  }
  if (versionStore.latestSnapshot) {
    const v = versionStore.getVersionById(versionStore.latestSnapshot)
    if (v) list.push({ ...v, tag: '快照版' })
  }
  return list
})

const sections = computed(() => [
  { id: 'latest', label: '最新版本', icon: SparklesIcon, versions: latestVersions.value },
  { id: 'release', label: '正式版', icon: StarIcon, versions: versionStore.getReleaseVersions() },
  { id: 'snapshot', label: '快照版', icon: BeakerIcon, versions: versionStore.getSnapshotVersions() },
  { id: 'old', label: '远古版', icon: ClockIcon, versions: versionStore.versions.filter(v => v.version_type === 'old_beta' || v.version_type === 'old_alpha') },
])

async function loadInstalledVersions() {
  try { installedVersions.value = await tauri.listInstalledVersions() } catch (e) { console.error(e) }
}

async function handleRefresh() {
  showInfo('正在刷新版本列表...')
  await versionStore.refreshVersions()
  await loadInstalledVersions()
  showSuccess('版本列表已刷新')
}

function onInstallRequest(options: { mcVersion: string; forge?: string; neoforge?: string; fabric?: string; optifine?: string; liteloader?: string; instanceName: string }) {
  // 立刻返回版本列表
  selectedVersion.value = null
  // 设置下载状态，显示 DownloadPanel
  versionStore.startDownload(options.instanceName)
  // 后台执行安装
  tauri.installMerged(
    options.mcVersion,
    options.forge,
    options.neoforge,
    options.fabric,
    options.optifine,
    options.liteloader,
    options.instanceName,
  ).then(async () => {
    showSuccess(`${options.instanceName} 安装完成`)
    await loadInstalledVersions()
  }).catch((e) => {
    showError('安装失败', String(e))
  }).finally(() => {
    versionStore.finishDownload()
  })
}

async function handleDownload(versionId: string) {
  versionStore.startDownload(versionId)
  showInfo(`开始下载 ${versionId}`)
  startPolling()
  try {
    await tauri.downloadVersion(versionId)
    stopPolling()
    await loadInstalledVersions()
    versionStore.finishDownload()
    showSuccess(`${versionId} 下载完成`)
  } catch (e) {
    stopPolling()
    showError('下载失败', `无法下载版本 ${versionId}`, String(e))
    versionStore.finishDownload()
  }
}

function handleUninstall(versionId: string) {
  showConfirm('卸载版本', `确定要卸载版本 ${versionId} 吗？此操作不可撤销。`, async () => {
    try {
      await tauri.uninstallVersion(versionId)
      installedVersions.value = installedVersions.value.filter(v => v !== versionId)
      showSuccess(`${versionId} 已卸载`)
    } catch (e) { showError('卸载失败', `无法卸载版本 ${versionId}`, String(e)) }
  })
}

onMounted(async () => {
  // 有缓存就不显示 loading
  if (versionStore.versions.length === 0) {
    loading.value = true
  }
  await Promise.all([versionStore.fetchVersions(), loadInstalledVersions()])
  loading.value = false
})
</script>

<template>
  <div class="flex h-full rounded-xl overflow-hidden bg-white shadow-sm">
    <!-- 左侧菜单 -->
    <aside class="w-48 bg-white border-r border-gray-200 flex flex-col shrink-0">
      <div class="flex-1 overflow-y-auto py-4">
        <button
          v-for="cat in categories"
          :key="cat.id"
          class="w-full flex items-center px-4 py-2.5 text-sm font-medium transition-colors"
          :class="activeCategory === cat.id
            ? 'bg-primary-50 text-primary-700 border-r-2 border-primary-500'
            : 'text-gray-700 hover:bg-gray-50'"
          @click="activeCategory = cat.id"
        >
          <component :is="cat.icon" class="w-5 h-5 mr-3" />
          {{ cat.label }}
        </button>
      </div>
      <div class="p-3 border-t border-gray-200">
        <button
          class="w-full flex items-center justify-center px-3 py-2 text-xs text-gray-600 hover:bg-gray-50 rounded-lg transition-colors"
          @click="tauri.openGameDir()"
        >
          <FolderOpenIcon class="w-4 h-4 mr-2" />
          打开游戏目录
        </button>
      </div>
    </aside>

    <!-- 右侧内容 -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <div class="px-6 py-4 bg-white border-b border-gray-200 flex items-center justify-between shrink-0">
        <div>
          <h2 class="text-lg font-semibold text-gray-900">
            {{ activeCategory === 'installed' ? '已安装' : '原版游戏' }}
          </h2>
          <p class="text-xs text-gray-500 mt-1">
            {{ activeCategory === 'installed' ? '已下载的版本，可启动或卸载' : 'Minecraft Java Edition 官方版本' }}
          </p>
        </div>
        <Tooltip text="刷新版本列表" position="bottom">
          <button
            class="p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
            @click="handleRefresh"
          >
            <ArrowPathIcon class="w-4 h-4" />
          </button>
        </Tooltip>
      </div>

      <div class="flex-1 overflow-y-auto">
        <transition name="slide-right" mode="out-in">
          <LoaderSelect
            v-if="selectedVersion"
            :key="'loader-' + selectedVersion"
            :mc-version="selectedVersion"
            @back="selectedVersion = null"
            @install="onInstallRequest"
          />

          <div v-else key="version-list" class="p-6 h-full">
            <div v-if="loading" class="flex items-center justify-center h-full">
              <div class="text-center">
                <div class="animate-spin rounded-full h-10 w-10 border-2 border-gray-200 border-t-primary-600 mx-auto"></div>
                <p class="text-sm text-gray-500 mt-3">加载中...</p>
              </div>
            </div>

            <div v-else-if="activeCategory === 'vanilla'" class="space-y-4">
              <VersionSection
                v-for="(section, idx) in sections"
                :id="section.id"
                :key="section.id"
                :label="section.label"
                :icon="section.icon"
                :versions="section.versions"
                :installed-ids="installedVersions"
                :downloading="versionStore.downloading"
                :default-expanded="idx === 0"
                :format-date="formatDate"
                :get-version-icon="getVersionIcon"
                @download="selectedVersion = $event"
                @uninstall="handleUninstall"
              />
            </div>

            <InstalledList
              v-else-if="activeCategory === 'installed'"
              :versions="installedVersions"
              :get-version-icon="getVersionIcon"
              @uninstall="handleUninstall"
            />

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
.slide-right-enter-active { transition: transform 0.3s ease-out, opacity 0.2s ease-out; }
.slide-right-leave-active { transition: transform 0.2s ease-in, opacity 0.15s ease-in; }
.slide-right-enter-from { transform: translateX(100%); opacity: 0; }
.slide-right-leave-to { transform: translateX(-30%); opacity: 0; }
</style>
