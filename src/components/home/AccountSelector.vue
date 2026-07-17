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
 */

import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { showWarning } from '@/utils/toast'
import SkinManager from '@/components/common/SkinManager.vue'
import AccountCard, { type AccountCardData } from './account-selector/AccountCard.vue'
import AccountIndicator from './account-selector/AccountIndicator.vue'
import LoginPrompt from './account-selector/LoginPrompt.vue'
import { useSwipeNavigation } from '@/composables/useSwipeNavigation'

const router = useRouter()
const authStore = useAuthStore()

const showSkinManager = ref(false)

/** 当前显示的卡片索引 */
const currentIndex = ref(0)

/**
 * 账号卡片列表（微软账号 + 离线账号，顺序稳定）
 *
 * 关键：cards 的顺序不随 currentUser 变化而重排，
 * 当前账号通过 isActive=true 标记，这样切换账号时 currentIndex 指向稳定不变。
 */
const cards = computed<AccountCardData[]>(() => {
  const list: AccountCardData[] = []
  const currentUuid = authStore.currentUser?.uuid

  // 微软账号
  for (const acc of authStore.msAccounts) {
    list.push({
      uuid: acc.uuid,
      username: acc.username,
      loginType: '正版',
      isExpired: acc.is_expired,
      isActive: acc.uuid === currentUuid,
    })
  }
  // 离线账号
  for (const acc of authStore.offlineAccounts) {
    list.push({
      uuid: acc.uuid,
      username: acc.username,
      loginType: '离线',
      isActive: acc.uuid === currentUuid,
    })
  }
  // 如果当前账号不在任何列表里（理论上不应发生），追加到末尾
  if (authStore.currentUser && !list.some(c => c.uuid === currentUuid)) {
    list.push({
      uuid: authStore.currentUser.uuid,
      username: authStore.currentUser.name,
      loginType: authStore.currentUser.login_type === 'Microsoft' ? '正版' : '离线',
      isActive: true,
    })
  }
  return list
})

/** 是否有"添加账号"卡片（末尾） */
const hasAddCard = computed(() => cards.value.length > 0)
/** 总卡片数（含添加卡片） */
const totalCards = computed(() => cards.value.length + (hasAddCard.value ? 1 : 0))

const isLoggedIn = computed(() => authStore.isLoggedIn)
const currentUsername = computed(() => authStore.currentUser?.name ?? '')
const currentLoginType = computed(() => {
  if (!authStore.currentUser) return ''
  return authStore.currentUser.login_type === 'Microsoft' ? '正版账号' : '离线账号'
})

/**
 * 确保 currentIndex 不越界，并定位到当前活跃账号。
 *
 * 首次加载或账号列表变化时，把 currentIndex 移到 active 卡片。
 * 切换账号时 cards 顺序稳定，currentIndex 不变。
 */
watch(cards, (newCards) => {
  const total = newCards.length + (hasAddCard.value ? 1 : 0)
  if (currentIndex.value >= total) {
    currentIndex.value = Math.max(0, total - 1)
  }
  // 如果当前索引不是 active 卡片，且存在 active 卡片，移过去
  const currentCard = newCards[currentIndex.value]
  const activeIndex = newCards.findIndex(c => c.isActive)
  if (activeIndex >= 0 && !currentCard?.isActive) {
    currentIndex.value = activeIndex
  }
})

/** 正在切换账号的锁，防止快速滑动时并发请求导致后端报错 */
const switching = ref(false)

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

/** 切换到指定索引（带边界检查 + 切换锁） */
function switchTo(index: number) {
  if (index < 0 || index >= totalCards.value) return
  if (switching.value) return  // 正在切换中，忽略
  currentIndex.value = index
  // 如果切换到了一个非当前账号卡片，触发账号切换
  const card = cards.value[index]
  if (card && !card.isActive) {
    switchAccount(card.uuid, card.loginType)
  }
}

function prev() { if (currentIndex.value > 0) switchTo(currentIndex.value - 1) }
function next() { if (currentIndex.value < totalCards.value - 1) switchTo(currentIndex.value + 1) }

async function switchAccount(targetUuid: string, loginType: string) {
  if (authStore.currentUser?.uuid === targetUuid) return
  if (switching.value) return  // 正在切换中，忽略
  switching.value = true
  try {
    if (loginType === '正版') {
      await authStore.switchMsAccount(targetUuid)
    } else {
      await authStore.switchOfflineAccount(targetUuid)
    }
    // 切换账号不改变皮肤数据，无需 bumpSkinVersion（皮肤变更由 SkinManager 负责）
  } catch (e) {
    showWarning(String(e))
    // 失败时回滚 currentIndex 到实际当前账号
    const activeIndex = cards.value.findIndex(c => c.isActive)
    if (activeIndex >= 0) currentIndex.value = activeIndex
  } finally {
    switching.value = false
  }
}

async function removeAccount(targetUuid: string, loginType: string, event: Event) {
  event.stopPropagation()
  try {
    if (loginType === '正版') {
      await authStore.removeMsAccount(targetUuid)
    } else {
      await authStore.removeOfflineAccount(targetUuid)
    }
    // 删除后调整索引
    if (currentIndex.value > 0) currentIndex.value--
  }
  catch (e) { showWarning(String(e)) }
}

function addAccount() { router.push('/login') }
async function logout() { await authStore.logoutUser() }

// 拖动/滚轮导航（onSwitch 回调即 switchTo，switchTo 内部自带 switching 检查）
const {
  isDragging, dragMoved, cardTransform,
  onPointerDown, onPointerMove, onPointerUp, onWheel,
} = useSwipeNavigation(totalCards, currentIndex, switchTo)

onMounted(() => {
  authStore.loadMsAccounts()
  authStore.loadOfflineAccounts()
})
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
          @pointerdown="onPointerDown"
          @pointermove="onPointerMove"
          @pointerup="onPointerUp"
          @pointercancel="onPointerUp"
          @wheel="onWheel"
        >
          <div
            class="flex transition-transform duration-300 ease-out"
            :class="{ 'transition-none': isDragging && dragMoved }"
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
