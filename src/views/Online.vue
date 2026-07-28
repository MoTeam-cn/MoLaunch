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
 * - JWT 过期（isReady=false）自动切回「设备」分类
 * - 已进入房间时（role=host/guest），RoomManager 自动显示对应面板
 */

import { ref, computed, watch, onMounted, provide } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useOnlineStore } from '@/stores/online'
import { useWebRTC } from '@/composables/useWebRTC'
import { useWebRTCMesh } from '@/composables/useWebRTCMesh'
import NavSidebar from '@/components/common/NavSidebar.vue'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import OnlineDevicePanel from '@/components/online/OnlineDevicePanel.vue'
import RoomManager from '@/components/online/RoomManager.vue'
import {
  Cog6ToothIcon,
  DevicePhoneMobileIcon,
  ServerStackIcon,
  PlusIcon,
  ArrowRightOnRectangleIcon,
  HomeIcon,
} from '@heroicons/vue/24/outline'
import type { Component } from 'vue'

/**
 * WebRTC 实例提升到页面级（房间挂起改造）
 *
 * 原实现：RoomManager.vue 内创建 hostMesh / guestWebrtc，切换侧边栏菜单时
 * RoomManager 被 v-if 卸载 → onUnmounted 触发 close() → WebRTC 连接断开。
 *
 * 现实现：实例提升到 Online.vue，RoomManager 改为 inject。切换侧边栏菜单
 * （device ↔ create ↔ join）时 RoomManager 卸载但 WebRTC 实例保持，
 * 房间连接不断。仅离开联机页面（Online.vue 卸载）时才 close()。
 *
 * provide key 与原 RoomManager.vue 保持一致（'hostMesh' / 'guestWebrtc'），
 * 子组件 RoomHostPanel / RoomGuestPanel 的 inject 链路无需改动。
 */
const HOST_MESH_KEY = 'hostMesh'
const GUEST_WEBRTC_KEY = 'guestWebrtc'
const hostMesh = useWebRTCMesh()
const guestWebrtc = useWebRTC()
provide(HOST_MESH_KEY, hostMesh)
provide(GUEST_WEBRTC_KEY, guestWebrtc)

interface NavCategory {
  id: string
  label: string
  icon: Component
  desc?: string
  children?: NavCategory[]
}

const route = useRoute()
const router = useRouter()
const onlineStore = useOnlineStore()

const status = computed(() => onlineStore.deviceStatus)
const isReady = computed(
  () => !!status.value && status.value.registered && status.value.logged_in && !status.value.token_expired,
)

/** 当前激活分类（device / create / join / room_details） */
const activeCategory = ref<'device' | 'create' | 'join' | 'room_details'>('device')

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

/** 是否在房间中（role=host/guest） */
const isInRoom = computed(() => onlineStore.roomState.role !== null)

/** 房间详情子项（仅在房间中时追加到 room 管理子菜单） */
const roomDetailsChild: NavCategory = {
  id: 'room_details',
  label: '房间详情',
  icon: HomeIcon,
  desc: '查看当前房间状态、参与者列表与连接信息',
}

/** 实际渲染的分类列表：未就绪时只显示「设备」；在房间中时追加「房间详情」子项 */
const categories = computed<NavCategory[]>(() => {
  if (!isReady.value) return [deviceCategory]
  // 在房间中时动态追加「房间详情」子项到 room 分类
  const roomWithDetails: NavCategory = {
    ...roomCategory,
    children: [...roomCategory.children!, roomDetailsChild],
  }
  return [deviceCategory, roomWithDetails]
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
    if (tab === 'create' || tab === 'join' || tab === 'room_details') {
      activeCategory.value = tab
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

onMounted(() => {
  void onlineStore.refreshStatus()
})

function goSettings() {
  router.push('/apps/settings?tab=online')
}
</script>

<template>
  <div class="flex h-full rounded-xl overflow-hidden bg-white shadow-sm">
    <!-- 左侧分类菜单（支持子菜单展开动画） -->
    <NavSidebar v-model="activeCategory" :categories="categories" />

    <!-- 右侧内容区 -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <!-- 顶部标题栏 -->
      <div class="px-6 py-4 bg-white border-b border-gray-200 shrink-0">
        <div class="flex items-center justify-between">
          <div>
            <h2 class="text-lg font-semibold text-gray-900">{{ activeLabel }}</h2>
            <p class="text-xs text-gray-500 mt-1">{{ activeDesc }}</p>
          </div>
          <div class="flex items-center gap-2">
            <span
              class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium"
              :class="badge.wrapClass"
            >
              <span class="w-1.5 h-1.5 rounded-full mr-1.5" :class="badge.dotClass" />
              {{ badge.text }}
            </span>
            <Tooltip text="联机设置">
              <Button type="ghost" size="small" @click="goSettings">
                <template #icon><Cog6ToothIcon class="w-4 h-4" /></template>
              </Button>
            </Tooltip>
          </div>
        </div>
      </div>

      <!-- 内容区 -->
      <div class="flex-1 overflow-y-auto p-6">
        <OnlineDevicePanel v-if="activeCategory === 'device'" />
        <RoomManager
          v-else
          :mode="activeCategory === 'room_details' ? (onlineStore.roomState.role === 'guest' ? 'join' : 'create') : activeCategory"
        />
      </div>
    </div>
  </div>
</template>
