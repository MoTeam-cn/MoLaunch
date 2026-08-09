<script setup lang="ts">
/**
 * MoLaunch 根组件
 */

import { onMounted, ref } from 'vue'
import router from '@/router'
import TopNavLayout from '@/components/layout/TopNavLayout.vue'
import BackToTop from '@/components/common/BackToTop.vue'
import DownloadPanel from '@/components/common/DownloadPanel.vue'
import DragOverlay from '@/components/common/DragOverlay.vue'
import MessageDrawer from '@/components/common/MessageDrawer.vue'
import CrashDialog from '@/components/common/CrashDialog.vue'
import HintDialog from '@/components/common/HintDialog.vue'
import UserAgreementDialog from '@/components/common/UserAgreementDialog.vue'
import Toast from '@/components/common/Toast.vue'
import UpdateDialog from '@/components/about/UpdateDialog.vue'
import UpdateLogDialog from '@/components/about/UpdateLogDialog.vue'
import Watermark from '@/components/common/Watermark.vue'
import { useSdkStore } from '@/stores/sdk'
import { useAuthStore } from '@/stores/auth'
import { useJavaStore } from '@/stores/java'
import { useSettingsStore } from '@/stores/settings'
import { useOnlineStore } from '@/stores/online'
import { setModalRef, showError } from '@/utils/modal'
import { setCrashDialogRef } from '@/utils/crashDialog'
import { setBuyHintDialogRef } from '@/utils/buyHint'
import { setStarHintDialogRef } from '@/utils/starHint'
import { setUserAgreementDialogRef, maybeRequireUserAgreement } from '@/utils/userAgreement'
import { setToastRef } from '@/utils/toast'
import { notifyFrontendReady } from '@/utils/splash'
import { initAutoCheck } from '@/utils/updater'
import { maybeShowUpdateLog } from '@/utils/updateLog'
import { consumeRelaunchRestore } from '@/utils/relaunchRestore'
import { initDownloadStream } from '@/composables/useDownloadStream'
import { useDragDrop } from '@/composables/useDragDrop'
import { useDevToolsGuard } from '@/composables/useDevToolsGuard'

const sdkStore = useSdkStore()
const authStore = useAuthStore()
const javaStore = useJavaStore()
const settingsStore = useSettingsStore()
const onlineStore = useOnlineStore()
const modalRef = ref<InstanceType<typeof MessageDrawer> | null>(null)
const crashDialogRef = ref<InstanceType<typeof CrashDialog> | null>(null)
const hintDialogRef = ref<InstanceType<typeof HintDialog> | null>(null)
const userAgreementDialogRef = ref<InstanceType<typeof UserAgreementDialog> | null>(null)
const toastRef = ref<InstanceType<typeof Toast> | null>(null)
// 会话恢复期间的加载遮罩，避免未恢复登录态就触发其他页面 invoke
const isRestoring = ref(true)

// 注册全局拖拽事件监听（必须在 onMounted 中调用，onUnmounted 自动清理）
useDragDrop()
// 全局 DevTools 防护：禁用右键菜单 + 拦截 F12/Ctrl+Shift+I 等 devtools 快捷键
// 后端 open_devtools action 仍可在开发者模式开启时调出，普通用户无法绕过
useDevToolsGuard()

onMounted(() => {
  console.log('[Startup][Frontend] App.vue onMounted @', new Date().toISOString())
  setModalRef(modalRef.value)
  setCrashDialogRef(crashDialogRef.value)
  setBuyHintDialogRef(hintDialogRef.value)
  setStarHintDialogRef(hintDialogRef.value)
  setUserAgreementDialogRef(userAgreementDialogRef.value)
  setToastRef(toastRef.value)
  initDownloadStream()
  // 前端已挂载：至少展示 SPLASH_MIN_MS 后关闭开屏窗口、显示主窗口
  notifyFrontendReady()
  // 启动自动更新检查（启动后 5s + 每 6 小时；dev 模式自动跳过）
  initAutoCheck()
  initApp()
})

async function initApp() {
  console.log('[Startup][Frontend] initApp start @', new Date().toISOString())
  // 全局《用户协议》门禁：首次启动需同意后才能使用（fire-and-forget，弹窗以高 z-index 覆盖加载遮罩，不影响后续初始化）
  void maybeRequireUserAgreement()
  console.log('[Startup][Frontend] maybeRequireUserAgreement fired')
  // Java 搜索不依赖 SDK/认证，提前并行启动（后端 list_java 是耗时操作）
  const javaPromise = javaStore.detectJava()
  console.log('[Startup][Frontend] javaStore.detectJava fired (background)')

  // 异步从后端 INI 同步主题色（不阻塞主流程，完成后覆盖前端 localStorage）
  settingsStore.syncPrimaryColorFromBackend()
  console.log('[Startup][Frontend] syncPrimaryColorFromBackend fired (background)')

  // 平台信息和设备 ID 互不依赖，并行获取
  console.log('[Startup][Frontend] awaiting fetchPlatformInfo + fetchDeviceId...')
  await Promise.all([
    sdkStore.fetchPlatformInfo(),
    sdkStore.fetchDeviceId(),
  ])
  console.log('[Startup][Frontend] fetchPlatformInfo + fetchDeviceId done @', new Date().toISOString())

  // 恢复登录状态
  console.log('[Startup][Frontend] awaiting authStore.restoreSession...')
  await authStore.restoreSession()
  console.log('[Startup][Frontend] restoreSession done @', new Date().toISOString())

  // 登录态已恢复，关闭加载遮罩
  isRestoring.value = false

  // 恢复管理员提权重启前的页面（若保存过且路径合法、已登录）
  // 放在"修正路由"之前，避免恢复路径被 /login 兜底逻辑覆盖
  const savedPath = consumeRelaunchRestore()
  if (
    savedPath &&
    authStore.isLoggedIn &&
    savedPath.startsWith('/apps') &&
    router.resolve(savedPath).matched.length > 0
  ) {
    await router.replace(savedPath)
  }

  // 联机云端初始化（静默注册/登录/刷新 token）
  // 失败不阻塞主流程，仅弹窗提示并禁用联机功能
  console.log('[Startup][Frontend] awaiting onlineStore.initAuth...')
  const cloudOk = await onlineStore.initAuth()
  console.log('[Startup][Frontend] initAuth done, cloudConnected =', cloudOk)
  if (!cloudOk) {
    // 云端连接失败，弹窗提示用户（联机按钮将自动禁用）
    showError(
      '云端连接失败',
      '与云端 API 连接失败，联机功能可能不可用。',
      `原因：${onlineStore.cloudError || '未知错误'}\n可在「设置 → 联机」页尝试重新连接。`,
    )
  }

  // 会话恢复后修正路由：
  // - 已登录但停在 /login → 跳首页
  // - 未登录但停在 requiresAuth 页面 → 跳登录页
  // （恢复期间守卫放行，可能导致用户停在错误页面，这里主动修正）
  const currentRoute = router.currentRoute.value
  if (authStore.isLoggedIn && currentRoute.path === '/login') {
    router.replace('/apps')
  } else if (!authStore.isLoggedIn && currentRoute.meta.requiresAuth) {
    router.replace('/login')
  }

  // 等待 Java 搜索完成（不阻塞 UI 和路由修正）
  await javaPromise
  // 启动「本次更新日志」弹窗：版本升级后展示一次（无记录/回退/同版本不弹）
  maybeShowUpdateLog()
  console.log('[Startup][Frontend] initApp fully done @', new Date().toISOString())
}
</script>

<template>
  <TopNavLayout>
    <!--
      wrapper div 提供 relative 定位上下文，让路由切换时离开组件的
      position: absolute 相对于此容器而非 main（main 有 p-2 padding）。
      h-full 继承 main 的高度（main 是 flex-1 子项，有确定高度）。
    -->
    <div id="app-content" class="relative h-full">
      <router-view v-slot="{ Component, route }">
        <!--
          不使用 mode="out-in"：Vue 3 transition 在 mode="out-in" 下，联机页
          （含 WebRTC composable）离开动画完成后（afterLeave 触发）新组件的
          beforeEnter 不触发，导致目标页面空白。改用默认模式（同时进出），
          新组件会立即挂载，避免空白。leave-active 设置 position: absolute
          让离开组件脱离布局流，避免两个组件垂直排列导致高度溢出。
        -->
        <transition name="route">
          <component :is="Component" :key="route.path" />
        </transition>
      </router-view>
      <!-- 全局拖拽遮蔽层：挂载在 nav 下方的内容容器内（absolute 铺满），
           物理上不覆盖顶部 nav，顶部虚线不会穿到 nav 区域 -->
      <DragOverlay />
    </div>
  </TopNavLayout>
  <Teleport to="body">
    <BackToTop />
    <DownloadPanel />
  </Teleport>
  <MessageDrawer ref="modalRef" />
  <CrashDialog ref="crashDialogRef" />
  <HintDialog ref="hintDialogRef" />
  <UserAgreementDialog ref="userAgreementDialogRef" />
  <Toast ref="toastRef" />
  <UpdateDialog />
  <UpdateLogDialog />
  <!-- 测试版水印：仅在测试版构建（package.json version 含 beta/alpha/rc/canary 后缀）时渲染 -->
  <Watermark />
  <!-- 会话恢复期间的加载遮罩 -->
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="isRestoring" class="fixed inset-0 z-[9998] flex items-center justify-center bg-white/80 backdrop-blur-sm">
        <div class="flex flex-col items-center gap-3">
          <div class="h-8 w-8 animate-spin rounded-full border-[3px] border-primary-200 border-t-primary-500" />
          <p class="text-sm text-gray-500">正在加载...</p>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style>
/* 加载遮罩过渡（isRestoring） */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* 路由切换过渡（默认模式，非 out-in） */
.route-enter-active,
.route-leave-active {
  transition: opacity 0.25s ease, transform 0.25s ease;
}

.route-enter-from {
  opacity: 0;
  transform: translateY(6px);
}

.route-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}

/* 离开组件脱离布局流，避免与新组件垂直排列导致高度溢出 */
.route-leave-active {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
}
</style>
