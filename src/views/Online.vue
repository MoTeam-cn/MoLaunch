<script setup lang="ts">
/**
 * 联机主页
 *
 * 采用与 [Settings.vue](src/views/Settings.vue) 一致的侧边栏布局：
 * - 左侧 NavSidebar：设备 / 房间管理（房间管理下有「创建房间」「加入房间」两个子项）
 * - 右侧内容区：根据 activeCategory 切换 OnlineDevicePanel / RoomManager
 *
 * 状态联动：
 * - 未注册 / 未登录时仅显示「设备」分类
 * - 登录成功（isReady=true）后自动追加「房间管理」分类并展开子项，选中「创建房间」
 * - 未就绪（退出登录等）自动切回「设备」分类；JWT 过期不切回（后端自动续期兜底）
 * - 已进入房间时（role=host/guest），RoomManager 自动显示对应面板
 *
 * 拆分（保持主文件 ≤ 300 行约束）：
 * - useOnlineNav：导航分类配置 + categories/badge/activeDesc/activeLabel + watch 联动
 * - OnlineTopBar：顶部标题栏 + 状态徽章 + 设置按钮
 * - useFrpSidebar：FRP 侧边栏子菜单（含「教程帮助」动作项，点击跳转设置-教程页）
 */

import { ref, computed, onMounted, provide } from 'vue'
import { useRouter } from 'vue-router'
import { useOnlineStore } from '@/stores/online'
import { getOnlineSession } from '@/composables/online/onlineSession'
import NavSidebar from '@/components/common/NavSidebar.vue'
import OnlineDevicePanel from '@/components/online/OnlineDevicePanel.vue'
import RoomManager from '@/components/online/RoomManager.vue'
import LobbyBrowser from '@/components/online/LobbyBrowser.vue'
import ProviderList from '@/components/frp/ProviderList.vue'
import TunnelManager from '@/components/frp/TunnelManager.vue'
import FrpLogs from '@/components/frp/FrpLogs.vue'
import AuthCenter from '@/components/frp/AuthCenter.vue'
import OnlineTopBar from '@/views/online/OnlineTopBar.vue'
import DisclaimerDialog from '@/components/common/DisclaimerDialog.vue'
import { useOnlineNav, type OnlineCategoryId } from '@/composables/useOnlineNav'
import { hasAgreedToday } from '@/utils/disclaimer'
import { showWarning } from '@/utils/modal'

/**
 * WebRTC 实例全局化（联机会话挂载）
 *
 * 原实现：实例提升到 Online.vue 页面级，离开联机页（路由切走）时 Online.vue
 * 卸载 → onUnmounted 触发 close() → P2P 断开、虚拟网卡丢失。
 *
 * 现实现：实例由全局联机会话 onlineSession 持有（App 级初始化，常驻整个应用
 * 生命周期），Online.vue 仅从会话取出实例并 provide。切换侧边栏菜单或离开
 * 联机页面都不会断开连接，回到联机页直接恢复。
 *
 * provide key 与原 RoomManager.vue 保持一致（'hostMesh' / 'guestWebrtc'），
 * 子组件 RoomHostPanel / RoomGuestPanel 的 inject 链路无需改动。
 */
const HOST_MESH_KEY = 'hostMesh'
const GUEST_WEBRTC_KEY = 'guestWebrtc'
const { hostMesh, guestWebrtc } = getOnlineSession()
provide(HOST_MESH_KEY, hostMesh)
provide(GUEST_WEBRTC_KEY, guestWebrtc)

const router = useRouter()
const onlineStore = useOnlineStore()

/** 当前激活分类（device / lobby / create / join / room_details / providers / tunnels / auth / logs） */
const activeCategory = ref<OnlineCategoryId>('device')

/** 使用协议抽屉：当日未同意过协议时进入联机页弹出（同意后存 localStorage，次日重新提醒） */
const disclaimerVisible = ref(!hasAgreedToday('online'))

const { categories, badge, activeDesc, activeLabel } = useOnlineNav(activeCategory)

/** 当前分类中处于封禁态（云端离线）的 id → 名称 映射，用于点击时弹窗告知原因 */
const sealedCategories = computed<Record<string, string>>(() => {
  const map: Record<string, string> = {}
  for (const cat of categories.value) {
    if (cat.sealed) map[cat.id] = cat.label
    for (const child of cat.children ?? []) {
      if (child.sealed) map[child.id] = child.label
    }
  }
  return map
})

/**
 * 跳转到 Frp 日志页查看指定隧道
 *
 * TunnelManager 卡片「查看日志」按钮调用，切换到 logs 分类并预选 tunnelId。
 * 用 provide/inject 而非 props，避免组件层级耦合（TunnelManager 通过 keep-alive 缓存）。
 *
 * 必须放在 activeCategory 声明之后（const 未 hoist）。
 */
function goToLogs(tunnelId: string): void {
  activeCategory.value = 'logs'
  // 直接写入 frp store，FrpLogs onMounted 会读取该值
  // 这里用动态导入避免循环依赖
  void import('@/stores/frp').then(({ useFrpStore }) => {
    useFrpStore().selectedLogTunnelId = tunnelId
  })
}
provide('goToLogs', goToLogs)

onMounted(() => {
  // 云端未连接时不发起任何网络请求，避免无意义失败
  if (!onlineStore.cloudConnected) return
  void onlineStore.refreshStatus()
  // 进入联机页自动检测 NAT 类型（已有结果时跳过，结果保留在 store 中侧边栏切换不丢失）
  void onlineStore.detectNat()
})

function goSettings() {
  router.push('/apps/settings?tab=online')
}

/**
 * 跳转到设置 - 更多 - 教程子页
 *
 * FRP 侧边栏「教程帮助」子项点击调用。通过 `tab=about` 切到「更多」分类，
 * `subtab=tutorial` 由 SettingsMore.vue 读取后切换到「教程」子页签。
 */
function goTutorial() {
  router.push('/apps/settings?tab=about&subtab=tutorial')
}

/**
 * 侧边栏分类切换处理器
 *
 * 'tutorial' 是动作项（跳转到设置-教程页），不切换 activeCategory；
 * 封禁态分类（云端离线时的房间/大厅）点击后弹窗告知原因，不切换；
 * 其余分类直接赋值。
 */
function handleCategoryChange(id: string): void {
  if (id === 'tutorial') {
    goTutorial()
    return
  }
  const sealedLabel = sealedCategories.value[id]
  if (sealedLabel) {
    showWarning(
      '功能已封存',
      `「${sealedLabel}」需要连接云端服务，当前云端连接失败，暂不可用。`,
      onlineStore.cloudError ?? undefined,
    )
    return
  }
  activeCategory.value = id as OnlineCategoryId
}

// ============================================================
// keep-alive 动态组件（侧边栏切换时保留各面板状态）
// ============================================================
// OnlineDevicePanel / LobbyBrowser / RoomManager 各自缓存，
// 切换侧边栏菜单时仅 deactivate → activate，不触发 onUnmounted，
// 表单输入 / 搜索结果 / 分页位置等组件级状态完整保留。
// 房间连接（WebRTC 实例）由 Online.vue provide，不受影响。
const currentComponent = computed(() => {
  switch (activeCategory.value) {
    case 'device': return OnlineDevicePanel
    case 'lobby': return LobbyBrowser
    case 'providers': return ProviderList
    case 'tunnels': return TunnelManager
    case 'auth': return AuthCenter
    case 'logs': return FrpLogs
    default: return RoomManager
  }
})

/** 仅 RoomManager 需要 mode prop，其余组件传空对象避免 fallthrough */
const currentProps = computed<Record<string, unknown>>(() => {
  if (activeCategory.value === 'device' || activeCategory.value === 'lobby') return {}
  const mode: 'create' | 'join' = activeCategory.value === 'room_details'
    ? (onlineStore.roomState.role === 'guest' ? 'join' : 'create')
    : (activeCategory.value === 'join' ? 'join' : 'create')
  return { mode }
})
</script>

<template>
  <!--
    单根包裹：App.vue 的 <transition mode="out-in"> 要求子组件为单根，
    多根（v-if/v-else 两个根 div）会导致路由切换时 transition 卡住、
    新组件无法挂载（表现为切走联机页后其他页面空白）。
    外层 div 保持 h-full 以继承父容器高度。
  -->
  <div class="h-full">
    <!--
      云端离线时不再整页遮罩：页面可进入，房间管理/联机大厅由 useOnlineNav 置为封禁态
      （灰色 + 锁图标，点击弹窗告知原因）；FRP（第三方隧道）不依赖 MoLaunch 云端仍可用。
    -->
    <div class="flex h-full rounded-xl overflow-hidden bg-white shadow-sm">
      <!-- 左侧分类菜单（支持子菜单展开动画） -->
      <NavSidebar :model-value="activeCategory" :categories="categories" @update:model-value="handleCategoryChange" />

      <!-- 右侧内容区 -->
      <div class="flex-1 flex flex-col overflow-hidden">
        <!-- 顶部标题栏 -->
        <OnlineTopBar
          :active-label="activeLabel"
          :active-desc="activeDesc"
          :badge="badge"
          @go-settings="goSettings"
        />

        <!-- 内容区（keep-alive 缓存各面板，侧边栏切换时保留状态） -->
        <div class="flex-1 overflow-y-auto p-6">
          <keep-alive>
            <component :is="currentComponent" v-bind="currentProps" />
          </keep-alive>
        </div>
      </div>
    </div>

    <!-- 使用协议抽屉（当日未同意时展示；teleport 到 #app-content，位置不影响单根约束） -->
    <DisclaimerDialog v-model:visible="disclaimerVisible" kind="online" />
  </div>
</template>
