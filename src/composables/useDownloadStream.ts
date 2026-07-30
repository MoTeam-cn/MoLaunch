/**
 * 下载进度 WebSocket 流（替代 useDownloadPolling 的轮询方案）
 *
 * 工作原理：
 * - 监听 `versionStore.downloading` 状态变化
 * - true 时通过 `getWsPort` IPC 获取 WS 端口 + 鉴权 token，建立 WebSocket 连接
 * - 连接建立后客户端发送 `{"type":"auth","token":"<token>"}` 鉴权
 * - 后端校验通过后返回 `{"type":"auth_ok"}` 并开始推送 snapshot
 * - 后端在 progress_callback / stage_callback / cancel / pause / resume 时
 *   通过 broadcast channel 推送 snapshot，WS 服务器以 200ms 节流转发给前端
 * - `onmessage` 解析 JSON → 更新 versionStore（与原轮询逻辑一致）
 * - false 时关闭 WS 连接
 *
 * 初始状态恢复：
 * - 页面刷新后进入 Downloads.vue 时，先调用 `getDownloadProgress()` 一次性获取当前状态
 * - `initDownloadStream` 会自动建立 WS 连接接收后续更新
 *
 * 断线重连：
 * - `onclose` 时如果 `versionStore.downloading` 仍为 true，3 秒后自动重连
 *
 * 安全：
 * - WS 端口绑定 127.0.0.1，仅本机可访问
 * - token 通过 IPC（Tauri 内部通道）获取，不经过网络
 * - 后端 3 秒内未收到正确 token 则关闭连接，防止本机其他进程窃听
 */

import { watch } from 'vue'
import { useVersionStore } from '@/stores/version'
import { getWsPort } from '@/utils/tauri'
import { toastSuccess } from '@/utils/toast'
import { isDownloading, getDownloadProgress } from '@/utils/api/system'
import type { DownloadStage, RawDownloadProgress, RawDownloadStage } from '@/types/download'
import { safeCall } from '@/utils/async'

let ws: WebSocket | null = null
/** 重连定时器（onclose 后如果仍在下载，3 秒后重连） */
let reconnectTimer: ReturnType<typeof setTimeout> | null = null
/** 是否已主动关闭（避免主动关闭后触发重连） */
let manualClose = false
/** 防止 initDownloadStream 被多次调用注册多个 watch（App.vue + Downloads.vue 都会调用） */
let watchRegistered = false
/** 是否已完成鉴权（鉴权前不处理进度消息，鉴权后处理所有后续消息） */
let authenticated = false

/**
 * 处理 WS 收到的进度 snapshot
 *
 * 逻辑与原 useDownloadPolling 的轮询回调一致：
 * 1. 错误码检查 → stopStream（交给调用方 catch 处理 UI）
 * 2. stages 映射 + 加权百分比计算
 * 3. 暂停状态检测
 * 4. 完成检查 → finishDownload + toastSuccess
 */
function handleProgress(progress: RawDownloadProgress) {
  if (!progress || !progress.stages || progress.stages.length === 0) return

  const versionStore = useVersionStore()

  // 错误码检查：仅停止 WS，不 finishDownload / toastError
  // 失败的 UI 提示和 finishDownload 由调用方的 catch 统一处理
  if (progress.error_code && progress.error_code !== 0) {
    if (import.meta.env.DEV) {
      console.debug('[WS] Download failed with error_code=', progress.error_code, '(交给调用方 catch 处理)')
    }
    closeStream()
    return
  }

  const stages: DownloadStage[] = progress.stages.map((s: RawDownloadStage) => ({
    name: s.name,
    progress: s.progress,
    weight: s.weight,
    status: s.status,
    bytes_downloaded: s.bytes_downloaded,
    bytes_total: s.bytes_total,
    files_downloaded: s.files_downloaded || 0,
    files_total: s.files_total || 0,
    group: s.group ?? null,
  }))

  // 检测暂停状态：任意 stage 携带 is_paused=true 即表示全局暂停
  const isPaused = progress.stages.some((s: RawDownloadStage) => s.is_paused === true)

  let weightedProgress = 0
  let totalWeight = 0
  for (const stage of stages) {
    totalWeight += stage.weight
    weightedProgress += stage.progress * stage.weight
  }
  const percentage = totalWeight > 0
    ? Math.min(100, parseFloat(((weightedProgress / totalWeight) * 100).toFixed(1)))
    : 0

  versionStore.updateProgress({
    stages,
    current_stage_index: progress.current_stage_index ?? 0,
    global_speed: progress.global_speed ?? 0,
    global_bytes_downloaded: progress.global_bytes_downloaded ?? 0,
    global_bytes_total: progress.global_bytes_total ?? 0,
    percentage,
    isPaused,
  })

  if (progress.is_complete) {
    if (import.meta.env.DEV) {
      console.debug('[WS] Download complete, closing stream')
    }
    closeStream()
    const completedName = versionStore.downloadingVersion || '下载任务'
    versionStore.finishDownload()
    toastSuccess(`${completedName} 下载完成`)
  }
}

/**
 * 建立 WebSocket 连接
 *
 * 通过 `getWsPort` IPC 获取端口 + token，然后连接 `ws://127.0.0.1:{port}`。
 * 连接建立后立即发送鉴权消息，后端校验通过后返回 auth_ok。
 * 如果端口为 0（WS 服务器尚未启动），500ms 后重试。
 */
async function openStream() {
  if (ws) return // 已连接

  manualClose = false
  authenticated = false

  const wsInfo = await safeCall(() => getWsPort(), 'get ws port')
  if (!wsInfo || !wsInfo.port || wsInfo.port === 0 || !wsInfo.token) {
    // WS 服务器尚未启动或 token 缺失，500ms 后重试
    if (reconnectTimer) clearTimeout(reconnectTimer)
    reconnectTimer = setTimeout(() => {
      const versionStore = useVersionStore()
      if (versionStore.downloading) {
        void openStream()
      }
    }, 500)
    return
  }

  const url = `ws://127.0.0.1:${wsInfo.port}`
  if (import.meta.env.DEV) {
    console.debug('[WS] Connecting to', url)
  }

  ws = new WebSocket(url)

  ws.onopen = () => {
    if (import.meta.env.DEV) {
      console.debug('[WS] Connected, sending auth token')
    }
    // 连接建立后立即发送鉴权消息
    // 后端 3 秒内未收到正确 token 则关闭连接
    ws?.send(JSON.stringify({ type: 'auth', token: wsInfo.token }))
  }

  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data)

      // 鉴权阶段：等待 auth_ok 消息
      if (!authenticated) {
        if (data.type === 'auth_ok') {
          authenticated = true
          if (import.meta.env.DEV) {
            console.debug('[WS] Authenticated, ready to receive progress')
          }
        } else {
          // 鉴权前的消息可能是 Close 帧的 JSON 或非法消息，忽略
          if (import.meta.env.DEV) {
            console.debug('[WS] Pre-auth message ignored:', data)
          }
        }
        return
      }

      // 鉴权后：处理进度消息
      handleProgress(data as RawDownloadProgress)
    } catch (e) {
      console.error('[WS] Failed to parse message:', e)
    }
  }

  ws.onerror = (e) => {
    console.error('[WS] Error:', e)
  }

  ws.onclose = () => {
    ws = null
    authenticated = false
    if (import.meta.env.DEV) {
      console.debug('[WS] Closed')
    }
    // 非主动关闭 + 仍在下载 → 3 秒后重连
    if (!manualClose) {
      const versionStore = useVersionStore()
      if (versionStore.downloading) {
        if (reconnectTimer) clearTimeout(reconnectTimer)
        reconnectTimer = setTimeout(() => {
          if (versionStore.downloading) {
            void openStream()
          }
        }, 3000)
      }
    }
  }
}

/** 关闭 WS 连接（主动关闭，不触发重连） */
function closeStream() {
  manualClose = true
  if (reconnectTimer) {
    clearTimeout(reconnectTimer)
    reconnectTimer = null
  }
  if (ws) {
    ws.close()
    ws = null
  }
  authenticated = false
}

/**
 * 初始化下载进度 WS 流
 *
 * 在 `App.vue` 的 `onMounted` 中调用一次即可。监听 `versionStore.downloading`：
 * - true → 建立 WS 连接
 * - false → 关闭 WS 连接
 *
 * 启动时主动检查后端下载状态：
 * - 应用刷新/重启后 `versionStore.downloading` 为 false，但后端可能仍在下载
 * - 主动调用 `isDownloading` 检查，若仍在下载则 `startDownload` 恢复状态，
 *   随后 watch 检测到 downloading=true 自动建立 WS 连接接收后续进度
 *
 * 内部有 guard 防止多次调用注册多个 watch（Downloads.vue 也会调用此函数作为保险）。
 */
export function initDownloadStream() {
  if (watchRegistered) return
  watchRegistered = true

  const versionStore = useVersionStore()

  // 启动时主动检查后端是否有进行中的下载任务
  void (async () => {
    const active = await safeCall(() => isDownloading(), 'init check downloading')
    if (active) {
      const raw = await safeCall(() => getDownloadProgress(), 'init get progress')
      versionStore.startDownload(raw?.version_name || '')
    }
  })()

  watch(
    () => versionStore.downloading,
    (downloading) => {
      if (downloading) {
        void openStream()
      } else {
        closeStream()
      }
    },
  )
}
