<script setup lang="ts">
/**
 * 微软账号皮肤上传 + 账号管理快捷入口
 *
 * - 上传新皮肤（选择文件 + 皮肤模型 Steve/Alex 切换）
 * - 修改密码 / 修改用户名（打开外部网页）
 * - emit upload，业务逻辑由父组件处理
 */
import { open } from '@tauri-apps/plugin-shell'
import { showError } from '@/utils/toast'

const props = defineProps<{
  variant: 'classic' | 'slim'
  uploading: boolean
}>()
const emit = defineEmits<{
  'update:variant': ['classic' | 'slim']
  upload: []
}>()

function openChangePassword() {
  open('https://account.live.com/password/Change').catch(() => showError('打开网页失败'))
}

function openChangeUsername() {
  open('https://www.minecraft.net/zh-hans/msaprofile/mygames/editprofile').catch(() => showError('打开网页失败'))
}
</script>

<template>
  <!-- 上传新皮肤 -->
  <div class="rounded-lg border border-gray-100 p-4">
    <div class="mb-3 text-sm font-medium text-gray-700">上传新皮肤</div>
    <div class="mb-3 text-xs text-gray-500">
      支持 64x64 或 64x32 PNG<br/>
      文件需小于 24KB（Mojang 限制）
    </div>
    <div class="mb-3">
      <label class="mb-1 block text-xs text-gray-500">皮肤模型</label>
      <div class="flex gap-2">
        <button
          class="flex-1 rounded-md border px-3 py-1.5 text-xs transition-colors"
          :class="props.variant === 'classic' ? 'border-primary-500 bg-primary-50 text-primary-700' : 'border-gray-200 text-gray-600 hover:bg-gray-50'"
          @click="emit('update:variant', 'classic')"
        >Steve（经典）</button>
        <button
          class="flex-1 rounded-md border px-3 py-1.5 text-xs transition-colors"
          :class="props.variant === 'slim' ? 'border-primary-500 bg-primary-50 text-primary-700' : 'border-gray-200 text-gray-600 hover:bg-gray-50'"
          @click="emit('update:variant', 'slim')"
        >Alex（纤细）</button>
      </div>
    </div>
    <button
      class="w-full rounded-md bg-primary-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-700 disabled:opacity-50"
      :disabled="uploading"
      @click="emit('upload')"
    >
      {{ uploading ? '处理中...' : '选择文件并上传' }}
    </button>
  </div>

  <!-- 账号管理快捷入口 -->
  <div class="rounded-lg border border-gray-100 p-4">
    <div class="mb-3 text-sm font-medium text-gray-700">账号管理</div>
    <div class="space-y-2">
      <button
        class="flex w-full items-center justify-between rounded-md border border-gray-200 px-3 py-2 text-xs text-gray-700 transition-colors hover:bg-gray-50"
        @click="openChangePassword"
      >
        <span>修改密码</span>
        <svg class="h-3.5 w-3.5 text-gray-400" viewBox="0 0 20 20" fill="currentColor"><path d="M11 3a1 1 0 100 2h2.586l-6.293 6.293a1 1 0 101.414 1.414L15 6.414V9a1 1 0 102 0V4a1 1 0 00-1-1h-5z" /><path d="M5 5a2 2 0 00-2 2v8a2 2 0 002 2h8a2 2 0 002-2v-3a1 1 0 10-2 0v3H5V7h3a1 1 0 000-2H5z" /></svg>
      </button>
      <button
        class="flex w-full items-center justify-between rounded-md border border-gray-200 px-3 py-2 text-xs text-gray-700 transition-colors hover:bg-gray-50"
        @click="openChangeUsername"
      >
        <span>修改用户名（每30天一次）</span>
        <svg class="h-3.5 w-3.5 text-gray-400" viewBox="0 0 20 20" fill="currentColor"><path d="M11 3a1 1 0 100 2h2.586l-6.293 6.293a1 1 0 101.414 1.414L15 6.414V9a1 1 0 102 0V4a1 1 0 00-1-1h-5z" /><path d="M5 5a2 2 0 00-2 2v8a2 2 0 002 2h8a2 2 0 002-2v-3a1 1 0 10-2 0v3H5V7h3a1 1 0 000-2H5z" /></svg>
      </button>
    </div>
  </div>
</template>
