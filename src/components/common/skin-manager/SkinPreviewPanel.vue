<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
/**
 * 皮肤管理 - 3D 预览面板（从 SkinManager.vue 抽出）
 *
 * 显示当前账号的 3D 人物模型、头像、用户名、皮肤模型、披风/皮肤名称，
 * 以及下载当前皮肤 PNG 到本地的按钮。
 */
const SkinAvatar = defineAsyncComponent(() => import('../SkinAvatar.vue'))
const SkinModel3D = defineAsyncComponent(() => import('../SkinModel3D.vue'))
const Tooltip = defineAsyncComponent(() => import('../Tooltip.vue'))
import type { AnimationType } from './SkinAnimationSelector.vue'
import type { CapeInfo } from '@/utils/tauri'

interface Props {
  /** 皮肤 PNG URL */
  skinUrl: string | null
  /** 披风 PNG URL */
  capeUrl: string | null
  /** 皮肤模型：classic Steve | slim Alex */
  variant: 'classic' | 'slim'
  /** 动画状态 */
  animation: AnimationType
  /** 当前账号 UUID */
  uuid: string
  /** 当前账号用户名 */
  username: string
  /** 是否微软账号（决定登录类型显示与披风/本地皮肤文本切换） */
  isMicrosoft: boolean
  /** 当前已装备的披风（微软账号显示） */
  activeCape: CapeInfo | null
  /** 当前选中的本地皮肤名称（离线账号显示） */
  selectedLocalSkin: string | null
}

defineProps<Props>()
defineEmits<{ save: [] }>()
</script>

<template>
  <div class="rounded-lg border border-gray-100 bg-gray-50/50 p-4">
    <div class="mb-3 flex items-center justify-between">
      <div class="text-sm font-medium text-gray-700">当前形象</div>
      <div class="text-[10px] text-gray-400">拖动旋转</div>
    </div>
    <!-- 3D 人物模型（skinview3d 渲染，皮肤 + 披风） -->
    <div class="flex justify-center rounded-md bg-white p-2 shadow-sm">
      <SkinModel3D
        :skin-url="skinUrl"
        :cape-url="capeUrl"
        :variant="variant"
        :height="280"
        :animation="animation"
      />
    </div>
    <div class="mt-3 flex items-center gap-3">
      <SkinAvatar :skin-url="skinUrl ?? undefined" :uuid="uuid" :username="username" :size="40" :overlay="true" :login-type="isMicrosoft ? 'Microsoft' : 'Offline'" />
      <div class="flex-1 space-y-1 text-xs text-gray-500">
        <div>用户名：{{ username }}</div>
        <div>皮肤模型：{{ variant === 'slim' ? 'Alex（纤细）' : 'Steve（经典）' }}</div>
        <div v-if="isMicrosoft">当前披风：{{ activeCape?.display_name ?? '未装备' }}</div>
        <div v-else>当前皮肤：{{ selectedLocalSkin ?? '默认' }}</div>
      </div>
      <!-- 下载当前皮肤按钮 -->
      <Tooltip text="下载当前皮肤 PNG 到本地" position="top" :delay="0">
        <!-- 保留原生 button：下载图标按钮（h-7 w-7 + border），
             Button.vue 的 scoped size 类固定 height/padding 会使图标按钮过宽 -->
        <button
          class="flex-none flex h-7 w-7 items-center justify-center rounded border border-gray-200 text-gray-600 transition-colors hover:bg-gray-50 disabled:opacity-40"
          :disabled="!skinUrl"
          @click="$emit('save')"
        >
          <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
            <path d="M10 3a1 1 0 011 1v6.586l2.293-2.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 111.414-1.414L9 10.586V4a1 1 0 011-1z" />
            <path d="M3 14a1 1 0 011 1v1h12v-1a1 1 0 112 0v2a1 1 0 01-1 1H3a1 1 0 01-1-1v-2a1 1 0 011-1z" />
          </svg>
        </button>
      </Tooltip>
    </div>
  </div>
</template>
