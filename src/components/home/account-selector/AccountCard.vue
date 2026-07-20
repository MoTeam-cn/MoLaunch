<script setup lang="ts">
/**
 * 单个账号卡片（轮播中的一张）
 * - 头像 + 用户名 + 类型/状态 + 皮肤/登出按钮
 * - 纯展示组件，所有操作通过 emit 上抛由父组件处理
 */
import SkinAvatar from '@/components/common/SkinAvatar.vue'
import Button from '@/components/common/Button.vue'

export interface AccountCardData {
  uuid: string
  username: string
  loginType: string  // '正版' | '离线'
  isExpired?: boolean
  isActive?: boolean
}

defineProps<{ card: AccountCardData }>()

const emit = defineEmits<{
  skin: [card: AccountCardData]
  logout: [card: AccountCardData, event: Event]
}>()
</script>

<template>
  <div class="flex-none" style="width: 100%;">
    <div
      class="group relative rounded-xl border-2 p-4 transition-all"
      :class="card.isActive
        ? 'border-primary-500 bg-primary-50'
        : 'border-gray-200 bg-white hover:border-primary-300'"
    >
      <!-- 头像（PCL2 双层立体头像，离线账号显示默认皮肤） -->
      <div class="mb-3 flex justify-center">
        <SkinAvatar
          :uuid="card.uuid"
          :username="card.username"
          :size="96"
          :overlay="true"
          :rounded="false"
          :login-type="card.loginType === '正版' ? 'Microsoft' : 'Offline'"
        />
      </div>

      <!-- 用户名 -->
      <div class="truncate text-center text-base font-medium" :class="card.isActive ? 'text-primary-700' : 'text-gray-800'">
        {{ card.username }}
      </div>

      <!-- 账号类型 + 状态 -->
      <div class="mt-1 flex items-center justify-center gap-1.5 text-xs">
        <span :class="card.isActive ? 'text-primary-500' : 'text-gray-400'">{{ card.loginType }}</span>
        <span v-if="card.isActive" class="rounded-full bg-primary-100 px-2 py-0.5 text-primary-600">当前</span>
        <span v-else-if="card.isExpired" class="rounded-full bg-yellow-100 px-2 py-0.5 text-yellow-600">过期</span>
        <span v-else class="text-gray-300">点击切换</span>
      </div>

      <!-- 常驻操作按钮组（所有账号卡片） -->
      <div class="mt-3 flex flex-wrap justify-center gap-1.5">
        <Button
          type="outline"
          size="mini"
          :title="card.loginType === '正版' ? '皮肤与披风管理' : '本地皮肤选择'"
          @click.stop="emit('skin', card)"
        >
          <template #icon>
            <svg class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
              <path d="M10 2a8 8 0 100 16 8 8 0 000-16zm0 2a6 6 0 110 12 6 6 0 010-12z" />
            </svg>
          </template>
          皮肤
        </Button>
        <button
          class="flex items-center gap-1 rounded-md border border-gray-200 px-2.5 py-1 text-xs text-red-500 transition-colors hover:border-red-300 hover:bg-red-50"
          :title="card.isActive ? '退出登录' : '删除此账号'"
          @click.stop="emit('logout', card, $event)"
        >
          <svg class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
            <path v-if="card.isActive" fill-rule="evenodd" d="M3 4a1 1 0 011-1h7a1 1 0 110 2H5v10h6a1 1 0 110 2H4a1 1 0 01-1-1V4zm11.3 3.3a1 1 0 011.4 0l3 3a1 1 0 010 1.4l-3 3a1 1 0 01-1.4-1.4l1.3-1.3H9a1 1 0 110-2h6.6l-1.3-1.3a1 1 0 010-1.4z" clip-rule="evenodd" />
            <path v-else fill-rule="evenodd" d="M4.3 4.3a1 1 0 011.4 0L10 8.6l4.3-4.3a1 1 0 111.4 1.4L11.4 10l4.3 4.3a1 1 0 01-1.4 1.4L10 11.4l-4.3 4.3a1 1 0 01-1.4-1.4L8.6 10 4.3 5.7a1 1 0 010-1.4z" clip-rule="evenodd" />
          </svg>
          {{ card.isActive ? '登出' : '删除' }}
        </button>
      </div>
    </div>
  </div>
</template>
