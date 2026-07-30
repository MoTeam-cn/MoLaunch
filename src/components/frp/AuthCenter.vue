<script setup lang="ts">
/**
 * 认证中心（占位）
 *
 * 阶段三完整实现：管理厂商 OAuth / Device Code 认证、token OS 密钥存储、
 * 认证状态查询、刷新 token、撤销授权等。
 *
 * 阶段二仅显示占位 UI，避免侧边栏菜单点击后空白。
 */
import { ShieldCheckIcon } from '@heroicons/vue/24/outline'
</script>

<template>
  <div class="flex flex-col items-center justify-center py-16">
    <ShieldCheckIcon class="w-12 h-12 text-gray-300 mb-3" />
    <p class="text-sm font-medium text-gray-500">认证中心</p>
    <p class="text-xs text-gray-400 mt-1">此功能将在阶段三上线，敬请期待</p>
  </div>
</template>
