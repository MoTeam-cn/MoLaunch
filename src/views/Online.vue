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

import { ref, computed, watch, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useOnlineStore } from '@/stores/online'
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
} from '@heroicons/vue/24/outline'
import type { Component } from 'vue'

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

/** 当前激活分类（device / create / join） */
const activeCategory = ref<'device' | 'create' | 'join'>('device')

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

/** 实际渲染的分类列表：未就绪时只显示「设备」 */
const categories = computed<NavCategory[]>(() => {
  return isReady.value ? [deviceCategory, roomCategory] : [deviceCategory]
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
    } else if (activeCategory.value === 'device') {
      // 登录成功且 URL 无有效 tab → 默认跳到创建房间
      activeCategory.value = 'create'
    }
  } else if (activeCategory.value !== 'device') {
    // JWT 过期 / 退出登录 → 切回设备
    activeCategory.value = 'device'
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
        <RoomManager v-else :mode="activeCategory" />
      </div>
    </div>
  </div>
</template>
