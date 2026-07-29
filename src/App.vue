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
import Modal from '@/components/common/Modal.vue'
import CrashDialog from '@/components/common/CrashDialog.vue'
import Toast from '@/components/common/Toast.vue'
import UpdateDialog from '@/components/about/UpdateDialog.vue'
import { useSdkStore } from '@/stores/sdk'
import { useAuthStore } from '@/stores/auth'
import { useJavaStore } from '@/stores/java'
import { useSettingsStore } from '@/stores/settings'
import { setModalRef } from '@/utils/modal'
import { setCrashDialogRef } from '@/utils/crashDialog'
import { setToastRef } from '@/utils/toast'
import { initAutoCheck } from '@/utils/updater'
import { initDownloadPolling } from '@/composables/useDownloadPolling'
import { useDragDrop } from '@/composables/useDragDrop'

const sdkStore = useSdkStore()
const authStore = useAuthStore()
const javaStore = useJavaStore()
const settingsStore = useSettingsStore()
const modalRef = ref<InstanceType<typeof Modal> | null>(null)
const crashDialogRef = ref<InstanceType<typeof CrashDialog> | null>(null)
const toastRef = ref<InstanceType<typeof Toast> | null>(null)
// 会话恢复期间的加载遮罩，避免未恢复登录态就触发其他页面 invoke
const isRestoring = ref(true)

// 注册全局拖拽事件监听（必须在 onMounted 中调用，onUnmounted 自动清理）
useDragDrop()

onMounted(() => {
  console.log('[Startup][Frontend] App.vue onMounted @', new Date().toISOString())
  setModalRef(modalRef.value)
  setCrashDialogRef(crashDialogRef.value)
  setToastRef(toastRef.value)
  initDownloadPolling()
  // 启动自动更新检查（启动后 5s + 每 6 小时；dev 模式自动跳过）
  initAutoCheck()
  initApp()
})

async function initApp() {
  console.log('[Startup][Frontend] initApp start @', new Date().toISOString())
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
  console.log('[Startup][Frontend] initApp fully done @', new Date().toISOString())
}
</script>

<template>
  <TopNavLayout>
    <router-view v-slot="{ Component }">
      <transition name="fade" mode="out-in">
        <component :is="Component" />
      </transition>
    </router-view>
  </TopNavLayout>
  <Teleport to="body">
    <BackToTop />
    <DownloadPanel />
  </Teleport>
  <Modal ref="modalRef" />
  <CrashDialog ref="crashDialogRef" />
  <Toast ref="toastRef" />
  <UpdateDialog />
  <DragOverlay />
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
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
