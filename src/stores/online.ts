/**
 * 联机功能状态管理
 *
 * 管理设备认证状态 + api-server 地址 + 房间状态（阶段二）。
 *
 * 设计：
 * - `deviceStatus`：本地缓存的上次查询到的设备状态（null 表示未查询）
 * - `apiServerUrl`：从后端配置同步的 api-server 地址（与 settings store 解耦，避免循环依赖）
 * - `refreshStatus()`：拉取最新设备状态（不发网络请求，仅读本地凭证 + 后端配置）
 * - 所有写操作（register/login/logout/clear）成功后自动 refreshStatus()
 *
 * 阶段二扩展房间状态：
 * - `roomState`：当前房间状态（角色 + 房间码 + 参与者列表）
 * - `roomAction(action, payload)`：统一调度信令 action 的薄封装
 *   （WebRTC 协商、keepalive 定时器等业务逻辑放在 RoomManager.vue 中，
 *    store 只持久化房间元信息，避免与组件生命周期耦合）
 */

import { defineStore } from 'pinia'
import { ref } from 'vue'
import type {
  CreateRoomResponse,
  DeviceStatus,
  JoinRoomResponse,
  ListParticipantsResponse,
  NatDetectionResult,
  ParticipantInfo,
  RoomInfoResponse,
  StunServersResponse,
} from '@/types/online'
import {
  getAuthStatus,
  registerDevice,
  loginDevice,
  logoutDevice,
  clearDevice,
  getStunServers,
  createRoom,
  getRoomInfo,
  closeRoom,
  joinRoom,
  leaveRoom,
  keepaliveRoom,
  listParticipants,
} from '@/utils/api/online-manager'
import { applyConfig, getConfigMap } from '@/utils/api/config'
import { toastSuccess, toastError } from '@/utils/toast'
import { safeCall } from '@/utils/async'

/** 房间角色 */
export type RoomRole = 'host' | 'guest' | null

/**
 * 创建房间步骤（UI 进度反馈用）
 *
 * mesh 拓扑下房主创建房间不再生成本地 Offer（改为 per-participant 按需生成）：
 * - `stun`：获取 STUN 服务器列表
 * - `create`：调用后端创建房间
 * - `null`：未在创建中 / 已完成 / 失败
 */
export type RoomCreateStep = 'stun' | 'create' | null

/** 房间状态（阶段二） */
export interface RoomState {
  /** 角色：房主 / 加入方 / null（未在房间） */
  role: RoomRole
  /** 房间码 */
  roomCode: string
  /** 房主虚拟 IP */
  hostVirtualIp: string
  /** 自己的虚拟 IP */
  selfVirtualIp: string
  /** 子网 CIDR */
  subnet: string
  /** 最大人数 */
  maxPlayers: number
  /** 房间过期时间（Unix 秒） */
  expiresAt: number
  /** STUN 服务器列表（房间内一致） */
  stunServers: string[]
  /** 房主 MC 版本（加入方需匹配） */
  hostMcVersion: string
  /** 房主 MC 端口 */
  hostMcPort: number
  /** 当前参与者列表（房主维护） */
  participants: ParticipantInfo[]
  /** 加入方的 participant_id */
  participantId: string | null
}

/** 创建空房间状态 */
function emptyRoom(): RoomState {
  return {
    role: null,
    roomCode: '',
    hostVirtualIp: '',
    selfVirtualIp: '',
    subnet: '',
    maxPlayers: 0,
    expiresAt: 0,
    stunServers: [],
    hostMcVersion: '',
    hostMcPort: 0,
    participants: [],
    participantId: null,
  }
}

export const useOnlineStore = defineStore('online', () => {
  // ===== 设备认证 =====
  /** 设备认证状态（null 表示未查询） */
  const deviceStatus = ref<DeviceStatus | null>(null)
  /** 是否正在执行写操作（注册/登录/登出/清除） */
  const loading = ref(false)
  /** 是否正在拉取状态 */
  const refreshing = ref(false)
  /** 当前 api-server 地址（从后端配置同步） */
  const apiServerUrl = ref('')

  // ===== 房间状态 =====
  /** 当前房间状态（null 表示未在房间） */
  const roomState = ref<RoomState>(emptyRoom())
  /** 是否正在执行房间操作 */
  const roomLoading = ref(false)
  /** 创建房间当前步骤（UI 进度反馈，null 表示未在创建中） */
  const roomCreateStep = ref<RoomCreateStep>(null)
  /** STUN 服务器列表缓存（房间创建/加入前预取） */
  const stunServers = ref<string[]>([])

  // ===== NAT 检测 =====
  /** NAT 检测结果（null 表示未检测） */
  const natResult = ref<NatDetectionResult | null>(null)

  /**
   * 拉取最新设备状态（不发起网络请求，仅读本地凭证 + 后端配置）
   *
   * 同时同步 apiServerUrl（用于 SettingsOnline 页显示）。
   */
  async function refreshStatus(): Promise<void> {
    refreshing.value = true
    await safeCall(async () => {
      const status = await getAuthStatus()
      deviceStatus.value = status
      apiServerUrl.value = status.api_server_url
    }, '[Online] refresh status')
    refreshing.value = false
  }

  /**
   * 更新 api-server 地址（写入后端 INI，不立即触发设备状态刷新）
   *
   * 后端 `apply_online` 会忽略空字符串，避免误清空。
   * @returns 是否保存成功
   */
  async function setApiServerUrl(url: string): Promise<boolean> {
    const trimmed = url.trim()
    if (!trimmed) {
      toastError('api-server 地址不能为空')
      return false
    }
    const ok = await safeCall(
      async () => {
        await applyConfig({ onlineApiServerUrl: trimmed })
        apiServerUrl.value = trimmed
      },
      '[Online] set api server url',
    )
    if (ok !== undefined) {
      toastSuccess('api-server 地址已保存')
      return true
    }
    return false
  }

  /** 从后端配置同步 apiServerUrl（不拉取设备状态，用于 SettingsOnline 初始化） */
  async function syncApiServerUrlFromConfig(): Promise<void> {
    await safeCall(async () => {
      const cfg = await getConfigMap()
      apiServerUrl.value = cfg.onlineApiServerUrl
    }, '[Online] sync api server url from config')
  }

  /** 注册新设备 */
  async function register(): Promise<boolean> {
    loading.value = true
    const ok = await safeCall(
      async () => {
        const status = await registerDevice()
        deviceStatus.value = status
      },
      '[Online] register device',
    )
    loading.value = false
    if (ok !== undefined) {
      toastSuccess('设备注册成功')
      return true
    }
    return false
  }

  /** 登录设备（刷新 JWT） */
  async function login(): Promise<boolean> {
    loading.value = true
    const ok = await safeCall(
      async () => {
        const status = await loginDevice()
        deviceStatus.value = status
      },
      '[Online] login device',
    )
    loading.value = false
    if (ok !== undefined) {
      toastSuccess('设备登录成功')
      return true
    }
    return false
  }

  /** 登出设备（撤销 JWT，保留密钥） */
  async function logout(): Promise<boolean> {
    loading.value = true
    const ok = await safeCall(
      async () => {
        await logoutDevice()
        await refreshStatus()
      },
      '[Online] logout device',
    )
    loading.value = false
    if (ok !== undefined) {
      toastSuccess('设备已登出')
      return true
    }
    return false
  }

  /** 清除设备凭证（注销设备，删除本地密钥） */
  async function clear(): Promise<boolean> {
    loading.value = true
    const ok = await safeCall(
      async () => {
        await clearDevice()
        await refreshStatus()
      },
      '[Online] clear device',
    )
    loading.value = false
    if (ok !== undefined) {
      toastSuccess('设备凭证已清除')
      return true
    }
    return false
  }

  // ============================================================
  // 房间相关操作（阶段二）
  // ============================================================

  /** 拉取 STUN 服务器列表并缓存 */
  async function fetchStunServers(): Promise<string[]> {
    const result = await getStunServers()
    if (result.code !== 1 || !result.data) {
      throw new Error(result.msg || '获取 STUN 服务器列表失败')
    }
    const list = (result.data as StunServersResponse).servers ?? []
    stunServers.value = list
    return list
  }

  /**
   * 房主创建房间
   *
   * @param sdpOffer 房主本地 SDP Offer
   * @param iceCandidates 房主收集的 ICE candidate 数组
   * @param maxPlayers 最大人数（含房主）
   * @param password 房间密码（空字符串表示无密码）
   * @param hostMcVersion 房主 MC 版本
   * @param hostMcPort 房主 MC 端口
   * @param preloadedStun 可选，调用方已预取的 STUN 列表（避免重复获取）
   * @returns 创建房间响应
   */
  async function hostCreateRoom(
    sdpOffer: string,
    iceCandidates: string[],
    maxPlayers: number,
    password: string,
    hostMcVersion: string,
    hostMcPort: number,
    preloadedStun?: string[],
  ): Promise<CreateRoomResponse> {
    roomLoading.value = true
    try {
      const stun = preloadedStun ?? (await fetchStunServers())
      const result = await createRoom({
        sdpOffer,
        iceCandidates,
        maxPlayers,
        password,
        stunServers: stun,
        hostMcVersion,
        hostMcPort,
      })
      if (result.code !== 1 || !result.data) {
        throw new Error(result.msg || '创建房间失败')
      }
      const data = result.data
      roomState.value = {
        role: 'host',
        roomCode: data.roomCode,
        hostVirtualIp: data.hostVirtualIp,
        selfVirtualIp: data.hostVirtualIp,
        subnet: data.subnet,
        maxPlayers,
        expiresAt: data.expiresAt,
        stunServers: stun,
        hostMcVersion,
        hostMcPort,
        participants: [],
        participantId: null,
      }
      toastSuccess(`房间已创建：${data.roomCode}`)
      return data
    } finally {
      roomLoading.value = false
      roomCreateStep.value = null
    }
  }

  /** 房主关闭房间 */
  async function hostCloseRoom(): Promise<void> {
    if (roomState.value.role !== 'host' || !roomState.value.roomCode) return
    roomLoading.value = true
    try {
      await closeRoom(roomState.value.roomCode)
      roomState.value = emptyRoom()
      toastSuccess('房间已关闭')
    } finally {
      roomLoading.value = false
    }
  }

  /**
   * 加入方加入房间
   *
   * @param roomCode 房间码
   * @param password 房间密码（空字符串表示无密码）
   * @returns 加入房间响应（含 hostSdpOffer / hostIceCandidates 用于 WebRTC 协商）
   */
  async function guestJoinRoom(roomCode: string, password: string): Promise<JoinRoomResponse> {
    roomLoading.value = true
    try {
      const result = await joinRoom(roomCode, password)
      if (result.code !== 1 || !result.data) {
        throw new Error(result.msg || '加入房间失败')
      }
      const data = result.data
      roomState.value = {
        role: 'guest',
        roomCode,
        hostVirtualIp: '',
        selfVirtualIp: data.playerVirtualIp,
        subnet: data.subnet,
        maxPlayers: 0,
        expiresAt: 0,
        stunServers: data.stunServers ?? [],
        hostMcVersion: '',
        hostMcPort: 0,
        participants: [],
        participantId: data.participantId,
      }
      // 拉取房间公开信息补全元数据
      await refreshRoomInfo()
      toastSuccess(`已加入房间：${roomCode}`)
      return data
    } finally {
      roomLoading.value = false
    }
  }

  /** 加入方退出房间 */
  async function guestLeaveRoom(): Promise<void> {
    if (roomState.value.role !== 'guest' || !roomState.value.roomCode) return
    roomLoading.value = true
    try {
      await leaveRoom(roomState.value.roomCode)
      roomState.value = emptyRoom()
      toastSuccess('已退出房间')
    } finally {
      roomLoading.value = false
    }
  }

  /** 刷新房间公开信息（房主/加入方通用） */
  async function refreshRoomInfo(): Promise<RoomInfoResponse | null> {
    if (!roomState.value.roomCode) return null
    const result = await getRoomInfo(roomState.value.roomCode)
    if (result.code !== 1 || !result.data) return null
    const info = result.data
    roomState.value.maxPlayers = info.maxPlayers
    roomState.value.expiresAt = info.expiresAt
    roomState.value.hostMcVersion = info.hostMcVersion
    roomState.value.hostMcPort = info.hostMcPort
    if (roomState.value.role === 'guest') {
      // 加入方需要 STUN 与房主一致
      roomState.value.stunServers = info.stunServers ?? roomState.value.stunServers
    }
    return info
  }

  /** 房主拉取参与者列表 */
  async function refreshParticipants(): Promise<ParticipantInfo[]> {
    if (roomState.value.role !== 'host' || !roomState.value.roomCode) return []
    const result = await listParticipants(roomState.value.roomCode)
    if (result.code !== 1 || !result.data) return []
    const list = (result.data as ListParticipantsResponse).participants ?? []
    roomState.value.participants = list
    return list
  }

  /** 房主保活 */
  async function keepalive(): Promise<{ expiresAt: number; serverTime: number } | null> {
    if (roomState.value.role !== 'host' || !roomState.value.roomCode) return null
    const result = await keepaliveRoom(roomState.value.roomCode)
    if (result.code !== 1 || !result.data) return null
    roomState.value.expiresAt = result.data.expiresAt
    return result.data
  }

  /** 重置房间状态（不调用后端，仅清空本地） */
  function resetRoomState(): void {
    roomState.value = emptyRoom()
  }

  return {
    // 设备认证状态
    deviceStatus,
    loading,
    refreshing,
    apiServerUrl,
    // 房间状态
    roomState,
    roomLoading,
    roomCreateStep,
    stunServers,
    // NAT 检测
    natResult,
    // 设备认证方法
    refreshStatus,
    syncApiServerUrlFromConfig,
    setApiServerUrl,
    register,
    login,
    logout,
    clear,
    // 房间方法
    fetchStunServers,
    hostCreateRoom,
    hostCloseRoom,
    guestJoinRoom,
    guestLeaveRoom,
    refreshRoomInfo,
    refreshParticipants,
    keepalive,
    resetRoomState,
  }
})
