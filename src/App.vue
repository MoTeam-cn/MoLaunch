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
import { setModalRef } from '@/utils/modal'
import { setToastRef } from '@/utils/toast'

const sdkStore = useSdkStore()
const authStore = useAuthStore()
const modalRef = ref<InstanceType<typeof Modal> | null>(null)
const toastRef = ref<InstanceType<typeof Toast> | null>(null)

onMounted(() => {
  setModalRef(modalRef.value)
  setToastRef(toastRef.value)
  initApp()
})

async function initApp() {
  // 获取平台信息
  await sdkStore.fetchPlatformInfo()
  
  // 检查 SDK 状态
  await sdkStore.checkStatus()
  
  // 如果 SDK 未初始化，尝试初始化
  if (!sdkStore.initialized) {
    try {
      await sdkStore.initialize()
    } catch (e) {
      console.error('Failed to initialize SDK:', e)
    }
  }
  
  // 恢复登录状态
  await authStore.restoreSession()

  // 后台加载 Java 信息
  sdkStore.loadJava()
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
  <BackToTop />
  <DownloadPanel />
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
