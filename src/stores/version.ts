/**
 * 版本状态管理
 */

import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { VersionInfo, VersionList } from '@/types/version'
import * as tauri from '@/utils/tauri'

export interface DownloadStage {
  name: string
  progress: number
  weight: number
  status: 'waiting' | 'loading' | 'finished' | 'failed'
  bytes_downloaded: number
  bytes_total: number
  files_downloaded: number
  files_total: number
}

export interface DownloadProgress {
  stages: DownloadStage[]
  current_stage_index: number
  global_speed: number
  global_bytes_downloaded: number
  global_bytes_total: number
  percentage: number
}

export const useVersionStore = defineStore('version', () => {
  // 状态
  const versions = ref<VersionInfo[]>([])
  const latestRelease = ref('')
  const latestSnapshot = ref('')
  const loading = ref(false)
  const error = ref<string | null>(null)
  
  // 下载状态
  const downloading = ref(false)
  const downloadingVersion = ref<string | null>(null)
  const downloadProgress = ref<DownloadProgress | null>(null)
  
  // 版本选择器状态（用于在页面切换时保持状态）
  const selectedVersion = ref<string | null>(null)
  
  // 加载器版本列表缓存（按 MC 版本号缓存）
  const loaderVersionsCache = ref<Record<string, {
    forge: string[]
    neoforge: { version: string; recommended: boolean }[]
    fabric: { version: string; stable: boolean }[]
    optifine: { display_name: string; is_preview: boolean }[]
    liteloader: string[]
  }>>({})

  // 方法
  async function fetchVersions() {
    // 已有数据则不重复请求
    if (versions.value.length > 0) return

    loading.value = true
    error.value = null
    
    try {
      const result = await tauri.listVersions()
      versions.value = result.versions
      latestRelease.value = result.latest_release
      latestSnapshot.value = result.latest_snapshot
    } catch (e) {
      error.value = String(e)
      console.error('Failed to fetch versions:', e)
    } finally {
      loading.value = false
    }
  }

  function refreshVersions() {
    versions.value = []
    return fetchVersions()
  }

  function startDownload(versionId: string) {
    downloading.value = true
    downloadingVersion.value = versionId
    downloadProgress.value = null
  }

  function updateProgress(progress: DownloadProgress) {
    downloadProgress.value = progress
  }

  function finishDownload() {
    downloading.value = false
    downloadingVersion.value = null
    downloadProgress.value = null
  }

  function getVersionById(id: string): VersionInfo | undefined {
    return versions.value.find(v => v.id === id)
  }

  function getReleaseVersions(): VersionInfo[] {
    return versions.value.filter(v => v.version_type === 'release')
  }

  function getSnapshotVersions(): VersionInfo[] {
    return versions.value.filter(v => v.version_type === 'snapshot')
  }
  
  // 获取加载器缓存
  function getLoaderCache(mcVersion: string) {
    return loaderVersionsCache.value[mcVersion] || null
  }
  
  // 设置加载器缓存
  function setLoaderCache(mcVersion: string, data: {
    forge: string[]
    neoforge: { version: string; recommended: boolean }[]
    fabric: { version: string; stable: boolean }[]
    optifine: { display_name: string; is_preview: boolean }[]
    liteloader: string[]
  }) {
    loaderVersionsCache.value[mcVersion] = data
  }

  return {
    versions,
    latestRelease,
    latestSnapshot,
    loading,
    error,
    downloading,
    downloadingVersion,
    downloadProgress,
    selectedVersion,
    loaderVersionsCache,
    fetchVersions,
    refreshVersions,
    startDownload,
    updateProgress,
    finishDownload,
    getVersionById,
    getReleaseVersions,
    getSnapshotVersions,
    getLoaderCache,
    setLoaderCache,
  }
})
