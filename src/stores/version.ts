/**
 * 版本状态管理
 *
 * 启动相关逻辑（launchGame/stopGame/cancelLaunch/进度轮询/Java 下载进度/游戏退出监听）
 * 已拆分到 `composables/useLaunchState.ts`，本 store 通过 composable 委托调用。
 */

import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import type { VersionInfo } from '@/types/version'
import type { DownloadProgress } from '@/types/download'
import * as tauri from '@/utils/tauri'
import { useLaunchState } from '@/composables/useLaunchState'

// 重新导出 DownloadStage/DownloadProgress，保持向后兼容（其他文件可能从 '@/stores/version' import）
export type { DownloadStage, DownloadProgress } from '@/types/download'

export const useVersionStore = defineStore('version', () => {
  // 版本列表状态
  const versions = ref<VersionInfo[]>([])
  const latestRelease = ref('')
  const latestSnapshot = ref('')
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 下载状态
  const downloading = ref(false)
  const downloadingVersion = ref<string | null>(null)
  const downloadProgress = ref<DownloadProgress | null>(null)

  // 启动状态（委托给 useLaunchState composable）
  const {
    launching, launchingVersionId, runningPid, runningVersionId,
    launchProgress, launchStageName, javaDownloadProgress,
    launchGame, stopGame, cancelLaunch, checkRunningGame, cleanupGameExitListener,
  } = useLaunchState()

  // 版本选择器状态（用于在页面切换时保持状态）
  const selectedVersion = ref<string | null>(null)

  // 持久化 selectedVersion：变化时自动保存到 config.ini
  watch(selectedVersion, (val) => {
    tauri.setSelectedVersion(val).catch((e) => {
      console.error('Failed to persist selectedVersion:', e)
    })
  })

  /** 从 config.ini 恢复上次选中的版本（启动器启动时调用） */
  async function restoreSelectedVersion() {
    try {
      const saved = await tauri.getSelectedVersion()
      if (saved) {
        // 验证版本是否仍然存在
        const installed = await tauri.listInstalledVersionsWithType()
        if (installed.some((v) => v.id === saved)) {
          selectedVersion.value = saved
        } else {
          // 版本已不存在，清空持久化
          await tauri.setSelectedVersion(null)
        }
      }
    } catch (e) {
      console.error('Failed to restore selectedVersion:', e)
    }
  }

  // 加载器版本列表缓存（按 MC 版本号缓存）
  const loaderVersionsCache = ref<Record<string, {
    forge: string[]
    neoforge: { version: string; recommended: boolean }[]
    fabric: { version: string; stable: boolean }[]
    optifine: { display_name: string; is_preview: boolean }[]
    liteloader: string[]
  }>>({})

  /** 拉取版本清单（已有数据则跳过） */
  async function fetchVersions() {
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
      throw e
    } finally {
      loading.value = false
    }
  }

  /** 强制重新拉取版本清单 */
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

  /** 获取加载器缓存 */
  function getLoaderCache(mcVersion: string) {
    return loaderVersionsCache.value[mcVersion] || null
  }

  /** 设置加载器缓存 */
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
    launching,
    launchingVersionId,
    runningPid,
    runningVersionId,
    launchProgress,
    launchStageName,
    javaDownloadProgress,
    selectedVersion,
    restoreSelectedVersion,
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
    launchGame,
    stopGame,
    cancelLaunch,
    checkRunningGame,
    cleanupGameExitListener,
  }
})
