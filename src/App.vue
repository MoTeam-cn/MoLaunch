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

onMounted(() => {
  setModalRef(modalRef.value)
  setToastRef(toastRef.value)
  // 全局暴露，供 Pinia store 等非组件代码使用
  ;(window as any).__toastRef = toastRef.value
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
