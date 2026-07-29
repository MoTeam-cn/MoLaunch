/**
 * 客户端自动更新工具（module-level 单例）
 *
 * 设计参考：`utils/modal.ts` / `utils/toast.ts` 的 module-level 单例模式。
 * - 状态用 `reactive` 暴露，组件可直接 watch
 * - `checkForUpdate` / `downloadAndInstall` / `closeDialog` 为纯函数，可在任意位置调用
 * - `initAutoCheck` 在 App.vue 启动时调用一次，注册启动 5s + 每 6h 定时检查
 *
 * 配套组件：`src/components/about/UpdateDialog.vue`
 *
 * See: docs/updater/design.md §4.2
 */

import { reactive } from 'vue'
import { check, type Update, type DownloadEvent } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { toastInfo, toastError } from '@/utils/toast'

export type UpdateStatus =
  | 'idle'
  | 'checking'
  | 'available'
  | 'no-update'
  | 'downloading'
  | 'installing'
  | 'done'
  | 'error'

export interface UpdateState {
  /** 当前状态 */
  status: UpdateStatus
  /** 新版本号 */
  version: string
  /** 更新日志 */
  notes: string
  /** 是否强制更新（来自 manifest 扩展字段） */
  forceUpdate: boolean
  /** 已下载字节数 */
  downloaded: number
  /** 总字节数 */
  total: number
  /** 错误信息 */
  error: string
  /** 是否显示弹窗（手动触发或发现更新时为 true） */
  showDialog: boolean
}

/** 全局更新状态（响应式，组件可直接 watch） */
export const updateState = reactive<UpdateState>({
  status: 'idle',
  version: '',
  notes: '',
  forceUpdate: false,
  downloaded: 0,
  total: 0,
  error: '',
  showDialog: false,
})

/** 当前待安装的 Update 对象（checkForUpdate 后缓存，downloadAndInstall 使用） */
let pendingUpdate: Update | null = null

/** 防止并发检查 */
let checking = false

/** 防止并发下载 */
let installing = false

/**
 * 检查更新
 *
 * @param opts.silent 静默模式（启动时/定时触发）：
 *   - true：仅在发现可用更新时弹窗，无更新/检查失败均不打扰用户
 *   - false：手动触发，无论结果都给用户反馈（toast 或弹窗）
 */
export async function checkForUpdate(opts: { silent?: boolean } = {}): Promise<void> {
  const { silent = false } = opts
  if (checking) return
  checking = true

  if (!silent) {
    updateState.status = 'checking'
    updateState.showDialog = true
    updateState.error = ''
  }

  try {
    const update = await check()
    if (update?.available) {
      // 读取 rawJson 中的 force_update 扩展字段（api-server 下发，Tauri plugin 会忽略未知字段）
      // Update.rawJson 是 manifest 接口返回的原始 JSON，包含 MoLaunch 扩展的 force_update 字段
      const forceUpdate = update.rawJson?.force_update === true

      updateState.status = 'available'
      updateState.version = update.version
      updateState.notes = update.body ?? ''
      updateState.forceUpdate = forceUpdate
      updateState.downloaded = 0
      updateState.total = 0
      updateState.error = ''
      updateState.showDialog = true
      pendingUpdate = update
    } else {
      updateState.status = 'no-update'
      updateState.showDialog = false
      if (!silent) {
        toastInfo('当前已是最新版本')
      }
    }
  } catch (e) {
    updateState.status = 'error'
    updateState.error = String(e)
    updateState.showDialog = false
    if (!silent) {
      toastError(`检查更新失败：${String(e)}`)
    }
    console.error('[Updater] checkForUpdate error:', e)
  } finally {
    checking = false
  }
}

/**
 * 下载并安装更新
 *
 * 调用前提：updateState.status === 'available'（即 checkForUpdate 已发现更新）。
 * 流程：下载（带进度回调）→ 安装（plugin 内部替换文件）→ relaunch 重启主进程。
 */
export async function downloadAndInstall(): Promise<void> {
  if (installing) return
  if (!pendingUpdate) {
    toastError('无可用的更新，请先检查更新')
    return
  }
  installing = true

  updateState.status = 'downloading'
  updateState.downloaded = 0
  updateState.total = 0
  updateState.error = ''

  try {
    await pendingUpdate.downloadAndInstall((event: DownloadEvent) => {
      if (event.event === 'Started' && event.data.contentLength) {
        updateState.total = event.data.contentLength
      } else if (event.event === 'Progress' && event.data.chunkLength) {
        updateState.downloaded += event.data.chunkLength
      }
    })

    updateState.status = 'installing'
    // 文件替换已完成，重启主进程加载新版
    await relaunch()
    // relaunch 后进程退出，以下代码不会执行
    updateState.status = 'done'
  } catch (e) {
    updateState.status = 'error'
    updateState.error = String(e)
    console.error('[Updater] downloadAndInstall error:', e)
    toastError(`更新失败：${String(e)}`)
  } finally {
    installing = false
  }
}

/**
 * 关闭更新弹窗（用户点击"稍后"时调用）
 *
 * 强制更新时不允许关闭，调用方需自行判断（UpdateDialog 内也会二次校验）。
 */
export function closeDialog(): void {
  if (updateState.forceUpdate) return
  if (updateState.status === 'downloading' || updateState.status === 'installing') return
  updateState.showDialog = false
  updateState.status = 'idle'
}

/**
 * 初始化自动检查
 *
 * 在 App.vue onMounted 中调用一次：
 * - 启动后 5s 静默检查一次（避免启动时网络请求与登录恢复抢资源）
 * - 之后每 6 小时检查一次（避免长开用户错过更新）
 *
 * 开发模式（vite dev）下跳过自动检查，避免 dev 版本号低于发布版本时反复触发更新。
 */
export function initAutoCheck(): void {
  if (import.meta.env.DEV) {
    console.info('[Updater] Dev mode, skip auto check')
    return
  }

  setTimeout(() => {
    checkForUpdate({ silent: true }).catch((e) => {
      console.error('[Updater] startup auto check failed:', e)
    })
  }, 5000)

  setInterval(() => {
    checkForUpdate({ silent: true }).catch((e) => {
      console.error('[Updater] scheduled auto check failed:', e)
    })
  }, 6 * 60 * 60 * 1000)
}
