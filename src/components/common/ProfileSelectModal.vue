<script setup lang="ts">
/**
 * authlib 多角色选择弹窗
 *
 * 当 `authlib_login` 返回 `NeedSelect`（账号有多个角色且服务器无 selected_profile）时，
 * 前端用此弹窗让用户选择一个 profile，再调用 `authlib_select_profile` 完成登录。
 *
 * 与 DeviceCodeModal.vue 风格一致：Teleport 到 body，fade 过渡。
 */

import type { AuthlibProfile } from '@/types/auth'
import Button from '@/components/common/Button.vue'

defineProps<{
  visible: boolean
  profiles: AuthlibProfile[]
  loading?: boolean
}>()
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'select', profile: AuthlibProfile): void
}>()

function handleSelect(profile: AuthlibProfile) {
  emit('select', profile)
}

function handleClose() {
  emit('close')
}
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div
        v-if="visible"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
        @click.self="handleClose"
      >
        <div class="mx-4 w-full max-w-md rounded-2xl bg-white p-6 shadow-xl">
          <!-- 标题 -->
          <div class="mb-4 flex items-center gap-3">
            <svg class="h-6 w-6 text-primary-500" viewBox="0 0 20 20" fill="currentColor">
              <path d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" />
            </svg>
            <h3 class="text-lg font-semibold text-gray-900">选择角色</h3>
          </div>

          <p class="mb-4 text-sm text-gray-600">检测到此账号有多个角色，请选择要使用的角色：</p>

          <!-- 角色列表 -->
          <div class="space-y-2">
            <button
              v-for="profile in profiles"
              :key="profile.id"
              type="button"
              class="flex w-full items-center gap-3 rounded-lg border border-gray-200 px-4 py-3 text-left transition-all hover:border-primary-400 hover:bg-primary-50/30"
              :disabled="loading"
              @click="handleSelect(profile)"
            >
              <div class="flex h-8 w-8 flex-none items-center justify-center rounded-full bg-primary-100 text-sm font-medium text-primary-600">
                {{ profile.name.charAt(0).toUpperCase() }}
              </div>
              <div class="min-w-0 flex-1">
                <div class="truncate text-sm font-medium text-gray-900">{{ profile.name }}</div>
                <div class="truncate text-xs text-gray-400">{{ profile.id }}</div>
              </div>
              <svg class="h-4 w-4 flex-none text-gray-300" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M7.3 4.3a1 1 0 011.4 0l5 5a1 1 0 010 1.4l-5 5a1 1 0 01-1.4-1.4L11.6 10 7.3 5.7a1 1 0 010-1.4z" clip-rule="evenodd" />
              </svg>
            </button>
          </div>

          <!-- 底部按钮 -->
          <div class="mt-4 flex justify-end">
            <Button type="text" :disabled="loading" @click="handleClose">取消</Button>
          </div>

          <!-- 加载遮罩 -->
          <div v-if="loading" class="mt-3 flex items-center justify-center gap-2 text-sm text-primary-600">
            <div class="h-4 w-4 animate-spin rounded-full border-2 border-primary-200 border-t-primary-500" />
            <span>正在完成登录...</span>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
