<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
/**
 * 主页右侧默认内容区 - 时钟卡片
 *
 * 顶部固定显示大时钟 + 日期 + 星期，
 * 底部轮播区每 6 秒自动翻页切换一张信息卡片：
 * - 内存使用（进度条）
 * - 已安装版本数
 * - 最近一次启动
 * - 缓存占用
 *
 * 数据加载失败时该卡片被跳过，不阻塞轮播。
 * 启动游戏时由 Home.vue 切换到 LaunchLog，本组件不处理启动状态。
 *
 * 逻辑已抽离到 composables/useHomeClockCards.ts。
 */
import { useHomeClockCards } from '@/composables/useHomeClockCards'
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))

const {
  timeText,
  secondsText,
  dateText,
  cards,
  currentIndex,
  currentCard,
  startCarousel,
} = useHomeClockCards()
</script>

<template>
  <div class="flex h-full flex-col items-center justify-center px-8 py-6">
    <!-- 顶部时钟区 -->
    <div class="flex-none text-center">
      <!-- 大时钟 -->
      <div class="flex items-baseline justify-center">
        <span class="text-7xl font-light tabular-nums text-gray-900 tracking-tight">
          {{ timeText }}
        </span>
        <span class="ml-2 text-2xl font-light tabular-nums text-primary-500">
          :{{ secondsText }}
        </span>
      </div>
      <!-- 日期 -->
      <p class="mt-3 text-sm text-gray-500">{{ dateText }}</p>
    </div>

    <!-- 底部轮播信息卡片 -->
    <div class="mt-10 h-32 w-full max-w-sm">
      <Transition name="card-flip" mode="out-in">
        <div
          v-if="currentCard"
          :key="currentCard.key"
          class="flex h-full flex-col rounded-xl border border-gray-200 bg-gradient-to-br from-gray-50 to-white px-5 py-4 shadow-sm"
        >
          <!-- 标题行 -->
          <div class="flex items-center gap-2 text-gray-500">
            <component :is="currentCard.icon" class="h-4 w-4" />
            <span class="text-xs font-medium">{{ currentCard.label }}</span>
          </div>

          <!-- 主数值 -->
          <p class="mt-2 flex-1 text-2xl font-semibold text-gray-900">
            {{ currentCard.value }}
          </p>

          <!-- 副信息 -->
          <p v-if="currentCard.sub" class="mt-1 truncate text-xs text-gray-500">
            {{ currentCard.sub }}
          </p>

          <!-- 进度条 -->
          <div
            v-if="currentCard.progress !== undefined"
            class="mt-2 h-1.5 overflow-hidden rounded-full bg-gray-100"
          >
            <div
              class="h-full rounded-full transition-all duration-500"
              :class="currentCard.progressColor ?? 'bg-primary-500'"
              :style="{ width: currentCard.progress + '%' }"
            />
          </div>
        </div>
      </Transition>

      <!-- 无卡片占位 -->
      <div
        v-if="cards.length === 0"
        class="flex h-full items-center justify-center text-xs text-gray-400"
      >
        加载中...
      </div>
    </div>

    <!-- 指示点 -->
    <div v-if="cards.length > 1" class="mt-4 flex items-center gap-1.5">
      <Tooltip
        v-for="(card, idx) in cards"
        :key="card.key"
        :text="card.label"
        position="top"
      >
        <div
          role="button"
          tabindex="0"
          class="h-1.5 rounded-full transition-all duration-300 cursor-pointer"
          :class="idx === currentIndex
            ? 'w-4 bg-primary-500'
            : 'w-1.5 bg-gray-300 hover:bg-gray-400'"
          @click="currentIndex = idx; startCarousel()"
          @keydown.enter="currentIndex = idx; startCarousel()"
        />
      </Tooltip>
    </div>
  </div>
</template>

<style scoped>
/* 翻页动画：向上滑动 + 淡入淡出 */
.card-flip-enter-active,
.card-flip-leave-active {
  transition: opacity 0.4s ease, transform 0.4s ease;
}
.card-flip-enter-from {
  opacity: 0;
  transform: translateY(12px);
}
.card-flip-leave-to {
  opacity: 0;
  transform: translateY(-12px);
}
</style>
