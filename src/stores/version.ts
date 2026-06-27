/**
 * 版本状态管理
 */

import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { VersionInfo, VersionList } from '@/types/version'
import * as tauri from '@/utils/tauri'

export interface DownloadProgress {
  stage: string
  current: number
  total: number
  percentage: number
  speed: number
  bytesDownloaded: number
  bytesTotal: number
  filesRemaining: number
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

  return {
    versions,
    latestRelease,
    latestSnapshot,
    loading,
    error,
    downloading,
    downloadingVersion,
    downloadProgress,
    fetchVersions,
    refreshVersions,
    startDownload,
    updateProgress,
    finishDownload,
    getVersionById,
    getReleaseVersions,
    getSnapshotVersions,
  }
})
