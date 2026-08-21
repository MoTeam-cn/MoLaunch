<script setup lang="ts">
/**
 * 结束游戏悬浮按钮
 * 右下角圆钮，视觉与 BackToTop 完全统一：主题色底 + 停止方块图标。
 * 显示前提：游戏已启动（runningPid 非空），点击调用 store 的 stopGame 停止游戏。
 * 位置协调：贴底（bottom 24px），BackToTop 在其可见时上移避让。
 */

import { watch, onUnmounted } from 'vue'
import { StopIcon } from '@heroicons/vue/24/solid'
import { useVersionStore } from '@/stores/version'
import { stopGameVisible } from '@/composables/useFloatingButtonState'
import Tooltip from '@/components/common/Tooltip.vue'

const versionStore = useVersionStore()

// 同步可见状态到共享 ref，供 BackToTop / DownloadPanel 调整位置
watch(
  () => versionStore.runningPid,
  (pid) => { stopGameVisible.value = pid !== null },
  { immediate: true },
)

function handleStopGame() {
  if (!versionStore.runningPid) return
  versionStore.stopGame()
}

onUnmounted(() => { stopGameVisible.value = false })
</script>

<template>
  <Transition name="stop-game">
    <!-- fixed 定位落在 Tooltip 根元素（trigger）上，按钮撑满；悬浮提示"结束游戏" -->
    <Tooltip
      v-if="versionStore.runningPid"
      class="stop-game-trigger"
      text="结束游戏"
      position="top"
    >
      <button
        class="stop-game-btn"
        title="结束游戏"
        @click="handleStopGame"
      >
        <StopIcon class="w-5 h-5 text-white" />
      </button>
    </Tooltip>
  </Transition>
</template>

<style scoped>
/* fixed 定位在 Tooltip 根元素（trigger），确保 tooltip 位置基准正确 */
.stop-game-trigger {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 50;
  width: 44px;
  height: 44px;
}

.stop-game-btn {
  width: 100%;
  height: 100%;
  border-radius: 50%;
  background: var(--color-primary-600);
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 8px rgb(var(--color-primary-rgb-600) / 0.35);
  transition: background-color 0.2s ease, transform 0.2s ease, box-shadow 0.2s ease;
}

.stop-game-btn:hover {
  background: var(--color-primary-700);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgb(var(--color-primary-rgb-600) / 0.4);
}

.stop-game-btn:active {
  transform: scale(0.95);
}

/* 进入/离开动画（简洁淡入滑入） */
.stop-game-enter-active {
  animation: slide-in 0.25s ease-out;
}

.stop-game-leave-active {
  animation: slide-out 0.2s ease-in forwards;
}

@keyframes slide-in {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes slide-out {
  to {
    opacity: 0;
    transform: translateY(8px);
  }
}
</style>
