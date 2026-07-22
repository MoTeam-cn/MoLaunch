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
import { safeCall } from '@/utils/async'

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
    safeCall(() => tauri.setSelectedVersion(val), 'persist selectedVersion')
  })

  /** 从 config.ini 恢复上次选中的版本（启动器启动时调用）
   *
   * @param installedList 可选，已获取的已安装版本列表，避免重复调用 IPC
   */
  async function restoreSelectedVersion(installedList?: { id: string }[]) {
    await safeCall(async () => {
      const saved = await tauri.getSelectedVersion()
      if (saved) {
        // 验证版本是否仍然存在（复用传入的列表或重新获取）
        const installed = installedList ?? await tauri.listInstalledVersionsWithType()
        if (installed.some((v) => v.id === saved)) {
          selectedVersion.value = saved
        } else {
          // 版本已不存在，清空持久化
          await tauri.setSelectedVersion(null)
        }
      }
    }, 'restore selectedVersion')
  }

  /** 快速恢复上次选中的版本（仅读取 config，不校验是否仍存在）
   *
   * 用于首页首屏快速显示版本名 + 启用开始游戏按钮，避免阻塞在磁盘扫描上。
   * 后续应调用 `validateSelectedVersion` 校验版本是否仍然存在。
   */
  async function restoreSelectedVersionFast() {
    const saved = await safeCall(() => tauri.getSelectedVersion(), 'restore selectedVersion (fast)')
    if (saved) selectedVersion.value = saved
  }

  /** 校验当前 selectedVersion 是否仍存在于已安装列表
   *
   * - 不存在则清空持久化，并自动回退到第一个已安装版本（若有）
   * - 为空且 installedList 非空时自动选中第一个
   */
  async function validateSelectedVersion(installedList: { id: string }[]) {
    const current = selectedVersion.value
    if (current && installedList.some((v) => v.id === current)) {
      return // 仍然有效
    }
    // 当前版本无效或为空
    if (current) {
      // 版本已不存在，清空持久化
      await safeCall(() => tauri.setSelectedVersion(null), 'clear selectedVersion')
    }
    // 自动回退到第一个已安装版本
    selectedVersion.value = installedList.length > 0 ? installedList[0].id : null
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
    restoreSelectedVersionFast,
    validateSelectedVersion,
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
