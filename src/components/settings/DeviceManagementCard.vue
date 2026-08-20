<script setup lang="ts">
/**
 * 设置-联机 - 设备管理卡片（登出 / 清除凭证）
 */
import { defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
import { useOnlineStore } from '@/stores/online'
import { showConfirm } from '@/utils/modal'
import { ArrowRightOnRectangleIcon, TrashIcon } from '@heroicons/vue/24/outline'

const onlineStore = useOnlineStore()

async function handleLogout() {
  await onlineStore.logout()
}

function handleClearCredentials() {
  showConfirm(
    '清除设备凭证',
    '此操作将删除本地密钥与 JWT，需要重新注册设备才能继续使用联机功能。\n是否继续？',
    async () => {
      await onlineStore.clear()
    },
  )
}
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">设备管理</h3>
    <div class="divide-y divide-gray-200">
      <!-- 登出 -->
      <div class="px-5 py-4 flex items-center justify-between gap-4">
        <div class="min-w-0">
          <p class="text-sm font-medium text-gray-900">登出设备</p>
          <p class="text-xs text-gray-500 mt-0.5">撤销当前 JWT，保留本地密钥，可重新登录</p>
        </div>
        <Button
          type="outline"
          size="small"
          :loading="onlineStore.loading"
          :disabled="!onlineStore.deviceStatus?.logged_in"
          @click="handleLogout"
        >
          <template #icon><ArrowRightOnRectangleIcon class="w-4 h-4" /></template>
          登出
        </Button>
      </div>
      <!-- 清除凭证 -->
      <div class="px-5 py-4 flex items-center justify-between gap-4">
        <div class="min-w-0">
          <p class="text-sm font-medium text-gray-900">清除设备凭证</p>
          <p class="text-xs text-gray-500 mt-0.5">删除本地密钥与 JWT，需要重新注册设备</p>
        </div>
        <Button
          type="outline"
          size="small"
          :loading="onlineStore.loading"
          :disabled="!onlineStore.deviceStatus?.registered"
          @click="handleClearCredentials"
        >
          <template #icon><TrashIcon class="w-4 h-4" /></template>
          清除
        </Button>
      </div>
    </div>
  </div>
</template>