<script setup lang="ts">
/**
 * 结束游戏悬浮按钮
 * 右下角红色圆钮，与 BackToTop / DownloadPanel 同款简约风格。
 * 显示前提：游戏已启动（runningPid 非空），点击调用 store 的 stopGame 停止游戏。
 * 位置协调：贴底（bottom 24px），BackToTop 在其可见时上移避让。
 */

import { watch, onUnmounted } from 'vue'
import { StopIcon } from '@heroicons/vue/24/solid'
import { useVersionStore } from '@/stores/version'
import { stopGameVisible } from '@/composables/useFloatingButtonState'

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
  // 与主页启动按钮行为一致：停止游戏
}

onUnmounted(() => { stopGameVisible.value = false })
</script>

<template>
  <Transition name="stop-game">
    <button
      v-if="versionStore.runningPid"
      class="stop-game-btn"
      title="结束游戏"
      @click="handleStopGame"
    >
      <StopIcon class="w-5 h-5 text-white" />
    </button>
  </Transition>
</template>

<style scoped>
.stop-game-btn {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 50;
  width: 44px;
  height: 44px;
  border-radius: 50%;
  background: var(--color-red-600, #dc2626);
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 8px rgb(220 38 38 / 0.35);
  transition: background-color 0.2s ease, transform 0.2s ease, box-shadow 0.2s ease;
}

.stop-game-btn:hover {
  background: var(--color-red-700, #b91c1c);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgb(220 38 38 / 0.4);
}

.stop-game-btn:active {
  transform: translateY(0) scale(0.95);
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
