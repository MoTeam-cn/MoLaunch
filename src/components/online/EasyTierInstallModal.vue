<script setup lang="ts">
/**
 * easytier 内核安装进度弹窗（联机页搭桥前置依赖）
 *
 * 监听 `easytier-install-progress` 事件：download/extract 阶段显示进度 + 取消按钮，
 * done/error 阶段自动隐藏。安装由 useScaffolding.ensureInstalled 触发。
 */
import { computed, defineAsyncComponent } from 'vue'
import { useEasyTierInstall } from '@/composables/useEasyTierInstall'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))

const { progress, cancel } = useEasyTierInstall()

/** 下载/解压阶段显示弹窗（事件驱动，与谁触发安装无关） */
const visible = computed(() => {
  const ph = progress.value?.phase
  return ph === 'download' || ph === 'extract'
})
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="visible" class="fixed inset-0 z-[10000] flex items-center justify-center p-4">
        <div class="absolute inset-0 bg-black/50" />
        <div class="relative w-full max-w-sm bg-white rounded-2xl shadow-xl p-6">
          <h3 class="text-base font-semibold text-gray-900">正在下载 easytier 内核</h3>
          <p class="mt-1 text-xs text-gray-500">{{ progress?.message ?? '准备下载...' }}</p>
          <div class="mt-4 h-1.5 rounded-full bg-gray-100 overflow-hidden">
            <div
              class="h-full bg-primary-500 transition-all duration-300"
              :style="{ width: `${progress?.percent ?? 0}%` }"
            />
          </div>
          <div class="mt-5 flex justify-end">
            <Button type="outline" size="small" @click="cancel">取消</Button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>