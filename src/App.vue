<script setup lang="ts">
/**
 * MoLaunch 根组件
 */

import { onMounted } from 'vue'
import TopNavLayout from '@/components/layout/TopNavLayout.vue'
import BackToTop from '@/components/common/BackToTop.vue'
import { useSdkStore } from '@/stores/sdk'
import { useAuthStore } from '@/stores/auth'

const sdkStore = useSdkStore()
const authStore = useAuthStore()

onMounted(async () => {
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
})
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
