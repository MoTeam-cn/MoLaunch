<script setup lang="ts">
/**
 * 顶部导航布局组件
 */

import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { invoke } from '@tauri-apps/api/core'
import * as tauri from '@/utils/tauri'
import { safeCall } from '@/utils/async'
import { useOnlineStore } from '@/stores/online'
import { applyPendingUpdate, checkForUpdate } from '@/utils/updater'
import { getConfigMap, applyConfig } from '@/utils/api/config'
import { toastError } from '@/utils/toast'
import Tooltip from '@/components/common/Tooltip.vue'
import ExitConfirmDialog from './ExitConfirmDialog.vue'
import { topNavItems, experimentalNavItem } from './topNavItems'
import { useTauriEvent } from '@/composables/useTauriEvent'
import { useExperimental } from '@/composables/useExperimental'
const appWindow = getCurrentWebviewWindow()
const onlineStore = useOnlineStore()
const { enabled: experimentalEnabled } = useExperimental()

const router = useRouter()
const route = useRoute()
const isMaximized = ref(false)
const unlistenResized = ref<(() => void) | null>(null)

const visibleNavItems = computed(() => {
  const items = topNavItems.filter((i) => i.path !== experimentalNavItem.path)
  if (experimentalEnabled.value) {
    const idx = items.findIndex((i) => i.path === '/apps/settings')
    if (idx >= 0) {
      items.splice(idx, 0, experimentalNavItem)
    } else {
      items.push(experimentalNavItem)
    }
  }
  return items
})

// 双击计时器
let lastClickTime = 0
let clickTimer: ReturnType<typeof setTimeout> | null = null

function isActive(path: string): boolean {
  return route.path === path
}

function navigateTo(path: string) {
  router.push(path)
}

function handleDownloadClick() {
  const now = Date.now()
  const timeDiff = now - lastClickTime

  if (timeDiff < 300) {
    // 双击：进入下载管理页面
    if (clickTimer) {
      clearTimeout(clickTimer)
      clickTimer = null
    }
    router.push('/apps/downloads')
  } else {
    // 单击：延迟执行导航到版本页面
    clickTimer = setTimeout(() => {
      router.push('/apps/versions')
      clickTimer = null
    }, 300)
  }

  lastClickTime = now
}

// 退出选择弹框（close_behavior === 'ask' 时显示）
const showExitDialog = ref(false)
// 退出防重入：托盘退出事件与关闭按钮可能并发触发 doExit
const exiting = ref(false)

/**
 * 关闭主窗口入口（右上角关闭按钮 / 后端 Alt+F4 等关闭请求统一走这里）
 *
 * 按持久化的 closeBehavior 分流：
 * - ask：弹出"直接退出 / 保留托盘"选择框
 * - tray：隐藏主界面（进程 / 全局 keepalive 定时器继续运行）
 * - exit：执行退出清理后退出
 */
async function handleClose() {
  if (exiting.value) return
  const cfg = await safeCall(() => getConfigMap(), 'read config before close')
  const behavior = cfg?.closeBehavior ?? 'ask'
  if (behavior === 'ask') {
    showExitDialog.value = true
    return
  }
  if (behavior === 'tray') {
    await appWindow.hide().catch(() => toastError('隐藏主界面失败，请重试'))
    return
  }
  await doExit()
}

/**
 * 直接退出：前端完成配置保存 / 联机退房 / 待安装更新等清理，
 * 再通知后端统一清理 frpc 隧道与 TUN 虚拟网卡后退出进程。
 */
async function doExit() {
  if (exiting.value) return
  exiting.value = true
  // 关闭窗口前先保存配置
  await safeCall(() => tauri.saveConfigToFile(), 'save config before close')
  // 联机状态清理：根据角色通知服务端退出房间，避免僵尸房间/参与者记录
  // 3s 超时保护，网络问题不卡住关窗；API 失败不阻塞关窗（服务端 keepalive 超时兜底）
  const role = onlineStore.roomState.role
  if (role === 'host') {
    await Promise.race([
      onlineStore.hostCloseRoom().catch(() => toastError('离开房间失败')),
      new Promise<void>((r) => setTimeout(r, 3000)),
    ])
  } else if (role === 'guest') {
    await Promise.race([
      onlineStore.guestLeaveRoom().catch(() => toastError('离开房间失败')),
      new Promise<void>((r) => setTimeout(r, 3000)),
    ])
  }
  // Windows 便携版：退出前检查是否有待安装更新（appdata/last.exe）
  // 有则启动 updater.exe 替换主 exe，退出后 updater 接管，下次启动即为新版本
  await applyPendingUpdate().catch(() => false)
  // 前端清理完成，交由后端统一清理 frpc/TUN 后退出（覆盖 Alt+F4 等绕过路径）
  await invoke('request_exit').catch((e) => {
    console.error('[Exit] request_exit failed:', e)
    window.close()
  })
}

/** 退出选择框确认：可选"记住本次选择"，下次关闭直接生效 */
async function onExitConfirm({ action, remember }: { action: 'exit' | 'tray'; remember: boolean }) {
  if (remember) {
    await safeCall(
      () => applyConfig({ closeBehavior: action }),
      'save close behavior',
      () => toastError('保存关闭偏好失败'),
    )
  }
  showExitDialog.value = false
  if (action === 'exit') {
    await doExit()
  } else {
    await appWindow.hide().catch(() => toastError('隐藏主界面失败，请重试'))
  }
}

// 后端关闭请求（Alt+F4 / 任务栏关闭，ask 模式下由后端转发到前端弹框）
const closeReqEvent = useTauriEvent('window-close-requested', () => {
  void handleClose()
})
// 托盘"检查更新"：复用现有检查更新流程
const trayCheckEvent = useTauriEvent('tray-check-update', () => {
  void checkForUpdate()
})

onMounted(async () => {
  closeReqEvent.start()
  trayCheckEvent.start()
  isMaximized.value = await appWindow.isMaximized()
  unlistenResized.value = await appWindow.onResized(async () => {
    isMaximized.value = await appWindow.isMaximized()
  })
})

onUnmounted(() => {
  if (unlistenResized.value) {
    unlistenResized.value()
    unlistenResized.value = null
  }
  closeReqEvent.stop()
  trayCheckEvent.stop()
})
</script>

<template>
  <div class="flex flex-col h-screen overflow-hidden">
    <!-- 顶部栏：蓝色背景 -->
    <!-- z-[10002] 确保所有弹窗（Modal z-10000 / Toast z-10001 / Tooltip z-9999）均无法遮蔽顶部 nav -->
    <!-- id="app-nav"：全局标记。全屏组件若需避开导航栏，请挂载到 #app-content（见 Drawer render-in-place） -->
    <header id="app-nav" class="shrink-0 bg-primary-600 select-none relative z-[10002]">
      <div class="h-12 flex items-center">
        <!-- 左侧拖拽区域 -->
        <div data-tauri-drag-region class="flex items-center pl-4 pr-2 h-full">
          <span class="text-sm font-bold text-white pointer-events-none">MoLaunch</span>
        </div>

        <!-- 中间拖拽空隙 -->
        <div data-tauri-drag-region class="flex-1 h-full" />

        <!-- 导航菜单 -->
        <nav class="flex items-center space-x-1">
          <!-- 保留原生 button：蓝色头部导航项依赖 active 状态切换 + 白色自定义配色，
               Button.vue 的 type 体系（蓝/灰）不适配头部蓝底场景 -->
          <Tooltip
            v-for="item in visibleNavItems"
            :key="item.path"
            :text="item.cloudDependent && !onlineStore.cloudConnected
              ? '云端连接失败，联机功能暂不可用（可在设置中重试）'
              : ''"
            position="bottom"
          >
            <button
              :disabled="item.cloudDependent && !onlineStore.cloudConnected"
              class="flex items-center px-4 py-1.5 rounded-md text-sm font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:bg-transparent"
              :class="[
                isActive(item.path)
                  || (item.hasDblClick && isActive('/apps/downloads'))
                  || (item.path === '/apps/tools' && route.path.startsWith('/apps/tools'))
                  || (item.path === '/apps/experimental' && route.path.startsWith('/apps/experimental'))
                  ? 'bg-white/20 text-white'
                  : 'text-white/70 hover:bg-white/10 hover:text-white'
              ]"
              @click="item.cloudDependent && !onlineStore.cloudConnected
                ? null
                : (item.hasDblClick ? handleDownloadClick() : navigateTo(item.path))"
            >
              <component :is="item.icon" class="w-4 h-4 mr-1.5" />
              {{ item.name }}
            </button>
          </Tooltip>
        </nav>

        <!-- 中间拖拽空隙 -->
        <div data-tauri-drag-region class="flex-1 h-full" />

        <!-- 右侧：窗口控制 -->
        <!-- 保留原生 button：窗口控制（最小化/最大化/关闭）需 h-full 撑满 48px 标题栏，
             Button.vue 的 scoped size 类固定 height 无法被 h-full 覆盖 -->
        <div class="flex items-center h-full">
          <!-- 最小化 -->
          <button
            class="h-full w-11 flex items-center justify-center hover:bg-white/10 transition-colors group"
            @click="appWindow.minimize()"
          >
            <svg class="w-3.5 h-3.5 text-white/60 group-hover:text-white" viewBox="0 0 12 12" fill="none">
              <rect x="1" y="5.5" width="10" height="1" rx="0.5" fill="currentColor" />
            </svg>
          </button>

          <!-- 最大化/还原 -->
          <button
            class="h-full w-11 flex items-center justify-center hover:bg-white/10 transition-colors group"
            @click="appWindow.toggleMaximize()"
          >
            <svg v-if="!isMaximized" class="w-3.5 h-3.5 text-white/60 group-hover:text-white" viewBox="0 0 12 12" fill="none">
              <rect x="1.5" y="1.5" width="9" height="9" rx="1" stroke="currentColor" stroke-width="1" />
            </svg>
            <svg v-else class="w-3.5 h-3.5 text-white/60 group-hover:text-white" viewBox="0 0 12 12" fill="none">
              <rect x="3.5" y="0.5" width="7.5" height="7.5" rx="1" stroke="currentColor" stroke-width="1" />
              <path d="M1 3.5V10C1 10.5523 1.44772 11 2 11H8.5" stroke="currentColor" stroke-width="1" />
            </svg>
          </button>

          <!-- 关闭 -->
          <button
            class="h-full w-11 flex items-center justify-center hover:bg-red-500 transition-colors group"
            @click="handleClose"
          >
            <svg class="w-3.5 h-3.5 text-white/60 group-hover:text-white" viewBox="0 0 12 12" fill="none">
              <path d="M1.5 1.5L10.5 10.5M1.5 10.5L10.5 1.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
            </svg>
          </button>
        </div>
      </div>
    </header>
    
    <!-- 主内容区：跟随主题色（primary-100/30）的淡色调背景 -->
    <main class="flex-1 overflow-hidden p-2 bg-primary-100/30">
      <slot />
    </main>

    <!-- 退出选择弹框（close_behavior === 'ask' 时弹出） -->
    <ExitConfirmDialog v-model="showExitDialog" @confirm="onExitConfirm" />
  </div>
</template>
