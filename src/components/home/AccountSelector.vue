<script setup lang="ts">
/**
 * 账号选择器（单卡片展示 + 左右滑动切换动画）
 *
 * - 一次只展示一个账号卡片（大头像 + 用户名 + 类型）
 * - 左右箭头 / 拖动 / 滚轮 切换账号，带平滑滑动动画
 * - 末尾有"添加账号"卡片
 * - 离线账号显示默认皮肤（Steve/Alex）
 */

import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { showWarning } from '@/utils/toast'
import SkinAvatar from '@/components/common/SkinAvatar.vue'
import SkinManager from '@/components/common/SkinManager.vue'
import Tooltip from '@/components/common/Tooltip.vue'

const router = useRouter()
const authStore = useAuthStore()

const showSkinManager = ref(false)

/** 当前显示的卡片索引 */
const currentIndex = ref(0)

interface AccountCard {
  uuid: string
  username: string
  loginType: string  // 'Microsoft' | 'Offline'
  isExpired?: boolean
  isActive?: boolean
}

/**
 * 账号卡片列表（微软账号 + 离线账号，顺序稳定）
 *
 * 关键：cards 的顺序不随 currentUser 变化而重排，
 * 当前账号通过 isActive=true 标记，这样切换账号时 currentIndex 指向稳定不变。
 */
const cards = computed<AccountCard[]>(() => {
  const list: AccountCard[] = []
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
async function onCardSkin(card: AccountCard) {
  if (!card.isActive) {
    await switchAccount(card.uuid, card.loginType)
  }
  showSkinManager.value = true
}

/** 点击卡片上的"登出/删除"按钮 */
async function onCardLogout(card: AccountCard, event: Event) {
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
function login() { router.push('/login') }
async function logout() { await authStore.logoutUser() }

/** ---- 拖动切换支持 ---- **/
const isDragging = ref(false)
const dragOffset = ref(0)
let dragStartX = 0
let dragMoved = false

function onPointerDown(e: PointerEvent) {
  isDragging.value = true
  dragMoved = false
  dragStartX = e.clientX
  dragOffset.value = 0
}
function onPointerMove(e: PointerEvent) {
  if (!isDragging.value) return
  const dx = e.clientX - dragStartX
  if (Math.abs(dx) > 4) dragMoved = true
  dragOffset.value = dx
}
function onPointerUp() {
  if (!isDragging.value) return
  isDragging.value = false
  const threshold = 60
  if (dragOffset.value < -threshold && currentIndex.value < totalCards.value - 1) {
    next()
  } else if (dragOffset.value > threshold && currentIndex.value > 0) {
    prev()
  }
  dragOffset.value = 0
}
/** 鼠标滚轮左右切换（带节流，防止快速滑动并发请求） */
let lastWheelTime = 0
const WHEEL_THROTTLE_MS = 300  // 节流间隔，与切换动画时长匹配
function onWheel(e: WheelEvent) {
  // 只在非拖动、非切换中时响应滚轮
  if (isDragging.value || switching.value) return
  // 节流：间隔内忽略
  const now = Date.now()
  if (now - lastWheelTime < WHEEL_THROTTLE_MS) return

  let direction = 0
  if (Math.abs(e.deltaY) > Math.abs(e.deltaX)) {
    direction = e.deltaY > 0 ? 1 : -1
  } else if (e.deltaX !== 0) {
    direction = e.deltaX > 0 ? 1 : -1
  }
  if (direction === 0) return
  e.preventDefault()

  const newIndex = currentIndex.value + direction
  if (newIndex < 0 || newIndex >= totalCards.value) return
  lastWheelTime = now
  switchTo(newIndex)
}

/** 卡片滑动 transform */
const cardTransform = computed(() => {
  if (isDragging.value && dragMoved) {
    return `translateX(calc(-${currentIndex.value * 100}% + ${dragOffset.value}px))`
  }
  return `translateX(-${currentIndex.value * 100}%)`
})

onMounted(() => {
  authStore.loadMsAccounts()
  authStore.loadOfflineAccounts()
})
</script>

<template>
  <div class="account-card-container flex h-full w-full min-w-0 flex-col overflow-hidden">
    <!-- 未登录状态 -->
    <div v-if="!isLoggedIn" class="flex flex-col items-center py-8">
      <svg class="mb-5 h-12 w-12 text-gray-300" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 12a5 5 0 100-10 5 5 0 000 10zm0 2c-5 0-9 3-9 7v1h18v-1c0-4-4-7-9-7z" />
      </svg>
      <button
        class="rounded-lg bg-primary-600 px-6 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-700"
        @click="login"
      >
        立即登录
      </button>
      <div class="mt-4 flex gap-4 text-xs text-gray-300">
        <a class="cursor-pointer transition-colors hover:text-primary-500" @click="login">登录账号</a>
      </div>
    </div>

    <!-- 已登录状态：单卡片 + 左右切换 -->
    <template v-else>
      <!-- 卡片栏标题 + 指示器 -->
      <div class="mb-2 flex items-center justify-between px-1">
        <div class="text-xs font-medium text-gray-400">账号切换</div>
        <div class="flex items-center gap-1.5">
          <!-- 指示点（可点击切换） -->
          <Tooltip
            v-for="(card, i) in cards"
            :key="card.uuid"
            :text="card.username"
            position="bottom"
            :delay="200"
          >
            <button
              class="h-1.5 rounded-full transition-all hover:opacity-70"
              :class="i === currentIndex ? 'w-4 bg-primary-500' : 'w-1.5 bg-gray-300'"
              @click="switchTo(i)"
            />
          </Tooltip>
          <Tooltip
            v-if="hasAddCard"
            text="添加账号"
            position="bottom"
            :delay="200"
          >
            <button
              class="h-1.5 rounded-full transition-all hover:opacity-70"
              :class="currentIndex === cards.length ? 'w-4 bg-primary-500' : 'w-1.5 bg-gray-300'"
              @click="switchTo(cards.length)"
            />
          </Tooltip>
          <span class="ml-1 text-[10px] text-gray-300">{{ currentIndex + 1 }}/{{ totalCards }}</span>
        </div>
      </div>

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
            <div
              v-for="card in cards"
              :key="card.uuid"
              class="flex-none"
              style="width: 100%;"
            >
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
                  <button
                    class="flex items-center gap-1 rounded-md border border-gray-200 px-2.5 py-1 text-xs text-gray-600 transition-colors hover:border-primary-300 hover:bg-primary-50 hover:text-primary-600"
                    :title="card.loginType === '正版' ? '皮肤与披风管理' : '本地皮肤选择'"
                    @click.stop="onCardSkin(card)"
                  >
                    <svg class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
                      <path d="M10 2a8 8 0 100 16 8 8 0 000-16zm0 2a6 6 0 110 12 6 6 0 010-12z" />
                    </svg>
                    皮肤
                  </button>
                  <button
                    class="flex items-center gap-1 rounded-md border border-gray-200 px-2.5 py-1 text-xs text-red-500 transition-colors hover:border-red-300 hover:bg-red-50"
                    :title="card.isActive ? '退出登录' : '删除此账号'"
                    @click.stop="onCardLogout(card, $event)"
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
