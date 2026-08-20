/**
 * 启动状态 Pinia store（从 composables/useLaunchState.ts 下沉）
 *
 * 封装启动游戏流程（进度 + Java 自动下载监听）、运行中状态（pid / 版本 ID）与游戏退出事件监听；
 * 与版本列表解耦，验证由调用方负责。
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { listen } from '@tauri-apps/api/event'
import * as tauri from '@/utils/tauri'
import { toastSuccess, toastError, toastWarning } from '@/utils/toast'
import { showCrashDialog } from '@/utils/crashDialog'
import { maybeTriggerLaunchHints } from '@/utils/buyHint'
import { safeCall } from '@/utils/async'
import { useOnlineStore } from '@/stores/online'
import type { CrashInfo } from '@/types/version'

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

/** 启动进度事件名（与后端 pipeline/execute.rs 的 emit 对应） */
const LAUNCH_PROGRESS_EVENT = 'launch-progress'

export const useLaunchStore = defineStore('launch', () => {
  // 启动状态
  const launching = ref(false)
  const launchingVersionId = ref<string | null>(null)
  const runningPid = ref<number | null>(null)
  const runningVersionId = ref<string | null>(null)
  const launchProgress = ref<tauri.LaunchProgress | null>(null)
  let launchProgressUnlisten: (() => void) | null = null

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
    const unlisten = await safeCall(() => listen<GameExitEvent>('game-exited', (event) => {
      const { is_normal, exit_code, crash_info } = event.payload
      runningPid.value = null
      runningVersionId.value = null
      if (is_normal) {
        toastSuccess('游戏已退出')
      } else if (crash_info) {
        // 弹出崩溃分析对话框
        showCrashDialog(crash_info)
      } else {
        toastError(`游戏已退出（代码: ${exit_code}）`)
      }
    }), 'setup game exit listener')
    if (unlisten) unlistenFn = unlisten
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
    /** 临时追加的 JVM 参数（单次启动有效，不写入 setup.ini）
     *  用途：联机模块启动 MC 时追加 -Djava.net.preferIPv4Stack=true */
    extraJvmArgs?: string[]
  }): Promise<number> {
    launching.value = true
    launchingVersionId.value = params.versionId
    launchProgress.value = null

    await startProgressListener()
    await startJavaDownloadListener()

    try {
      // 联机模块自动注入：若当前在房间内（房主或加入方），追加 -Djava.net.preferIPv4Stack=true
      // 确保 MC 优先使用 IPv4 网络栈，避免虚拟局域网（TUN 接口为 IPv4）通信失败
      // 调用方显式传入的 extraJvmArgs 优先级最高，自动注入只在其未传时生效
      let extraJvmArgs = params.extraJvmArgs
      if (!extraJvmArgs) {
        const onlineStore = useOnlineStore()
        if (onlineStore.roomState.role === 'host' || onlineStore.roomState.role === 'guest') {
          extraJvmArgs = ['-Djava.net.preferIPv4Stack=true']
        }
      }
      const pid = await tauri.launchGame({ ...params, extraJvmArgs })
      runningPid.value = pid
      runningVersionId.value = params.versionId
      toastSuccess(`游戏已启动（PID: ${pid}）`)
      // 启动成功：自增计数并检查正版购买 / 点 Star 提示（非阻塞，不影响启动）
      void maybeTriggerLaunchHints()
      return pid
    } catch (e) {
      console.error('Failed to launch game:', e)
      // 用户主动取消启动：调用方已提示「已取消启动」，此处不再重复弹错误
      const msg = e instanceof Error ? e.message : String(e)
      if (!msg.includes('启动已取消')) {
        toastError(msg)
      }
      throw e
    } finally {
      stopProgressListener()
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
      toastWarning('游戏已停止')
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
    const pid = await safeCall(() => tauri.getRunningGame(), 'check running game')
    if (pid !== undefined) runningPid.value = pid
  }

  /** 监听启动进度事件（后端 pipeline 每步 update_progress 时 emit，替代 200ms 轮询） */
  async function startProgressListener() {
    await stopProgressListener()
    launchProgressUnlisten = await listen<tauri.LaunchProgress>(
      LAUNCH_PROGRESS_EVENT,
      (e) => {
        launchProgress.value = e.payload
      },
    )
  }

  async function stopProgressListener() {
    if (launchProgressUnlisten) {
      launchProgressUnlisten()
      launchProgressUnlisten = null
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
})