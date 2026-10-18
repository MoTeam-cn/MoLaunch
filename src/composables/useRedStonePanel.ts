/**
 * 红石联机 - 创建房间面板逻辑 composable
 *
 * 中转服务器列表加载、隧道状态机（idle/creating/open/closed/error）、
 * 2s 轮询 + 15s 超时；端口选择复用 port-picker 子窗口；
 * 监听后端 `scaffolding-mc-port-change` / `mc-port-detected` 自动回填端口并 toast。
 */
import { ref, computed, onMounted, onUnmounted, onActivated, onDeactivated } from 'vue'
import {
  redstoneGetServers,
  redstonePlatformSupport,
  redstoneStart,
  redstoneStatus,
  redstoneStop,
} from '@/utils/api/redstone'
import { getRunningMcPort } from '@/utils/api/online-manager'
import { addressLatencyTest } from '@/utils/api/tools'
import { copyToClipboard } from '@/utils/clipboard'
import { toastError, toastInfo, toastSuccess } from '@/utils/toast'
import { openPickerWindow } from '@/utils/picker-window'
import { isMacOS } from '@/utils/platform'
import { useTauriEvent } from '@/utils/tauriEvent'
import type { RedStoneServer, RedStoneStatusResult } from '@/types/redstone'

const POLL_INTERVAL = 2000
const POLL_TIMEOUT = 15000
const CLOSED_MESSAGE = '房间已关闭（长时间无人或服务器维护）'
/** 平台不支持时的兜底提示（后端探测到具体原因时会覆盖它） */
export const MACOS_UNSUPPORTED_MESSAGE = '官方尚未提供 macOS 内核，红石联机在当前平台暂不可用'
export type RedStoneCreatePhase = 'idle' | 'creating' | 'open' | 'closed' | 'error'

/**
 * 延迟 → Tag 预设色映射（沿用联机测速默认阈值：
 * <50 绿 / <150 金 / <300 橙 / 否则红；未测灰）
 */
export function latencyTagColor(ms: number | null): string {
  if (ms == null) return 'gray'
  if (ms < 50) return 'green'
  if (ms < 150) return 'gold'
  if (ms < 300) return 'orange'
  return 'red'
}

/**
 * 平台支持状态（创建面板与内核面板共用）
 *
 * 初值对 macOS 取「不支持」的乐观判定，避免探测返回前闪现正常 UI；
 * 真正的判定来自后端 redstone_platform_support —— 它会**实际探测官方下载端点**
 * （而非写死平台），因此官方恢复 macOS 构建后无需更新启动器即可自动恢复。
 */
export function useRedStonePlatformSupport() {
  const platformSupported = ref(!isMacOS())
  /** 后端给出的不支持原因（探测失败或支持时为空串） */
  const platformReason = ref('')
  const platformChecking = ref(false)
  const platformUnsupported = computed(() => !platformSupported.value)
  /** 面板展示的不支持原因（后端原因优先，回退默认文案） */
  const platformUnsupportedMessage = computed(
    () => platformReason.value || MACOS_UNSUPPORTED_MESSAGE,
  )

  /** 探测平台支持；force=true 绕过后端的进程内缓存（「重新检测」用） */
  async function checkPlatformSupport(force = false): Promise<boolean> {
    if (platformChecking.value) return !platformUnsupported.value
    platformChecking.value = true
    try {
      const res = await redstonePlatformSupport(force)
      platformSupported.value = res.supported
      platformReason.value = res.reason ?? ''
    } catch {
      // 探测本身失败不阻断：按支持处理，让创建时暴露真实错误
      platformSupported.value = true
      platformReason.value = ''
    } finally {
      platformChecking.value = false
    }
    return !platformUnsupported.value
  }

  return {
    platformUnsupported,
    platformUnsupportedMessage,
    platformChecking,
    checkPlatformSupport,
  }
}

/** 红石联机创建面板逻辑（含端口选择与 MC 端口事件自动回填） */
export function useRedStonePanel() {
  const {
    platformUnsupported,
    platformUnsupportedMessage,
    platformChecking,
    checkPlatformSupport,
  } = useRedStonePlatformSupport()
  /** 事件监听是否已启动（探测翻转为支持后补启，避免重复注册） */
  let listenersStarted = false

  /** 启动红石相关加载与事件监听（平台支持时才调用） */
  function startRedstone() {
    void restoreStatus()
    void loadServers()
    void autoFillPort()
    if (!listenersStarted) {
      listenersStarted = true
      void portChangeListener.start()
      void mcPortDetectedListener.start()
    }
  }

  /** 「重新检测」：强制重新探测官方端点，恢复可用时直接进入正常流程 */
  async function recheckPlatformSupport() {
    const supported = await checkPlatformSupport(true)
    if (!supported) {
      toastError(platformUnsupportedMessage.value)
      return
    }
    toastSuccess('已检测到官方内核，红石联机可用')
    startRedstone()
  }
  const servers = ref<RedStoneServer[]>([])
  const serverLoading = ref(false)
  const serverError = ref('')
  const useManualServer = ref(false)
  const server = ref('')
  const mcPort = ref('')
  const portSelecting = ref(false)
  /** 节点延迟缓存（host → ms，测试失败为 null） */
  const latencies = ref<Record<string, number | null>>({})
  const latencyTesting = ref(false)
  const phase = ref<RedStoneCreatePhase>('idle')
  const status = ref<RedStoneStatusResult | null>(null)
  const errorMessage = ref('')
  const creating = ref(false)
  const stopping = ref(false)
  const restarting = ref(false)
  let pollTimer: ReturnType<typeof setInterval> | null = null
  let pollStart = 0
  let polling = false
  let mountedOnce = false

  const serverOptions = computed(() =>
    servers.value.map((s) => ({
      label: `${s.host}（${s.region}）`,
      value: s.host,
      latencyMs: latencies.value[s.host] ?? null,
    })),
  )
  const address = computed(() => {
    const host = status.value?.server
    const port = status.value?.port
    return host && port != null ? `${host}:${port}` : ''
  })
  function toMessage(e: unknown): string {
    return e instanceof Error ? e.message : String(e)
  }
  async function loadServers() {
    if (platformUnsupported.value) return
    serverLoading.value = true
    serverError.value = ''
    try {
      const res = await redstoneGetServers()
      servers.value = res.servers
      if (res.servers.length > 0) {
        useManualServer.value = false
        if (!res.servers.some((s) => s.host === server.value)) server.value = res.servers[0].host
        toastSuccess(`服务器列表已刷新（${res.servers.length} 个节点）`)
      } else {
        useManualServer.value = true
        serverError.value = '暂无可用的中转服务器，可手动填写服务器地址'
      }
    } catch (e) {
      useManualServer.value = true
      serverError.value = `服务器列表拉取失败：${toMessage(e)}，可手动填写地址`
      toastError(`服务器列表拉取失败：${toMessage(e)}`)
    } finally {
      serverLoading.value = false
    }
    void testServersLatency()
  }

  /** 测试各节点延迟（ping），自动首选延迟最低的可达节点 */
  async function testServersLatency(autoSelect = true) {
    if (servers.value.length === 0 || latencyTesting.value) return
    latencyTesting.value = true
    try {
      const res = await addressLatencyTest(
        servers.value.map((s) => ({
          name: s.region,
          host: s.host,
          port: 443,
          protocol: 'ping' as const,
        })),
      )
      const map: Record<string, number | null> = {}
      for (const item of res.results) map[item.host] = item.reachable ? item.latency_ms : null
      latencies.value = map
      const reachable = res.results.filter((r) => r.reachable)
      let bestHost: string | null = null
      if (autoSelect) {
        const best = [...reachable].sort((a, b) => a.latency_ms - b.latency_ms)[0]
        if (best && servers.value.some((s) => s.host === best.host)) {
          server.value = best.host
          bestHost = best.host
        }
      }
      const bestMsg = bestHost ? `，已自动首选 ${bestHost}` : ''
      toastSuccess(`延迟测试完成：${reachable.length}/${res.results.length} 个节点可达${bestMsg}`)
    } catch (e) {
      toastError(`节点延迟测试失败：${toMessage(e)}`)
    } finally {
      latencyTesting.value = false
    }
  }
  /**
   * 复用 port-picker 子窗口选择本机端口（与 FRP 创建隧道一致）；
   * 红石隧道仅支持穿透 Java 版联机端口，故按进程名过滤只显示 Java 进程的开放端口
   */
  async function handleSelectPort() {
    if (portSelecting.value) return
    portSelecting.value = true
    try {
      const value = await openPickerWindow({
        title: '选择 MC 端口（Java）',
        template: 'port-picker',
        data: { process_filter: 'java' },
        width: 400,
        height: 500,
      })
      if (value) {
        mcPort.value = String(value)
        toastSuccess(`已选择端口 ${value}`)
      }
    } catch (e) {
      if (!(e instanceof Error && e.message.includes('取消'))) toastError('选择端口失败：' + e)
    } finally {
      portSelecting.value = false
    }
  }
  async function autoFillPort() {
    try {
      const res = await getRunningMcPort()
      if (res.success && res.ports.length > 0 && !mcPort.value) mcPort.value = String(res.ports[0])
    } catch (e) {
      console.warn('[RedStone] 自动探测 MC 端口失败', e)
    }
  }
  function validateInputs(): string | null {
    if (!server.value.trim()) return '请先选择或填写中转服务器'
    const portStr = mcPort.value.trim()
    if (!/^\d{1,5}$/.test(portStr)) return 'MC 端口需为 1-65535 的数字'
    const port = Number(portStr)
    if (port < 1 || port > 65535) return 'MC 端口需在 1-65535 范围内'
    return null
  }
  function clearPollTimer() {
    if (pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
  }

  /** 单次状态轮询；返回 true 表示结束轮询 */
  async function pollStatus(): Promise<boolean> {
    if (polling) return false
    polling = true
    try {
      const res = await redstoneStatus()
      status.value = res
      if (res.status === 'open') {
        phase.value = 'open'
        errorMessage.value = ''
        toastSuccess('隧道已建立，可复制地址分享给好友')
        return true
      }
      if (res.status === 'closed') {
        phase.value = 'closed'
        errorMessage.value = CLOSED_MESSAGE
        toastError(CLOSED_MESSAGE)
        return true
      }
      if (!res.running) {
        phase.value = 'error'
        errorMessage.value = '隧道创建失败：服务器不可达或暂无可分配端口，请稍后重试或更换服务器'
        toastError(errorMessage.value)
        return true
      }
      if (Date.now() - pollStart > POLL_TIMEOUT) {
        phase.value = 'error'
        errorMessage.value = '隧道建立超时'
        toastError('隧道建立超时，请检查中转服务器或稍后重试')
        return true
      }
      return false
    } catch (e) {
      phase.value = 'error'
      errorMessage.value = `查询隧道状态失败：${toMessage(e)}`
      toastError(errorMessage.value)
      return true
    } finally {
      polling = false
    }
  }
  /** 以 2s 间隔轮询直至 open / 错误 / 超时 */
  function startPolling() {
    clearPollTimer()
    pollTimer = setInterval(async () => {
      if (await pollStatus()) clearPollTimer()
    }, POLL_INTERVAL)
  }

  async function launchTunnel(serverHost: string, mcPortValue: number) {
    creating.value = true
    clearPollTimer()
    try {
      await redstoneStart({ server: serverHost, mc_port: mcPortValue })
      phase.value = 'creating'
      errorMessage.value = ''
      pollStart = Date.now()
      startPolling()
    } catch (e) {
      phase.value = 'error'
      errorMessage.value = `隧道创建失败：${toMessage(e)}`
      toastError(errorMessage.value)
    } finally {
      creating.value = false
    }
  }

  function handleCreate() {
    if (platformUnsupported.value) {
      toastError(MACOS_UNSUPPORTED_MESSAGE)
      return
    }
    if (creating.value) return
    const error = validateInputs()
    if (error) {
      toastError(error)
      return
    }
    toastInfo(`正在连接 ${server.value.trim()} 创建隧道…`)
    void launchTunnel(server.value.trim(), Number(mcPort.value))
  }

  /** 重启 = 先 stop 再 start（沿用当前服务器与端口） */
  async function handleRestart() {
    if (platformUnsupported.value) {
      toastError(MACOS_UNSUPPORTED_MESSAGE)
      return
    }
    if (creating.value || restarting.value) return
    const error = validateInputs()
    if (error) {
      toastError(error)
      return
    }
    restarting.value = true
    toastInfo(`正在重启隧道（${server.value.trim()}）…`)
    try {
      await redstoneStop()
    } catch (e) {
      toastError(`停止旧隧道失败：${toMessage(e)}`)
    }
    await launchTunnel(server.value.trim(), Number(mcPort.value))
    restarting.value = false
  }

  async function handleStop() {
    if (stopping.value) return
    stopping.value = true
    try {
      await redstoneStop()
      clearPollTimer()
      phase.value = 'idle'
      status.value = null
      errorMessage.value = ''
      toastSuccess('隧道已停止')
    } catch (e) {
      toastError(`停止隧道失败：${toMessage(e)}`)
    } finally {
      stopping.value = false
    }
  }

  async function copyAddress() {
    if (address.value) await copyToClipboard(address.value, { toast: true })
  }

  async function restoreStatus() {
    if (platformUnsupported.value) return
    try {
      const res = await redstoneStatus()
      status.value = res
      if (!res.running) {
        if (res.status === 'closed') { phase.value = 'closed'; errorMessage.value = CLOSED_MESSAGE }
        else { phase.value = 'idle'; errorMessage.value = '' }
        return
      }
      if (res.status === 'open') { phase.value = 'open'; errorMessage.value = ''; return }
      phase.value = 'creating' // 进程存活但隧道未就绪：恢复创建中轮询
      errorMessage.value = ''
      pollStart = Date.now()
      startPolling()
    } catch {
      phase.value = 'idle'
    }
  }

  /** 后端 MC 端口变更事件（后台监视发现新端口）自动回填 */
  const portChangeListener = useTauriEvent<{ mcPort: number }>('scaffolding-mc-port-change', (p) => {
    if (p.mcPort && Number(p.mcPort) !== Number(mcPort.value)) {
      mcPort.value = String(p.mcPort)
      toastSuccess(`MC 端口已自动更新为 ${p.mcPort}`)
    }
  })
  /** watcher 捕获的 MC 局域网端口事件（payload 为裸端口号） */
  const mcPortDetectedListener = useTauriEvent<number>('mc-port-detected', (port) => {
    if (port && Number(port) !== Number(mcPort.value)) {
      mcPort.value = String(port)
      toastSuccess(`MC 端口已自动更新为 ${port}`)
    }
  })

  onMounted(() => {
    // keep-alive 首次激活紧跟 mounted：必须**同步**占位，否则 onActivated 会先于
    // 平台探测返回而执行，去重守卫失效（多打一次 status IPC，且切回时跳过状态恢复）
    mountedOnce = true
    // 再探测平台支持（会实际打官方端点）；不支持则不发起任何红石请求，由面板显示错误卡
    void (async () => {
      if (await checkPlatformSupport()) startRedstone()
    })()
  })
  onActivated(() => { // keep-alive 初次激活紧跟 mounted，去重避免重复拉取
    if (mountedOnce) {
      mountedOnce = false
      return
    }
    void restoreStatus()
  })
  onDeactivated(() => clearPollTimer())
  onUnmounted(() => clearPollTimer())

  return {
    platformUnsupported,
    platformUnsupportedMessage,
    platformChecking,
    recheckPlatformSupport,
    servers,
    serverLoading,
    serverError,
    useManualServer,
    server,
    mcPort,
    portSelecting,
    latencies,
    latencyTesting,
    phase,
    errorMessage,
    creating,
    stopping,
    restarting,
    serverOptions,
    address,
    handleSelectPort,
    loadServers,
    testServersLatency,
    handleCreate,
    handleRestart,
    handleStop,
    copyAddress,
  }
}