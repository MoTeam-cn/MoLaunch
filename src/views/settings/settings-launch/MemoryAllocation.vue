<script setup lang="ts">
/**
 * 内存分配子组件（SettingsLaunch 专用）
 *
 * 从 SettingsLaunch.vue 抽取的内存分配区块，包含：
 * - 系统内存可视化条（复用 useMemoryVisualizer composable）
 * - 分配模式切换（自动 / 自定义）
 * - 自动模式说明 / 自定义模式滑动条
 *
 * 配置加载与防抖保存复用 useConfigPage composable，独立维护 loaded 守卫。
 */
import { computed, ref, watch, defineAsyncComponent } from 'vue'
import * as tauri from '@/utils/tauri'
import { formatMemoryMB } from '@/utils/format'
import { useConfigPage } from '@/composables/useConfigPage'
import { useMemoryVisualizer } from '@/composables/useMemoryVisualizer'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Slider = defineAsyncComponent(() => import('@/components/common/Slider.vue'))
const MemoryUsageBar = defineAsyncComponent(() => import('@/components/settings/MemoryUsageBar.vue'))
import { safeCall } from '@/utils/async'
import { toastError } from '@/utils/toast'
import { recommendJvm } from '@/utils/jvm-recommend'

const minMemory = ref(512)
const maxMemory = ref(2048)
const memoryMode = ref<'auto' | 'custom'>('auto')

// 系统内存可视化（复用 useMemoryVisualizer composable）
const {
  systemMemory,
  totalMemoryMB,
  usedMemoryMB,
  gameMemoryMB,
  otherMemoryMB,
  usedPercent,
  gamePercent,
  applyAutoMemory,
  startMemoryPolling,
} = useMemoryVisualizer(maxMemory, minMemory)

const { loaded, markDirty } = useConfigPage({
  delay: 500,
  errorLabel: 'save launch settings',
  onLoad: async (cfg) => {
    memoryMode.value = cfg.memoryMode === 'custom' ? 'custom' : 'auto'
    if (memoryMode.value === 'custom') {
      // 自定义模式：读取保存的内存值
      minMemory.value = cfg.minMemory
      maxMemory.value = cfg.maxMemory
    }

    // 加载配置后获取系统内存并启动轮询
    await safeCall(async () => {
      systemMemory.value = await tauri.getSystemMemory()
      applyAutoMemory()
    }, 'get system memory', () => toastError('获取系统内存信息失败'))
    startMemoryPolling()
  },
})

// 自定义内存模式下：拖动滑块防抖保存
watch([minMemory, maxMemory], () => {
  if (!loaded.value) return
  // 自动模式下不保存内存配置到文件，启动时动态计算
  if (memoryMode.value === 'auto') return
  markDirty('minMemory', minMemory.value)
  markDirty('maxMemory', maxMemory.value)
})

// 切换内存模式时，同步到后端
watch(memoryMode, (mode) => {
  if (mode === 'auto') {
    applyAutoMemory()
  }
  if (!loaded.value) return
  markDirty('memoryMode', mode)
})

/** 按物理内存给出的推荐值 */
const recommendation = computed(() => recommendJvm(totalMemoryMB.value))

/** 采用推荐值：切到自定义模式并写入推荐内存 */
function useRecommendation() {
  memoryMode.value = 'custom'
  maxMemory.value = recommendation.value.memoryMB
  if (minMemory.value > maxMemory.value) minMemory.value = maxMemory.value
}

// 内存配置的防抖保存由 useDebouncedSave 在组件卸载时自动 flush；
// 轮询由 usePolling 在 onUnmounted 自动 stop。
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">内存分配</h3>

    <!-- 内存可视化条 -->
    <div v-if="systemMemory" class="px-5 pb-4">
      <MemoryUsageBar
        :system-memory="systemMemory"
        :used-mb="usedMemoryMB"
        :game-mb="gameMemoryMB"
        :other-mb="otherMemoryMB"
        :used-percent="usedPercent"
        :game-percent="gamePercent"
      />
    </div>

    <!-- 推荐值 -->
    <div class="mx-5 mb-4 rounded-md bg-gray-50 px-3 py-2.5">
      <div class="flex items-center justify-between gap-3">
        <p class="text-xs text-gray-600">
          推荐 {{ formatMemoryMB(recommendation.memoryMB) }}：{{ recommendation.reason }}
        </p>
        <Button size="small" type="outline" class="flex-none" @click="useRecommendation">
          用推荐值
        </Button>
      </div>
      <p class="mt-1 text-xs text-gray-400">
        通用参数：{{ recommendation.extraArgs.join(' ') }}（GC 由启动器按 Java 版本自动选择）
      </p>
    </div>

    <div class="divide-y divide-gray-200">
      <!-- 分配模式 -->
      <div class="px-5 py-4">
        <div class="flex items-center justify-between mb-2">
          <div>
            <p class="text-sm font-medium text-gray-900">分配模式</p>
            <p class="text-xs text-gray-500 mt-0.5">自动模式根据系统内存智能分配</p>
          </div>
        </div>
        <div class="flex gap-2">
          <Button
            :type="memoryMode === 'auto' ? 'primary' : 'outline'"
            size="small"
            class="flex-1"
            @click="memoryMode = 'auto'"
          >
            自动配置
          </Button>
          <Button
            :type="memoryMode === 'custom' ? 'primary' : 'outline'"
            size="small"
            class="flex-1"
            @click="memoryMode = 'custom'"
          >
            自定义
          </Button>
        </div>
      </div>
      <!-- 自动模式说明 -->
      <div v-if="memoryMode === 'auto'" class="px-5 py-4">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-900">游戏最大内存</p>
            <p class="text-xs text-gray-500 mt-0.5">根据可用内存自动分配 75%，上限 8 GB</p>
          </div>
          <span class="text-lg font-bold text-primary-600">{{ formatMemoryMB(maxMemory) }}</span>
        </div>
      </div>
      <!-- 自定义模式：滑动条 -->
      <div v-else class="px-5 py-4 space-y-4">
        <div>
          <div class="flex items-center justify-between mb-2">
            <p class="text-sm font-medium text-gray-900">最大内存</p>
            <span class="text-sm font-bold text-primary-600">{{ formatMemoryMB(maxMemory) }}</span>
          </div>
          <Slider
            v-model="maxMemory"
            :min="512"
            :max="totalMemoryMB > 0 ? Math.min(totalMemoryMB, 16384) : 8192"
            :step="256"
            class="w-full"
          />
          <div class="flex justify-between text-xs text-gray-400 mt-1">
            <span>512 MB</span>
            <span>{{ formatMemoryMB(totalMemoryMB > 0 ? Math.min(totalMemoryMB, 16384) : 8192) }}</span>
          </div>
        </div>
        <div>
          <div class="flex items-center justify-between mb-2">
            <p class="text-sm font-medium text-gray-900">最小内存</p>
            <span class="text-sm font-bold text-gray-600">{{ formatMemoryMB(minMemory) }}</span>
          </div>
          <Slider
            v-model="minMemory"
            :min="256"
            :max="maxMemory"
            :step="256"
            class="w-full"
          />
          <div class="flex justify-between text-xs text-gray-400 mt-1">
            <span>256 MB</span>
            <span>{{ formatMemoryMB(maxMemory) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
