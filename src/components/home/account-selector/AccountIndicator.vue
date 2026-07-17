<script setup lang="ts">
/**
 * 账号卡片栏指示器（圆点 + 计数）
 * - 每个圆点对应一张账号卡片，末尾圆点对应「添加账号」卡片
 * - 点击圆点切换到对应卡片
 */
import Tooltip from '@/components/common/Tooltip.vue'
import type { AccountCardData } from './AccountCard.vue'

defineProps<{
  cards: AccountCardData[]
  currentIndex: number
  hasAddCard: boolean
  totalCards: number
}>()

const emit = defineEmits<{ switch: [index: number] }>()
</script>

<template>
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
          @click="emit('switch', i)"
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
          @click="emit('switch', cards.length)"
        />
      </Tooltip>
      <span class="ml-1 text-[10px] text-gray-300">{{ currentIndex + 1 }}/{{ totalCards }}</span>
    </div>
  </div>
</template>
