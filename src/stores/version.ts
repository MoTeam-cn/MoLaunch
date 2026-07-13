/**
 * 版本状态管理
 */

import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { showSuccess, showWarning } from '@/utils/toast'
import type { VersionInfo } from '@/types/version'
import * as tauri from '@/utils/tauri'
import { listen } from '@tauri-apps/api/event'

// 游戏退出事件类型
interface GameExitEvent {
  pid: number
  version_id: string
  exit_code: number
  is_normal: boolean
}

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
  
  // 启动状态
  const launching = ref(false)
  const launchingVersionId = ref<string | null>(null) // 当前正在启动的版本ID
  const runningPid = ref<number | null>(null)
  const runningVersionId = ref<string | null>(null) // 当前正在运行的版本ID
  const launchProgress = ref<tauri.LaunchProgress | null>(null)
  let launchProgressTimer: number | null = null
  
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

  // 监听游戏退出事件
  let unlistenFn: (() => void) | null = null
  
  let exitPollTimer: number | null = null

  // 开始轮询游戏退出
  function startExitPolling() {
    stopExitPolling()
    exitPollTimer = window.setInterval(async () => {
      try {
        const pid = await tauri.getRunningGame()
        if (pid === null && runningVersionId.value) {
          runningPid.value = null
          runningVersionId.value = null
          showSuccess('游戏已退出')
          stopExitPolling()
        }
      } catch {
        // ignore
      }
    }, 1000)
  }

  function stopExitPolling() {
    if (exitPollTimer !== null) {
      clearInterval(exitPollTimer)
      exitPollTimer = null
    }
  }

  async function setupGameExitListener() {
    try {
      unlistenFn = await listen<GameExitEvent>('game-exited', (event) => {
        // 游戏退出：清理运行状态（不再 console.log，避免生产环境噪音）
        if (import.meta.env.DEV) {
          console.debug('[GameExit]', event.payload)
        }
        runningPid.value = null
        runningVersionId.value = null
        stopExitPolling()
      })
    } catch (e) {
      console.error('Failed to setup game exit listener:', e)
    }
  }
  
  // 清理监听器
  function cleanupGameExitListener() {
    if (unlistenFn) {
      unlistenFn()
      unlistenFn = null
    }
  }
  
  // 初始化时设置监听器
  setupGameExitListener()
  
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
      throw e // 重新抛出错误，让调用者处理
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

  // 启动游戏
  async function launchGame(params: {
    versionId: string
    javaPath?: string
    username: string
    uuid: string
    accessToken: string
    loginType?: string
    windowWidth?: number
    windowHeight?: number
    serverAddress?: string
    serverPort?: number
  }): Promise<number> {
    launching.value = true
    launchingVersionId.value = params.versionId
    launchProgress.value = null
    
    // 启动进度轮询
    startProgressPolling()
    
    try {
      const pid = await tauri.launchGame(params)
      runningPid.value = pid
      runningVersionId.value = params.versionId
      // 开始轮询检测游戏退出
      startExitPolling()
      return pid
    } catch (e) {
      console.error('Failed to launch game:', e)
      throw e
    } finally {
      // 启动完成后停止轮询
      stopProgressPolling()
      launching.value = false
      launchingVersionId.value = null
    }
  }

  // 停止游戏
  async function stopGame(): Promise<void> {
    try {
      await tauri.stopGame()
      runningPid.value = null
      runningVersionId.value = null
      stopExitPolling()
      showWarning('游戏已停止')
    } catch (e) {
      console.error('Failed to stop game:', e)
      throw e
    }
  }

  // 取消启动
  async function cancelLaunch(): Promise<void> {
    try {
      await tauri.cancelLaunch()
      launching.value = false
      launchingVersionId.value = null
      launchProgress.value = null
    } catch (e) {
      console.error('Failed to cancel launch:', e)
      throw e
    }
  }

  // 检查运行状态
  async function checkRunningGame(): Promise<void> {
    try {
      const pid = await tauri.getRunningGame()
      runningPid.value = pid
    } catch (e) {
      console.error('Failed to check running game:', e)
    }
  }

  // 开始进度轮询
  function startProgressPolling() {
    stopProgressPolling()
    launchProgressTimer = window.setInterval(async () => {
      try {
        const progress = await tauri.getLaunchProgress()
        if (progress) {
          launchProgress.value = progress
          // 如果完成或失败，停止轮询
          if (progress.stage === 'Finished' || progress.stage === 'Failed') {
            stopProgressPolling()
          }
        }
      } catch (e) {
        console.error('Failed to get launch progress:', e)
      }
    }, 200)
  }

  // 停止进度轮询
  function stopProgressPolling() {
    if (launchProgressTimer) {
      clearInterval(launchProgressTimer)
      launchProgressTimer = null
    }
  }

  // 计算属性：启动阶段名称
  const launchStageName = computed(() => {
    if (!launchProgress.value) return ''
    const stageNames: Record<string, string> = {
      'Init': '初始化',
      'GetJava': '获取Java',
      'Login': '登录验证',
      'ValidateFiles': '文件检查',
      'BuildArgs': '构建参数',
      'ExtractNatives': '解压原生库',
      'LaunchProcess': '启动进程',
      'WaitWindow': '等待窗口',
      'Finished': '完成',
      'Failed': '失败',
    }
    return stageNames[launchProgress.value.stage] || launchProgress.value.stage
  })

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
