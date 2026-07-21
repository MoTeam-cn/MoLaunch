<script setup lang="ts">
/**
 * MoLaunch 根组件
 */

import { onMounted, ref } from 'vue'
import router from '@/router'
import TopNavLayout from '@/components/layout/TopNavLayout.vue'
import BackToTop from '@/components/common/BackToTop.vue'
import DownloadPanel from '@/components/common/DownloadPanel.vue'
import Modal from '@/components/common/Modal.vue'
import CrashDialog from '@/components/common/CrashDialog.vue'
import Toast from '@/components/common/Toast.vue'
import { useSdkStore } from '@/stores/sdk'
import { useAuthStore } from '@/stores/auth'
import { useJavaStore } from '@/stores/java'
import { setModalRef } from '@/utils/modal'
import { setCrashDialogRef } from '@/utils/crashDialog'
import { setToastRef } from '@/utils/toast'
import { initDownloadPolling } from '@/composables/useDownloadPolling'

const sdkStore = useSdkStore()
const authStore = useAuthStore()
const javaStore = useJavaStore()
const modalRef = ref<InstanceType<typeof Modal> | null>(null)
const crashDialogRef = ref<InstanceType<typeof CrashDialog> | null>(null)
const toastRef = ref<InstanceType<typeof Toast> | null>(null)
// 会话恢复期间的加载遮罩，避免未恢复登录态就触发其他页面 invoke
const isRestoring = ref(true)

onMounted(() => {
  setModalRef(modalRef.value)
  setCrashDialogRef(crashDialogRef.value)
  setToastRef(toastRef.value)
  initDownloadPolling()
  initApp()
})

async function initApp() {
  // Java 搜索不依赖 SDK/认证，提前并行启动（后端 list_java 是耗时操作）
  const javaPromise = javaStore.detectJava()

  // 平台信息和设备 ID 互不依赖，并行获取
  await Promise.all([
    sdkStore.fetchPlatformInfo(),
    sdkStore.fetchDeviceId(),
  ])

  // 恢复登录状态
  await authStore.restoreSession()

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
