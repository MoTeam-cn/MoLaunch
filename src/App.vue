<script setup lang="ts">
/**
 * MoLaunch 根组件
 */

import { onMounted, ref } from 'vue'
import TopNavLayout from '@/components/layout/TopNavLayout.vue'
import BackToTop from '@/components/common/BackToTop.vue'
import Modal from '@/components/common/Modal.vue'
import { useSdkStore } from '@/stores/sdk'
import { useAuthStore } from '@/stores/auth'
import { setModalRef } from '@/utils/modal'

const sdkStore = useSdkStore()
const authStore = useAuthStore()
const modalRef = ref<InstanceType<typeof Modal> | null>(null)

onMounted(() => {
  // 设置弹窗引用
  setModalRef(modalRef.value)
  
  // 初始化
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
  <Modal ref="modalRef" />
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
