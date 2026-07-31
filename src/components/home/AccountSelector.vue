<script setup lang="ts">
/**
 * 账号选择器（单卡片展示 + 左右滑动切换动画）
 *
 * - 一次只展示一个账号卡片（大头像 + 用户名 + 类型）
 * - 左右箭头 / 拖动 / 滚轮 切换账号，带平滑滑动动画
 * - 末尾有"添加账号"卡片
 * - 离线账号显示默认皮肤（Steve/Alex）
 *
 * 子组件：
 *   - LoginPrompt         未登录时的登录引导
 *   - AccountIndicator    圆点指示器 + 计数
 *   - AccountCard         单个账号卡片（头像/用户名/操作按钮）
 *   - useSwipeNavigation  拖动/滚轮导航 composable
 *   - useAccountCards     账号列表构建 + 切换/删除/登出 composable
 */

import { ref } from 'vue'
import { useRouter } from 'vue-router'
import SkinManager from '@/components/common/SkinManager.vue'
import AccountCard, { type AccountCardData } from './account-selector/AccountCard.vue'
import AccountIndicator from './account-selector/AccountIndicator.vue'
import LoginPrompt from './account-selector/LoginPrompt.vue'
import { useSwipeNavigation } from '@/composables/useSwipeNavigation'
import { useAccountCards } from '@/composables/useAccountCards'
import { toastError } from '@/utils/toast'

const router = useRouter()

const showSkinManager = ref(false)

const {
  cards,
  currentIndex,
  hasAddCard,
  totalCards,
  isLoggedIn,
  currentUsername,
  currentLoginType,
  switchTo,
  prev,
  next,
  switchAccount,
  removeAccount,
  logout,
} = useAccountCards()

/** 点击卡片上的"皮肤"按钮：非当前账号先切换再打开 */
async function onCardSkin(card: AccountCardData) {
  if (!card.isActive) {
    await switchAccount(card.uuid, card.loginType)
  }
  showSkinManager.value = true
}

/** 点击卡片上的"登出/删除"按钮 */
async function onCardLogout(card: AccountCardData, event: Event) {
  if (card.isActive) {
    await logout()
  } else {
    await removeAccount(card.uuid, card.loginType, event)
  }
}

/**
 * 跳转到登录页添加新账号
 *
 * 关键：query 带 add=1，路由守卫据此放行已登录用户进入 /login
 * （否则守卫会把已登录用户重定向回 /apps，表现为"点击没反应"）
 *
 * 添加错误捕获：Vue Router 4 中 router.push 若被守卫拒绝会静默 resolve，
 * 这里手动捕获 NavigationFailure 并打印，便于排查"点击没反应"类问题。
 */
async function addAccount() {
  try {
    await router.push({ path: '/login', query: { add: '1' } })
  } catch (err) {
    console.error('[AccountSelector] 跳转登录页失败:', err)
    toastError('跳转登录页失败')
  }
}

// 拖动/滚轮导航（onSwitch 回调即 switchTo，switchTo 内部自带 switching 检查）
const {
  isDragging, dragMoved, isAnimating, cardTransform,
  onPointerDown, onPointerMove, onPointerUp, onWheel,
} = useSwipeNavigation(totalCards, currentIndex, switchTo)
</script>

<template>
  <div class="account-card-container flex h-full w-full min-w-0 flex-col overflow-hidden">
    <!-- 未登录状态 -->
    <LoginPrompt v-if="!isLoggedIn" />

    <!-- 已登录状态：单卡片 + 左右切换 -->
    <template v-else>
      <!-- 卡片栏标题 + 指示器 -->
      <AccountIndicator
        :cards="cards"
        :current-index="currentIndex"
        :has-add-card="hasAddCard"
        :total-cards="totalCards"
        @switch="switchTo"
      />

      <!-- 卡片容器（带左右切换按钮） -->
      <!-- 保留原生 button：左右箭头为图标按钮（p-1 紧凑尺寸），Button.vue 的 scoped
           size 类固定 padding 0 15px 会使图标按钮过宽 -->
      <div class="flex items-center gap-1">
        <!-- 左箭头 -->
        <button
          class="flex-none rounded-md p-1 text-gray-400 transition-colors hover:bg-gray-100 hover:text-primary-500 disabled:opacity-30 disabled:hover:bg-transparent"
          :disabled="currentIndex === 0"
          @click="prev"
        >
          <svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M12.7 4.3a1 1 0 010 1.4L8.4 10l4.3 4.3a1 1 0 01-1.4 1.4l-5-5a1 1 0 010-1.4l5-5a1 1 0 011.4 0z" clip-rule="evenodd" />
          </svg>
        </button>

        <!-- 卡片滑动区 -->
        <div
          class="relative min-w-0 flex-1 overflow-hidden rounded-xl"
          :class="{ 'cursor-grab': !isDragging, 'cursor-grabbing': isDragging }"
          @pointerdown="onPointerDown"
          @pointermove="onPointerMove"
          @pointerup="onPointerUp"
          @pointercancel="onPointerUp"
          @wheel="onWheel"
        >
          <div
            class="flex will-change-transform"
            :class="[
              (isDragging && dragMoved) ? 'transition-none' : 'transition-transform duration-300 ease-out',
              isDragging && dragMoved ? 'select-none' : '',
            ]"
            :style="{ transform: cardTransform }"
          >
            <!-- 账号卡片 -->
            <AccountCard
              v-for="card in cards"
              :key="card.uuid"
              :card="card"
              @skin="onCardSkin"
              @logout="onCardLogout"
            />

            <!-- 添加账号卡片 -->
            <div v-if="hasAddCard" class="flex-none" style="width: 100%;">
              <!-- 保留原生 button：卡片式 CTA 使用 flex-col 列布局 + 虚线边框 + min-height，
                   Button.vue 为行内 flex 且 svg 有 margin，不适合卡片列布局 -->
              <button
                class="flex h-full w-full flex-col items-center justify-center rounded-xl border-2 border-dashed border-gray-200 p-4 text-gray-400 transition-all hover:border-primary-400 hover:bg-primary-50/30 hover:text-primary-500"
                style="min-height: 200px;"
                @click="addAccount"
              >
                <svg class="mb-3 h-12 w-12" viewBox="0 0 20 20" fill="currentColor">
                  <path d="M10 5a1 1 0 011 1v3h3a1 1 0 110 2h-3v3a1 1 0 11-2 0v-3H6a1 1 0 110-2h3V6a1 1 0 011-1z" />
                </svg>
                <div class="text-base font-medium">添加账号</div>
                <div class="mt-1 text-xs text-gray-300">登录新的微软或离线账号</div>
              </button>
            </div>
          </div>
        </div>

        <!-- 右箭头 -->
        <button
          class="flex-none rounded-md p-1 text-gray-400 transition-colors hover:bg-gray-100 hover:text-primary-500 disabled:opacity-30 disabled:hover:bg-transparent"
          :disabled="currentIndex >= totalCards - 1"
          @click="next"
        >
          <svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M7.3 4.3a1 1 0 011.4 0l5 5a1 1 0 010 1.4l-5 5a1 1 0 01-1.4-1.4L11.6 10 7.3 5.7a1 1 0 010-1.4z" clip-rule="evenodd" />
          </svg>
        </button>
      </div>

      <!-- 当前账号信息 -->
      <div class="mt-3 rounded-lg bg-gray-50 px-3 py-2 text-xs text-gray-500">
        <div class="flex items-center justify-between">
          <span>当前账号</span>
          <span class="font-medium text-gray-700">{{ currentUsername }}</span>
        </div>
        <div class="mt-1 flex items-center justify-between">
          <span>账号类型</span>
          <span>{{ currentLoginType }}</span>
        </div>
      </div>
    </template>

    <!-- 皮肤管理弹窗 -->
    <SkinManager v-model:visible="showSkinManager" />
  </div>
</template>
