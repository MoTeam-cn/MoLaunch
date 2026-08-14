<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
/**
 * 账号选择器未登录状态
 * - 显示用户图标 + 「立即登录」按钮
 * - 点击按钮跳转到登录页
 */
import { useRouter } from 'vue-router'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))

const router = useRouter()
function login() { router.push('/login') }
</script>

<template>
  <div class="flex flex-col items-center py-8">
    <svg class="mb-5 h-12 w-12 text-gray-300" viewBox="0 0 24 24" fill="currentColor">
      <path d="M12 12a5 5 0 100-10 5 5 0 000 10zm0 2c-5 0-9 3-9 7v1h18v-1c0-4-4-7-9-7z" />
    </svg>
    <Button type="primary" @click="login">
      立即登录
    </Button>
    <div class="mt-4 flex gap-4 text-xs text-gray-300">
      <a class="cursor-pointer transition-colors hover:text-primary-500" @click="login">登录账号</a>
    </div>
  </div>
</template>
