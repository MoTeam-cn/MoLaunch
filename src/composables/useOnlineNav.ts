/**
 * Online 联机页导航分类与状态联动 composable
 *
 * 分类扁平化为：设备 / 联机大厅 / 创建房间 / 加入房间（+ FRP 子菜单）。
 * 就绪判定仅依赖「已注册 + 已登录」（不判断本地 token 过期，由后端自动续期）；
 * isReady watch 从 URL ?tab= 恢复激活项，isInRoom watch 进出房间自动切换分类。
 */

import { computed, watch, type Ref, type Component } from 'vue'
import { useRoute } from 'vue-router'
import {
  DevicePhoneMobileIcon,
  GlobeAltIcon,
  PlusIcon,
  ArrowRightOnRectangleIcon,
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
  /** 禁用态：未满足前置条件时灰色不可点击 */
  disabled?: boolean
  /** 封禁态：云端离线时灰色置灰但点击仍会触发，由 Online.vue 拦截弹窗告知原因 */
  sealed?: boolean
}

/** Online 页激活分类 ID 联合类型（tutorial 为侧边栏动作项，不会真正成为激活态） */
export type OnlineCategoryId =
  | 'device' | 'lobby' | 'create' | 'join'
  | 'providers' | 'tunnels' | 'auth' | 'logs' | 'tutorial'

/** 设备分类（始终可用） */
const deviceCategory: NavCategory = {
  id: 'device',
  label: '设备',
  icon: DevicePhoneMobileIcon,
  desc: '注册联机设备、登录获取访问凭证、查看设备 ID 与 JWT 状态',
}

/** 联机大厅分类（浏览公开房间，按整合包聚类） */
const lobbyCategory: NavCategory = {
  id: 'lobby',
  label: '联机大厅',
  icon: GlobeAltIcon,
  desc: '浏览公开房间列表，按整合包聚类展示并一键加入',
}

/** 创建房间分类（选整合包 → 生成房间码 → 拉起联机中心） */
const createCategory: NavCategory = {
  id: 'create',
  label: '创建房间',
  icon: PlusIcon,
  desc: '选择整合包、生成房间码并作为房主拉起联机中心',
}

/** 加入房间分类（输码 → 加入网络 → 探测进服地址） */
const joinCategory: NavCategory = {
  id: 'join',
  label: '加入房间',
  icon: ArrowRightOnRectangleIcon,
  desc: '输入 6 位公开标识加入已有房间',
}

/** URL `?tab=` 可恢复的合法分类 ID（device/lobby/create/join + FRP 子项） */
const VALID_TABS = new Set<OnlineCategoryId>([
  'device', 'lobby', 'create', 'join',
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
   * 联机功能是否就绪（显示联机大厅 / 创建房间 / 加入房间 / FRP 分类）
   *
   * 仅判断「已注册 + 已登录」，**不判断 token_expired**：
   * 后端 `load_creds_with_auto_refresh` 会在业务 action 调用前自动 refresh 续期，
   * 前端 `onlineManager` 也有 1003 → refresh → login → register 降级链兜底。
   */
  const isReady = computed(
    () => !!status.value && status.value.registered && status.value.logged_in,
  )

  /** 是否在房间中（role=host/guest） */
  const isInRoom = computed(() => onlineStore.roomState.role !== null)

  /**
   * 实际渲染的分类列表
   *
   * 云端离线（cloudConnected=false 且初始化已完成）时联机分类封禁置灰
   * （可点击但由 Online.vue 拦截弹窗告知原因），FRP 不受云端影响仍可用。
   */
  const categories = computed<NavCategory[]>(() => {
    const offline = !onlineStore.cloudConnected && !onlineStore.initializing
    if (offline) {
      return [
        deviceCategory,
        { ...lobbyCategory, sealed: true },
        { ...createCategory, sealed: true },
        { ...joinCategory, sealed: true },
        frpCategory,
      ]
    }
    if (!isReady.value) return [deviceCategory]
    return [deviceCategory, lobbyCategory, createCategory, joinCategory, frpCategory]
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

  /** 当前激活分类的描述 */
  const activeDesc = computed(() => {
    for (const cat of categories.value) {
      if (cat.id === activeCategory.value) return cat.desc ?? ''
    }
    return ''
  })

  /** 当前激活分类的标签 */
  const activeLabel = computed(() => {
    for (const cat of categories.value) {
      if (cat.id === activeCategory.value) return cat.label
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
   * 依赖 isReady，refreshStatus 异步完成前 categories 还不含联机分类，
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
      // 已进入房间时，无论 URL tab 如何，直接切到对应房主/房客面板
      if (isInRoom.value) {
        activeCategory.value = onlineStore.roomState.role === 'guest' ? 'join' : 'create'
        return
      }
      const tab = route.query.tab as string | undefined
      if (tab && isValidCategory(tab)) {
        activeCategory.value = tab as OnlineCategoryId
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
   * - 进入房间（role: null → host/guest）：房主切「创建房间」、房客切「加入房间」，
   *   内容区 RoomManager 按 role 渲染对应面板（进入后不退出，无「房间详情」子项）
   * - 离开房间（role: host/guest → null）：保持当前分类，RoomManager 恢复表单
   */
  watch(isInRoom, (inRoom) => {
    if (inRoom) {
      activeCategory.value = onlineStore.roomState.role === 'guest' ? 'join' : 'create'
    }
  })

  /**
   * 云端连接状态变化：断开时若停留在被封禁的分类（大厅/创建/加入）强制切回「设备」，
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
