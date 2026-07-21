<script setup lang="ts">
/**
 * 版本设置 - 内存分配子组件
 * 抄全局设置 SettingsLaunch.vue 的可视化 UI，新增「跟随全局」模式。
 * 通过 updateVersionPersonalization 保存到 setup.ini。
 */
import { ref, computed, watch, onMounted } from 'vue'
import * as tauri from '@/utils/tauri'
import { showSuccess, showError, showWarning } from '@/utils/toast'
import { formatBytes, formatMemoryMB } from '@/utils/format'
import { useDebouncedSave } from '@/composables/useDebouncedSave'
import Tooltip from '@/components/common/Tooltip.vue'
import Button from '@/components/common/Button.vue'
import { useVersionSettings } from '@/composables/useVersionSettings'
import { useMemoryVisualizer } from '@/composables/useMemoryVisualizer'

const { selectedId, personalization } = useVersionSettings()

// 版本独立内存（从 setup.ini 读取）
const memoryMode = ref<'inherit' | 'auto' | 'custom'>('inherit')
const minMemory = ref(0)
const maxMemory = ref(0)
const loaded = ref(false)

// 全局内存（用于「跟随全局」时的描述）
const globalMode = ref('auto')
const globalMin = ref(0)
const globalMax = ref(0)

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

const globalMemoryDesc = computed(() => {
  if (globalMode.value === 'auto') return `自动配置（约 ${formatMemoryMB(globalMax.value) || '动态计算'}）`
  return `自定义 ${formatMemoryMB(globalMin.value)} / ${formatMemoryMB(globalMax.value)}`
})

/** 切换内存模式 */
async function handleSaveMemoryMode(mode: 'inherit' | 'auto' | 'custom') {
  if (!selectedId.value) return
  const backendMode = mode === 'inherit' ? '' : mode
  try {
    const update: tauri.PersonalizationUpdate = { memoryMode: backendMode }
    // 切换到自定义时立即保存当前默认值，避免需要拖动滑块才写入
    if (mode === 'custom') {
      if (maxMemory.value < 512) applyAutoMemory()
      update.maxMemory = maxMemory.value
      update.minMemory = minMemory.value
    }
    await tauri.updateVersionPersonalization(selectedId.value, update)
    if (personalization.value) {
      personalization.value.memoryMode = backendMode
      if (mode === 'custom') {
        personalization.value.maxMemory = maxMemory.value
        personalization.value.minMemory = minMemory.value
      }
    }
    memoryMode.value = mode
    if (mode === 'auto') applyAutoMemory()
    showSuccess(mode === 'inherit' ? '已设置为跟随全局' : mode === 'auto' ? '已设置为自动配置' : '已切换为自定义')
  } catch (e) { showError('保存失败：' + String(e)) }
}

// 自定义模式下防抖保存
async function flushSaveMemory() {
  if (!selectedId.value || memoryMode.value !== 'custom') return
  try {
    await tauri.updateVersionPersonalization(selectedId.value, {
      maxMemory: maxMemory.value,
      minMemory: minMemory.value,
    })
    if (personalization.value) {
      personalization.value.maxMemory = maxMemory.value
      personalization.value.minMemory = minMemory.value
    }
  } catch (e) { console.error('Failed to save memory:', e) }
}

const { scheduleSave: scheduleSaveMemory } = useDebouncedSave(flushSaveMemory, 500)

function autoSaveMemory() {
  if (!loaded.value || memoryMode.value !== 'custom') return
  scheduleSaveMemory()
}

watch([minMemory, maxMemory], () => {
  if (maxMemory.value < 512) { showWarning('最大内存不能小于 512 MB'); return }
  if (minMemory.value >= maxMemory.value) {
    minMemory.value = Math.floor(maxMemory.value / 2)
  }
  autoSaveMemory()
})

onMounted(async () => {
  try {
    const cfg = await tauri.getConfigMap()
    globalMode.value = cfg.memoryMode
    globalMin.value = cfg.minMemory
    globalMax.value = cfg.maxMemory
  } catch (e) { console.error('Failed to load global memory:', e) }

  // 从 personalization 读取版本独立内存
  const p = personalization.value
  if (p) {
    const m = p.memoryMode
    memoryMode.value = m === 'auto' || m === 'custom' ? m : 'inherit'
    if (memoryMode.value === 'custom') {
      maxMemory.value = p.maxMemory || globalMax.value
      minMemory.value = p.minMemory || Math.floor(maxMemory.value / 2)
    } else if (memoryMode.value === 'auto') {
      applyAutoMemory()
    }
  } else {
    applyAutoMemory()
  }
  loaded.value = true

  startMemoryPolling()
})

// 内存配置的防抖保存由 useDebouncedSave 在组件卸载时自动 flush；
// 轮询由 usePolling 在 onUnmounted 自动 stop。
</script>

<template>
  <section class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
    <h3 class="mb-4 text-sm font-semibold text-gray-700">内存分配</h3>

    <!-- 内存可视化条（跟随全局时不显示） -->
    <div v-if="memoryMode !== 'inherit' && systemMemory" class="mb-4">
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
        <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-orange-400" /> 系统占用 {{ formatMemoryMB(usedMemoryMB) }}</span>
        <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-primary-500" /> 游戏分配 {{ formatMemoryMB(gameMemoryMB) }}</span>
        <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-green-400" /> 剩余 {{ formatMemoryMB(otherMemoryMB) }}</span>
      </div>
    </div>

    <!-- 分配模式 -->
    <div class="mb-4">
      <div class="flex items-center justify-between mb-2">
        <p class="text-sm font-medium text-gray-900">分配模式</p>
      </div>
      <div class="flex gap-2">
        <Button
          :type="memoryMode === 'inherit' ? 'primary' : 'outline'"
          size="small"
          class="flex-1"
          @click="handleSaveMemoryMode('inherit')"
        >
          跟随全局
        </Button>
        <Button
          :type="memoryMode === 'auto' ? 'primary' : 'outline'"
          size="small"
          class="flex-1"
          @click="handleSaveMemoryMode('auto')"
        >
          自动配置
        </Button>
        <Button
          :type="memoryMode === 'custom' ? 'primary' : 'outline'"
          size="small"
          class="flex-1"
          @click="handleSaveMemoryMode('custom')"
        >
          自定义
        </Button>
      </div>
    </div>

    <!-- 跟随全局：显示全局当前设置 -->
    <div v-if="memoryMode === 'inherit'" class="rounded-lg bg-gray-50 px-4 py-3 text-xs text-gray-600">
      使用全局设置：{{ globalMemoryDesc }}
    </div>

    <!-- 自动配置 -->
    <div v-else-if="memoryMode === 'auto'" class="flex items-center justify-between rounded-lg bg-gray-50 px-4 py-3">
      <div>
        <p class="text-sm font-medium text-gray-900">游戏最大内存</p>
        <p class="text-xs text-gray-500 mt-0.5">根据可用内存自动分配 75%，上限 8 GB</p>
      </div>
      <span class="text-lg font-bold text-primary-600">{{ formatMemoryMB(maxMemory) }}</span>
    </div>

    <!-- 自定义：滑块 -->
    <div v-else class="space-y-4 rounded-lg bg-gray-50 px-4 py-3">
      <div>
        <div class="flex items-center justify-between mb-2">
          <p class="text-sm font-medium text-gray-900">最大内存</p>
          <span class="text-sm font-bold text-primary-600">{{ formatMemoryMB(maxMemory) }}</span>
        </div>
        <input v-model.number="maxMemory" type="range" class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer" min="512" :max="totalMemoryMB > 0 ? Math.min(totalMemoryMB, 16384) : 8192" step="256" />
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
        <input v-model.number="minMemory" type="range" class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer" min="256" :max="maxMemory" step="256" />
        <div class="flex justify-between text-xs text-gray-400 mt-1">
          <span>256 MB</span>
          <span>{{ formatMemoryMB(maxMemory) }}</span>
        </div>
      </div>
    </div>
  </section>
</template>
