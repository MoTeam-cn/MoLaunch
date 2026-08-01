/**
 * Online 联机页导航分类与状态联动 composable
 *
 * 从 Online.vue 抽离（保持主文件 ≤ 300 行约束）：
 * - NavCategory 接口与 OnlineCategoryId 类型
 * - 静态分类配置（device / room / lobby）
 * - isReady / isInRoom / categories / badge / activeDesc / activeLabel 计算
 * - isReady watch：URL `?tab=` 恢复激活项 + JWT 过期切回「设备」
 * - isInRoom watch：进入房间自动跳「房间详情」，离开切回「创建房间」
 *
 * 复用项目现有 frpCategory（@/composables/useFrpSidebar），保持与原 Online.vue
 * 的 categories 末位追加逻辑一致。NavCategory 接口与 useFrpSidebar 内部定义同形，
 * 此处导出便于主文件 / NavSidebar 类型标注。
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

  const isReady = computed(
    () => !!status.value && status.value.registered && status.value.logged_in && !status.value.token_expired,
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
   * 子项 disabled 规则：
   * - 未在房间：「创建房间」「加入房间」可用，「房间详情」灰色
   * - 在房间中：「创建房间」「加入房间」灰色（必须先退出房间），「房间详情」可用
   */
  const categories = computed<NavCategory[]>(() => {
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
   * 依赖 isReady，refreshStatus 异步完成前 categories 还不含 room 子项，
   * 故此处需在 isReady 变化时再次校验 URL。
   */
  watch(isReady, (ready) => {
    if (ready) {
      const tab = route.query.tab
      if (tab === 'create' || tab === 'join') {
        activeCategory.value = tab
      } else if (tab === 'room_details' && isInRoom.value) {
        // 仅在房间中时才恢复到房间详情，否则该项 disabled 不可用
        activeCategory.value = 'room_details'
      } else if (activeCategory.value === 'device') {
        // 登录成功且 URL 无有效 tab → 默认跳到创建房间
        activeCategory.value = 'create'
      }
    } else if (activeCategory.value !== 'device') {
      // JWT 过期 / 退出登录 → 切回设备
      activeCategory.value = 'device'
    }
  })

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
