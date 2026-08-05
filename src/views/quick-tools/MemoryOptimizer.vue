<script setup lang="ts">
/**
 * 内存优化子组件
 *
 * 调用 Windows API 释放进程工作集内存，支持轻量/强力两种模式。
 * 优化按钮位于右侧，清理模式使用 Button 组件 primary/outline 切换。
 */
import { ref } from 'vue'
import {
  CpuChipIcon,
  CheckCircleIcon,
  QuestionMarkCircleIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import Tag from '@/components/common/Tag.vue'
import AlertV2 from '@/components/common/AlertV2.vue'
import { toastSuccess, toastError, toastInfo } from '@/utils/toast'
import { showConfirmAsync } from '@/utils/modal'
import { memoryOptimize } from '@/utils/api/tools'
import type { MemoryOptimizeMode, MemoryOptimizeResult } from '@/utils/api/tools'
import { formatBytes } from '@/utils/format'

const memState = ref<'idle' | 'optimizing'>('idle')
const memMode = ref<MemoryOptimizeMode>('light')
const memResult = ref<MemoryOptimizeResult | null>(null)

const memModeTooltip = '轻量模式：仅清空所有进程的工作集，释放几十~几百 MB，响应快、几乎无副作用。\n强力模式：额外清空系统待机内存列表（standby list），可释放数 GB，但已缓存的应用下次启动会变慢。'

function setMemMode(mode: MemoryOptimizeMode) {
  memMode.value = mode
}

async function optimizeMemory() {
  // 强力模式需二次确认
  if (memMode.value === 'strong') {
    const confirmed = await showConfirmAsync(
      '确认强力优化',
      '强力模式将清空系统待机内存列表，可能导致已缓存的应用（如浏览器、其他游戏）下次启动变慢。确定继续吗？',
    )
    if (!confirmed) return
  }

  memState.value = 'optimizing'
  try {
    const result = await memoryOptimize(memMode.value)
    memResult.value = result
    if (result.freed_bytes > 0) {
      toastSuccess(`已释放 ${formatBytes(result.freed_bytes)} 内存`)
    } else {
      toastInfo('内存已处于较优状态，无需优化')
    }
  } catch (e) {
    toastError(`内存优化失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    memState.value = 'idle'
  }
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <CpuChipIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">内存优化</h3>
      <Tooltip :text="memModeTooltip">
        <QuestionMarkCircleIcon class="h-4 w-4 text-gray-400 hover:text-gray-600" />
      </Tooltip>
    </div>
    <div class="px-5 pb-5 space-y-3">
      <p class="text-xs text-gray-500">
        释放系统占用的多余内存，降低资源消耗。优化后部分应用可能短暂变慢，系统会按需重新分配内存。
      </p>

      <!-- 模式选择 + 优化按钮 -->
      <div class="flex items-center justify-between gap-3">
        <div class="flex items-center gap-2">
          <span class="text-xs font-medium text-gray-700">优化模式</span>
          <div class="flex gap-1.5">
            <Button
              :type="memMode === 'light' ? 'primary' : 'outline'"
              size="small"
              @click="setMemMode('light')"
            >轻量</Button>
            <Button
              :type="memMode === 'strong' ? 'primary' : 'outline'"
              size="small"
              @click="setMemMode('strong')"
            >强力</Button>
          </div>
        </div>
        <Button
          type="primary"
          size="default"
          :disabled="memState === 'optimizing'"
          @click="optimizeMemory"
        >
          <template #icon>
            <CpuChipIcon class="h-4 w-4" :class="{ 'animate-pulse': memState === 'optimizing' }" />
          </template>
          {{ memState === 'optimizing' ? '优化中...' : (memMode === 'strong' ? '强力优化' : '轻量优化') }}
        </Button>
      </div>

      <!-- 强力模式警告 -->
      <AlertV2
        v-if="memMode === 'strong'"
        type="warning"
        message="强力模式会清空系统待机内存列表（standby list），可能释放数 GB 内存，但已缓存的应用（如浏览器、其他游戏）下次启动会变慢。建议仅在内存严重不足时使用。"
      />

      <!-- 优化结果 -->
      <div v-if="memResult" class="rounded-lg bg-green-50 px-4 py-3">
        <div class="flex items-center gap-3">
          <CheckCircleIcon class="h-5 w-5 flex-none text-green-500" />
          <div class="flex-1">
            <span class="text-sm font-medium text-green-800">
              已释放 {{ formatBytes(memResult.freed_bytes) }}
            </span>
            <Tag size="small" color="green" class="ml-2">{{ memResult.mode === 'strong' ? '强力' : '轻量' }}模式</Tag>
          </div>
        </div>
        <div class="mt-1.5 pl-8 text-xs text-green-600">
          系统可用内存 {{ formatBytes(memResult.before_bytes) }} → {{ formatBytes(memResult.after_bytes) }}
        </div>
      </div>
    </div>
  </section>
</template>
