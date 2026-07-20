/**
 * 启动状态管理 composable
 *
 * 从 stores/version.ts 抽出，封装：
 * - 启动游戏流程（轮询进度、Java 自动下载进度监听）
 * - 运行中游戏状态（pid、版本 ID）
 * - 游戏退出事件监听
 *
 * 与版本列表本身解耦：调用 launchGame 时直接传 launchGameParams，
 * 不依赖 versions 数组（验证由调用方负责）。
 */

import { ref, computed } from 'vue'
import { listen } from '@tauri-apps/api/event'
import * as tauri from '@/utils/tauri'
import { showSuccess, showError, showWarning } from '@/utils/toast'
import { showCrashDialog } from '@/utils/crashDialog'

/** 崩溃类别（与后端 CrashCategory 枚举对应） */
type CrashCategory = 'Java' | 'Memory' | 'Graphics' | 'Mod' | 'Forge' | 'Fabric' | 'OptiFine' | 'Unknown'

/** 崩溃详情（与后端 CrashInfo 结构对应） */
interface CrashInfo {
  reason: string
  category: CrashCategory
  log_lines: string[]
  suggestion: string
  problematic_mod: string | null
  crash_report_path?: string
  log_tail: string[]
}

/** 游戏退出事件 payload */
interface GameExitEvent {
  pid: number
  version_id: string
  exit_code: number
  is_normal: boolean
  crash_info?: CrashInfo
}

/** 启动进度阶段名映射（后端枚举 → 中文显示） */
const STAGE_NAMES: Record<string, string> = {
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

export function useLaunchState() {
  // 启动状态
  const launching = ref(false)
  const launchingVersionId = ref<string | null>(null)
  const runningPid = ref<number | null>(null)
  const runningVersionId = ref<string | null>(null)
  const launchProgress = ref<tauri.LaunchProgress | null>(null)
  let launchProgressTimer: number | null = null

  // Java 自动下载进度（启动时自动下载 Java 用，与版本设置页的独立下载共享事件）
  const javaDownloadProgress = ref<tauri.JavaDownloadProgress | null>(null)
  let javaDownloadUnlisten: (() => void) | null = null

  async function startJavaDownloadListener() {
    if (javaDownloadUnlisten) return
    javaDownloadUnlisten = await listen<tauri.JavaDownloadProgress>(
      tauri.JAVA_DOWNLOAD_PROGRESS_EVENT,
      (e) => { javaDownloadProgress.value = e.payload },
    )
  }
  function stopJavaDownloadListener() {
    if (javaDownloadUnlisten) {
      javaDownloadUnlisten()
      javaDownloadUnlisten = null
    }
    javaDownloadProgress.value = null
  }

  // 监听游戏退出事件
  let unlistenFn: (() => void) | null = null

  async function setupGameExitListener() {
    try {
      unlistenFn = await listen<GameExitEvent>('game-exited', (event) => {
        const { is_normal, exit_code, crash_info } = event.payload
        runningPid.value = null
        runningVersionId.value = null
        if (is_normal) {
          showSuccess('游戏已退出')
        } else if (crash_info) {
          // 弹出崩溃分析对话框
          showCrashDialog(crash_info)
        } else {
          showError(`游戏已退出（代码: ${exit_code}）`)
        }
      })
    } catch (e) {
      console.error('Failed to setup game exit listener:', e)
    }
  }

  function cleanupGameExitListener() {
    if (unlistenFn) {
      unlistenFn()
      unlistenFn = null
    }
  }

  // 初始化时设置监听器
  setupGameExitListener()

  /** 启动游戏 */
  async function launchGame(params: {
    versionId: string
    javaPath?: string
    username: string
    uuid: string
    loginType?: string
    windowWidth?: number
    windowHeight?: number
    serverAddress?: string
    serverPort?: number
  }): Promise<number> {
    launching.value = true
    launchingVersionId.value = params.versionId
    launchProgress.value = null

    startProgressPolling()
    await startJavaDownloadListener()

    try {
      const pid = await tauri.launchGame(params)
      runningPid.value = pid
      runningVersionId.value = params.versionId
      showSuccess(`游戏已启动（PID: ${pid}）`)
      return pid
    } catch (e) {
      console.error('Failed to launch game:', e)
      showError(e instanceof Error ? e.message : String(e))
      throw e
    } finally {
      stopProgressPolling()
      stopJavaDownloadListener()
      launching.value = false
      launchingVersionId.value = null
    }
  }

  /** 停止运行中的游戏 */
  async function stopGame(): Promise<void> {
    try {
      await tauri.stopGame()
      runningPid.value = null
      runningVersionId.value = null
      showWarning('游戏已停止')
    } catch (e) {
      console.error('Failed to stop game:', e)
      throw e
    }
  }

  /** 取消正在进行的启动 */
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

  /** 检查当前是否有运行中的游戏（启动器启动时调用） */
  async function checkRunningGame(): Promise<void> {
    try {
      const pid = await tauri.getRunningGame()
      runningPid.value = pid
    } catch (e) {
      console.error('Failed to check running game:', e)
    }
  }

  /** 启动进度轮询（每 200ms 拉取一次后端进度） */
  function startProgressPolling() {
    stopProgressPolling()
    launchProgressTimer = window.setInterval(async () => {
      try {
        const progress = await tauri.getLaunchProgress()
        if (progress) {
          launchProgress.value = progress
          if (progress.stage === 'Finished' || progress.stage === 'Failed') {
            stopProgressPolling()
          }
        }
      } catch (e) {
        console.error('Failed to get launch progress:', e)
      }
    }, 200)
  }

  function stopProgressPolling() {
    if (launchProgressTimer) {
      clearInterval(launchProgressTimer)
      launchProgressTimer = null
    }
  }

  /** 启动阶段名称（中文） */
  const launchStageName = computed(() => {
    if (!launchProgress.value) return ''
    return STAGE_NAMES[launchProgress.value.stage] || launchProgress.value.stage
  })

  return {
    launching,
    launchingVersionId,
    runningPid,
    runningVersionId,
    launchProgress,
    launchStageName,
    javaDownloadProgress,
    launchGame,
    stopGame,
    cancelLaunch,
    checkRunningGame,
    cleanupGameExitListener,
  }
}
