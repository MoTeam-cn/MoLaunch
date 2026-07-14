<script setup lang="ts">
/**
 * MoLaunch 根组件
 */

import { onMounted, ref } from 'vue'
import TopNavLayout from '@/components/layout/TopNavLayout.vue'
import BackToTop from '@/components/common/BackToTop.vue'
import DownloadPanel from '@/components/common/DownloadPanel.vue'
import Modal from '@/components/common/Modal.vue'
import Toast from '@/components/common/Toast.vue'
import { useSdkStore } from '@/stores/sdk'
import { useAuthStore } from '@/stores/auth'
import { useJavaStore } from '@/stores/java'
import { setModalRef } from '@/utils/modal'
import { setToastRef } from '@/utils/toast'
import { initDownloadPolling } from '@/composables/useDownloadPolling'

const sdkStore = useSdkStore()
const authStore = useAuthStore()
const javaStore = useJavaStore()
const modalRef = ref<InstanceType<typeof Modal> | null>(null)
const toastRef = ref<InstanceType<typeof Toast> | null>(null)
// 会话恢复期间的加载遮罩，避免未恢复登录态就触发其他页面 invoke
const isRestoring = ref(true)

onMounted(() => {
  setModalRef(modalRef.value)
  setToastRef(toastRef.value)
  initDownloadPolling()
  initApp()
})

async function initApp() {
  // 获取平台信息
  await sdkStore.fetchPlatformInfo()

  // 获取设备ID
  await sdkStore.fetchDeviceId()

  // 恢复登录状态
  await authStore.restoreSession()

  // 后台加载 Java 信息
  javaStore.detectJava()

  // 登录态已恢复，关闭加载遮罩
  isRestoring.value = false
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
