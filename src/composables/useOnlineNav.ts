/**
 * Online 联机页导航分类与状态联动 composable
 *
 * 就绪判定仅依赖「已注册 + 已登录」（不判断本地 token 过期，由后端自动续期）；
 * isReady watch 从 URL ?tab= 恢复激活项，isInRoom watch 进出房间自动切换分类。
 */

import { computed, watch, type Ref, type Component } from 'vue'
import { useRoute } from 'vue-router'
import {
  DevicePhoneMobileIcon,
  ServerStackIcon,
  PlusIcon,
  ArrowRightOnRectangleIcon,
  HomeIcon,
  GlobeAltIcon,
} from '@heroicons/vue/24/outline'
import { useOnlineStore } from '@/stores/online'
import { frpCategory } from '@/composables/useFrpSidebar'

/** 侧边栏分类项（与 NavSidebar.vue / useFrpSidebar 内部定义同形） */
export interface NavCategory {
  id: string
  label: string
  icon: Component
  desc?: string
  children?: NavCategory[]
  /** 禁用态：未满足前置条件时灰色不可点击（如未在房间时的「房间详情」） */
  disabled?: boolean
  /** 封禁态：云端离线时灰色置灰但点击仍会触发，由 Online.vue 拦截弹窗告知原因 */
  sealed?: boolean
}

/** Online 页激活分类 ID 联合类型（tutorial 为侧边栏动作项，不会真正成为激活态） */
export type OnlineCategoryId =
  | 'device' | 'lobby' | 'create' | 'join'
  | 'room_details' | 'providers' | 'tunnels' | 'auth' | 'logs' | 'tutorial'

/** 设备分类（始终可用） */
const deviceCategory: NavCategory = {
  id: 'device',
  label: '设备',
  icon: DevicePhoneMobileIcon,
  desc: '注册联机设备、登录获取访问凭证、查看设备 ID 与 JWT 状态',
}

/** 房间管理分类（仅已就绪时可用），包含「创建房间」「加入房间」两个子项 */
const roomCategory: NavCategory = {
  id: 'room',
  label: '房间管理',
  icon: ServerStackIcon,
  desc: '检测 NAT 类型、创建或加入房间、管理参与者与 P2P 连接',
  children: [
    {
      id: 'create',
      label: '创建房间',
      icon: PlusIcon,
      desc: '作为房主创建新房间，等待其他玩家加入',
    },
    {
      id: 'join',
      label: '加入房间',
      icon: ArrowRightOnRectangleIcon,
      desc: '通过房间码加入已有房间',
    },
  ],
}

/** 大厅分类（仅已就绪时可用，浏览公开房间列表） */
const lobbyCategory: NavCategory = {
  id: 'lobby',
  label: '联机大厅',
  icon: GlobeAltIcon,
  desc: '浏览公开房间列表，搜索整合包房间并一键加入',
}

/** URL `?tab=` 可恢复的合法分类 ID（device/lobby/顶层 + room/FRP 子项） */
const VALID_TABS = new Set<OnlineCategoryId>([
  'device', 'lobby',
  'create', 'join', 'room_details',
  'providers', 'tunnels', 'auth', 'logs',
])

/** tab 是否为可恢复的合法分类（用于 isReady watch 从 URL 恢复激活项） */
function isValidCategory(tab: string): boolean {
  return VALID_TABS.has(tab as OnlineCategoryId)
}

/**
 * 创建 Online 导航状态
 *
 * @param activeCategory 当前激活分类（由调用方持有 ref，watch 会修改它）
 * @param route useRoute() 返回值（用于 isReady 变化时从 ?tab= 恢复激活项）
 * @param onlineStore useOnlineStore() 实例
 */
export function useOnlineNav(
  activeCategory: Ref<OnlineCategoryId>,
  route: ReturnType<typeof useRoute> = useRoute(),
  onlineStore = useOnlineStore(),
) {
  const status = computed(() => onlineStore.deviceStatus)

  /**
   * 联机功能是否就绪（显示房间管理 / 大厅 / FRP 分类）
   *
   * 仅判断「已注册 + 已登录」，**不判断 token_expired**：
   * 后端 `load_creds_with_auto_refresh` 会在业务 action 调用前自动 refresh 续期，
   * 前端 `onlineManager` 也有 1003 → refresh → login → register 降级链兜底。
   * 本地 JWT 过期不拦截页面，除非静默续期全部失败（业务请求报 1003 且重试链
   * 也失败，由各调用方 toast 提示），避免"前端判断过期就隐藏整个联机功能"。
   */
  const isReady = computed(
    () => !!status.value && status.value.registered && status.value.logged_in,
  )

  /** 是否在房间中（role=host/guest） */
  const isInRoom = computed(() => onlineStore.roomState.role !== null)

  /** 房间详情子项（始终追加到 room 管理子菜单，未在房间时 disabled 灰色不可点） */
  const roomDetailsChild = computed<NavCategory>(() => ({
    id: 'room_details',
    label: '房间详情',
    icon: HomeIcon,
    desc: '查看当前房间状态、参与者列表与连接信息',
    disabled: !isInRoom.value,
  }))

  /** 实际渲染的分类列表
   *
   * 云端离线（cloudConnected=false 且初始化已完成）时：
   * - 「房间管理」「联机大厅」封禁置灰（可点击但由 Online.vue 拦截弹窗告知原因）
   * - 已在房间中（P2P 仍工作）时保留「房间详情」，创建/加入按在房规则禁用
   * - FRP（第三方隧道）不依赖 MoLaunch 云端，仍可用
   *
   * 子项 disabled 规则：
   * - 未在房间：「创建房间」「加入房间」可用，「房间详情」灰色
   * - 在房间中：「创建房间」「加入房间」灰色（必须先退出房间），「房间详情」可用
   */
  const categories = computed<NavCategory[]>(() => {
    const offline = !onlineStore.cloudConnected && !onlineStore.initializing
    if (offline) {
      const inRoom = isInRoom.value
      const children: NavCategory[] = inRoom
        ? [...roomCategory.children!.map((c) => ({ ...c, disabled: true, sealed: false })), roomDetailsChild.value]
        : roomCategory.children!.map((c) => ({ ...c, sealed: true, disabled: false }))
      return [
        deviceCategory,
        { ...lobbyCategory, sealed: true },
        { ...roomCategory, children, sealed: !inRoom },
        frpCategory,
      ]
    }
    if (!isReady.value) return [deviceCategory]
    const inRoom = isInRoom.value
    const children: NavCategory[] = [
      { ...roomCategory.children![0], disabled: inRoom },
      { ...roomCategory.children![1], disabled: inRoom },
      roomDetailsChild.value,
    ]
    const roomWithDetails: NavCategory = { ...roomCategory, children }
    return [deviceCategory, lobbyCategory, roomWithDetails, frpCategory]
  })

  /** 状态徽章文案与颜色 */
  const badge = computed(() => {
    if (!onlineStore.cloudConnected && !onlineStore.initializing) {
      return { text: '云端离线', dotClass: 'bg-red-400', wrapClass: 'bg-red-50 text-red-600' }
    }
    if (isReady.value) return { text: '已就绪', dotClass: 'bg-green-500', wrapClass: 'bg-green-50 text-green-700' }
    const isUnregistered = !status.value || !status.value.registered
    if (isUnregistered) return { text: '未注册', dotClass: 'bg-gray-400', wrapClass: 'bg-gray-100 text-gray-600' }
    return { text: '需登录', dotClass: 'bg-yellow-500', wrapClass: 'bg-yellow-50 text-yellow-700' }
  })

  /** 当前激活分类的描述（子项优先） */
  const activeDesc = computed(() => {
    for (const cat of categories.value) {
      if (cat.id === activeCategory.value) return cat.desc ?? ''
      if (cat.children) {
        const child = cat.children.find(c => c.id === activeCategory.value)
        if (child) return child.desc ?? ''
      }
    }
    return ''
  })

  /** 当前激活分类的标签（子项优先） */
  const activeLabel = computed(() => {
    for (const cat of categories.value) {
      if (cat.id === activeCategory.value) return cat.label
      if (cat.children) {
        const child = cat.children.find(c => c.id === activeCategory.value)
        if (child) return child.label
      }
    }
    return ''
  })

  /**
   * isReady 变化时自动切换分类
   *
   * 变 true 时优先从 URL `?tab=` 恢复激活项（刷新页面保留路径），
   * URL 无效才默认跳到「创建房间」。
   * 变 false 时强制切回「设备」（JWT 过期 / 退出登录）。
   *
   * 注意：NavSidebar 自身 onMounted 也会读 route.query.tab，但 categories
   * 依赖 isReady，refreshStatus 异步完成前 categories 还不含 room/FRP 子项，
   * `isValid` 校验失败导致无法恢复。故此处是**权威恢复点**——isReady 变 true
   * 时 categories 已就绪，完整恢复所有合法 tab。
   */
  watch(isReady, (ready) => {
    // 云端离线：强制停留「设备」，避免自动切到被封禁的分类
    if (!onlineStore.cloudConnected) {
      if (activeCategory.value !== 'device') activeCategory.value = 'device'
      return
    }
    if (ready) {
      // 已进入房间时，无论 URL tab 如何，房间详情始终优先（角色由 store 保留）
      if (isInRoom.value) {
        activeCategory.value = 'room_details'
        return
      }
      const tab = route.query.tab as string | undefined
      if (tab && isValidCategory(tab)) {
        if (tab === 'room_details' && !isInRoom.value) {
          // 未在房间时「房间详情」disabled，回退到创建房间
          activeCategory.value = 'create'
        } else {
          activeCategory.value = tab as OnlineCategoryId
        }
      } else if (activeCategory.value === 'device') {
        // 登录成功且 URL 无有效 tab → 默认跳到创建房间
        activeCategory.value = 'create'
      }
    } else if (activeCategory.value !== 'device') {
      // 未就绪（退出登录 / 认证失败）→ 切回设备；JWT 过期不触发（由后端自动续期）
      activeCategory.value = 'device'
    }
  }, { immediate: true })

  /**
   * 房间状态变化时自动切换分类
   *
   * - 进入房间（role: null → host/guest）：自动切到「房间详情」
   * - 离开房间（role: host/guest → null）：若当前在「房间详情」，切回创建/加入
   */
  watch(isInRoom, (inRoom) => {
    if (inRoom) {
      // 进入房间 → 自动跳到房间详情
      activeCategory.value = 'room_details'
    } else if (activeCategory.value === 'room_details') {
      // 离开房间且当前停在房间详情 → 切回创建房间
      activeCategory.value = 'create'
    }
  })

  /**
   * 云端连接状态变化：断开时若停留在被封禁的分类（房间/大厅）强制切回「设备」，
   * 避免内容区仍渲染 RoomManager/Lobby 而侧边栏已封禁的不一致状态
   */
  watch(
    () => onlineStore.cloudConnected,
    (connected) => {
      if (!connected && !onlineStore.initializing && activeCategory.value !== 'device') {
        activeCategory.value = 'device'
      }
    },
  )

  return {
    status,
    isReady,
    isInRoom,
    categories,
    badge,
    activeDesc,
    activeLabel,
  }
}
