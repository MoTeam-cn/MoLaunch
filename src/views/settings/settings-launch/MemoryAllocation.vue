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
import { ref, watch } from 'vue'
import * as tauri from '@/utils/tauri'
import { formatBytes, formatMemoryMB } from '@/utils/format'
import { useConfigPage } from '@/composables/useConfigPage'
import { useMemoryVisualizer } from '@/composables/useMemoryVisualizer'
import Tooltip from '@/components/common/Tooltip.vue'
import Button from '@/components/common/Button.vue'
import { safeCall } from '@/utils/async'
import { toastError } from '@/utils/toast'

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

// 内存配置的防抖保存由 useDebouncedSave 在组件卸载时自动 flush；
// 轮询由 usePolling 在 onUnmounted 自动 stop。
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">内存分配</h3>

    <!-- 内存可视化条 -->
    <div v-if="systemMemory" class="px-5 pb-4">
      <div class="flex items-center justify-between text-xs text-gray-500 mb-2">
        <span>{{ formatBytes(systemMemory.total) }} 总内存</span>
        <span>{{ systemMemory.usage_percent.toFixed(0) }}% 已占用</span>
      </div>
      <div class="h-6 rounded-full overflow-hidden flex bg-gray-100">
        <div class="h-full flex" :style="{ width: usedPercent + '%' }">
          <Tooltip :text="'系统已用: ' + formatMemoryMB(usedMemoryMB)" position="top" block>
            <div class="h-full w-full bg-orange-400 transition-all duration-500" />
          </Tooltip>
        </div>
        <div class="h-full flex" :style="{ width: gamePercent + '%' }">
          <Tooltip :text="'游戏分配: ' + formatMemoryMB(gameMemoryMB)" position="top" block>
            <div class="h-full w-full bg-primary-500 transition-all duration-500" />
          </Tooltip>
        </div>
        <div class="h-full flex flex-1">
          <Tooltip :text="'剩余: ' + formatMemoryMB(otherMemoryMB)" position="top" block>
            <div class="h-full w-full bg-green-400 transition-all duration-500" />
          </Tooltip>
        </div>
      </div>
      <div class="flex items-center gap-4 mt-2 text-xs">
        <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-orange-400"></span> 系统占用 {{ formatMemoryMB(usedMemoryMB) }}</span>
        <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-primary-500"></span> 游戏分配 {{ formatMemoryMB(gameMemoryMB) }}</span>
        <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-green-400"></span> 剩余 {{ formatMemoryMB(otherMemoryMB) }}</span>
      </div>
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
          <input
            v-model.number="maxMemory"
            type="range"
            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
            min="512"
            :max="totalMemoryMB > 0 ? Math.min(totalMemoryMB, 16384) : 8192"
            step="256"
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
          <input
            v-model.number="minMemory"
            type="range"
            class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
            min="256"
            :max="maxMemory"
            step="256"
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
