<script setup lang="ts">
/**
 * 启动设置页
 * - Java 路径选择（JavaPathSelector 子组件）
 * - 内存分配（自动/自定义 + 可视化条 + 滑动条）
 * - 版本隔离（Select 下拉）
 * - 游戏目录（只读展示）
 */
import { ref, computed, watch, onMounted, nextTick } from 'vue'
import * as tauri from '@/utils/tauri'
import { formatBytes, formatMemoryMB } from '@/utils/format'
import { useDebouncedSave } from '@/composables/useDebouncedSave'
import { usePolling } from '@/composables/usePolling'
import Select from '@/components/common/Select.vue'
import JavaPathSelector from './settings-launch/JavaPathSelector.vue'

const gameDir = ref('')
const minMemory = ref(512)
const maxMemory = ref(2048)
const systemMemory = ref<{ total: number; available: number; usage_percent: number } | null>(null)
const memoryMode = ref<'auto' | 'custom'>('auto')
const loaded = ref(false)
const isolationMode = ref(4)
// 启动高级选项（参考 PCL2 PageSetupLaunch 高级选项）
const launchDisableJlw = ref(false)
const launchDisableLua = ref(false)
const launchUseDedicatedGpu = ref(false)

// 1秒自动刷新内存（参考PCL2）；onUnmounted 自动清理
const { start: startMemoryPolling } = usePolling(async () => {
  systemMemory.value = await tauri.getSystemMemory()
}, 1000)

const isolationOptions = [
  { label: '关闭', value: 0 },
  { label: '隔离 Mod 版本', value: 1 },
  { label: '隔离非正式版', value: 2 },
  { label: '隔离非正式版 + Mod 版本', value: 3 },
  { label: '隔离所有版本（推荐）', value: 4 },
]

const totalMemoryMB = computed(() => systemMemory.value ? Math.round(systemMemory.value.total / 1024 / 1024) : 0)
const usedMemoryMB = computed(() => systemMemory.value ? Math.round((systemMemory.value.total - systemMemory.value.available) / 1024 / 1024) : 0)
const gameMemoryMB = computed(() => maxMemory.value)
const otherMemoryMB = computed(() => Math.max(0, totalMemoryMB.value - usedMemoryMB.value - gameMemoryMB.value))

const usedPercent = computed(() => totalMemoryMB.value > 0 ? (usedMemoryMB.value / totalMemoryMB.value) * 100 : 0)
const gamePercent = computed(() => totalMemoryMB.value > 0 ? (gameMemoryMB.value / totalMemoryMB.value) * 100 : 0)

function applyAutoMemory() {
  if (!systemMemory.value) return
  const availableMB = systemMemory.value.available / 1024 / 1024
  const suggested = Math.min(Math.round(availableMB * 0.75), 8192)
  maxMemory.value = Math.max(suggested, 512)
  minMemory.value = Math.round(maxMemory.value / 2)
}

const { markDirty } = useDebouncedSave('patch', async (patch) => {
  try {
    await tauri.applyConfig(patch)
  } catch (e) {
    console.error('Failed to save settings:', e)
  }
}, 500)

// 自定义内存模式下：拖动滑块防抖保存
watch([minMemory, maxMemory], () => {
  if (!loaded.value) return
  // 自动模式下不保存内存配置到文件，启动时动态计算
  if (memoryMode.value === 'auto') return
  markDirty('minMemory', minMemory.value)
  markDirty('maxMemory', maxMemory.value)
})

// 版本隔离模式保存
watch(isolationMode, (mode) => {
  if (!loaded.value) return
  markDirty('isolationMode', mode)
})

// 启动高级选项保存
watch(launchDisableJlw, (v) => {
  if (!loaded.value) return
  tauri.applyConfig({ launchDisableJlw: v })
})
watch(launchDisableLua, (v) => {
  if (!loaded.value) return
  tauri.applyConfig({ launchDisableLua: v })
})
watch(launchUseDedicatedGpu, (v) => {
  if (!loaded.value) return
  tauri.applyConfig({ launchUseDedicatedGpu: v })
})

// 切换内存模式时，同步到后端
watch(memoryMode, (mode) => {
  if (mode === 'auto') {
    applyAutoMemory()
  }
  if (!loaded.value) return
  markDirty('memoryMode', mode)
})

onMounted(async () => {
  try {
    const cfg = await tauri.getConfigMap()
    gameDir.value = cfg.gameDir || '.minecraft'
    memoryMode.value = cfg.memoryMode === 'custom' ? 'custom' : 'auto'
    if (memoryMode.value === 'custom') {
      // 自定义模式：读取保存的内存值
      minMemory.value = cfg.minMemory
      maxMemory.value = cfg.maxMemory
    }
    isolationMode.value = cfg.isolationMode
    launchDisableJlw.value = cfg.launchDisableJlw
    launchDisableLua.value = cfg.launchDisableLua
    launchUseDedicatedGpu.value = cfg.launchUseDedicatedGpu
  } catch (e) {
    console.error('Failed to load config:', e)
    gameDir.value = '.minecraft'
  }
  try {
    systemMemory.value = await tauri.getSystemMemory()
    applyAutoMemory()
  } catch (e) {
    console.error('Failed to get system memory:', e)
  }
  // 等待 watch 回调执行完毕（避免加载值被误判为用户改动触发保存）
  await nextTick()
  loaded.value = true

  // 1秒自动刷新内存（参考PCL2）；onUnmounted 自动清理
  startMemoryPolling()
})

// 内存配置的防抖保存由 useDebouncedSave 在组件卸载时自动 flush；
// 轮询由 usePolling 在 onUnmounted 自动 stop。
</script>

<template>
  <div class="space-y-6">
    <!-- 启动参数 -->
    <div class="bg-white rounded-lg border border-gray-300">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">启动参数</h3>
      <div class="divide-y divide-gray-200">
        <!-- Java 路径（子组件） -->
        <JavaPathSelector />
      </div>
    </div>

    <!-- 内存分配 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">内存分配</h3>

      <!-- 内存可视化条 -->
      <div v-if="systemMemory" class="px-5 pb-4">
        <div class="flex items-center justify-between text-xs text-gray-500 mb-2">
          <span>{{ formatBytes(systemMemory.total) }} 总内存</span>
          <span>{{ systemMemory.usage_percent.toFixed(0) }}% 已占用</span>
        </div>
        <div class="h-6 rounded-full overflow-hidden flex bg-gray-100">
          <div
            class="h-full bg-orange-400 transition-all duration-500"
            :style="{ width: usedPercent + '%' }"
            :title="'系统已用: ' + formatMemoryMB(usedMemoryMB)"
          />
          <div
            class="h-full bg-primary-500 transition-all duration-500"
            :style="{ width: gamePercent + '%' }"
            :title="'游戏分配: ' + formatMemoryMB(gameMemoryMB)"
          />
          <div
            class="h-full bg-green-400 transition-all duration-500 flex-1"
            :title="'剩余: ' + formatMemoryMB(otherMemoryMB)"
          />
        </div>
        <div class="flex items-center gap-4 mt-2 text-xs">
          <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-orange-400"></span> 系统占用 {{ formatMemoryMB(usedMemoryMB) }}</span>
          <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-primary-500"></span> 游戏分配 {{ formatMemoryMB(gameMemoryMB) }}</span>
          <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-green-400"></span> 剩余 {{ formatMemoryMB(otherMemoryMB) }}</span>
        </div>
      </div>

      <div class="divide-y divide-gray-200">
        <!-- 分配模式 -->
        <div class="px-5 py-4 flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-900">分配模式</p>
            <p class="text-xs text-gray-500 mt-0.5">自动模式根据系统内存智能分配</p>
          </div>
          <div class="flex gap-2">
            <button
              class="px-4 py-1.5 text-sm font-medium rounded-lg border transition-colors"
              :class="memoryMode === 'auto'
                ? 'border-primary-500 bg-primary-50 text-primary-700'
                : 'border-gray-300 text-gray-600 hover:border-gray-400'"
              @click="memoryMode = 'auto'"
            >
              自动配置
            </button>
            <button
              class="px-4 py-1.5 text-sm font-medium rounded-lg border transition-colors"
              :class="memoryMode === 'custom'
                ? 'border-primary-500 bg-primary-50 text-primary-700'
                : 'border-gray-300 text-gray-600 hover:border-gray-400'"
              @click="memoryMode = 'custom'"
            >
              自定义
            </button>
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

    <!-- 版本隔离 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">版本隔离</h3>
      <div class="divide-y divide-gray-200">
        <div class="px-5 py-4 flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-900">隔离模式</p>
            <p class="text-xs text-gray-500 mt-0.5">控制不同版本是否共享存档、Mod、资源包等</p>
          </div>
          <Select
            :model-value="isolationMode"
            :options="isolationOptions"
            class="w-56"
            @update:model-value="isolationMode = $event as number"
          />
        </div>
      </div>
    </div>

    <!-- 高级选项（参考 PCL2 PageSetupLaunch 高级选项）-->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">高级选项</h3>
      <div class="divide-y divide-gray-200">
        <!-- 禁用 Java Launch Wrapper -->
        <div class="px-5 py-4 flex items-center justify-between">
          <div class="min-w-0 mr-4">
            <p class="text-sm font-medium text-gray-900">禁用 Java Launch Wrapper</p>
            <p class="text-xs text-gray-500 mt-0.5">JLW 用于修复 Java 18- 在中文路径下可能无法正常启动的问题，若启动异常可尝试关闭</p>
          </div>
          <button
            class="relative inline-flex h-6 w-11 flex-none items-center rounded-full transition-colors"
            :class="launchDisableJlw ? 'bg-primary-500' : 'bg-gray-300'"
            @click="launchDisableJlw = !launchDisableJlw"
          >
            <span
              class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform"
              :class="launchDisableJlw ? 'translate-x-6' : 'translate-x-1'"
            />
          </button>
        </div>
        <!-- 禁用 LWJGL Unsafe Agent -->
        <div class="px-5 py-4 flex items-center justify-between">
          <div class="min-w-0 mr-4">
            <p class="text-sm font-medium text-gray-900">禁用 LWJGL Unsafe Agent</p>
            <p class="text-xs text-gray-500 mt-0.5">LUA 用于修复 LWJGL 3.4.1 的性能问题，若游戏卡顿可尝试关闭</p>
          </div>
          <button
            class="relative inline-flex h-6 w-11 flex-none items-center rounded-full transition-colors"
            :class="launchDisableLua ? 'bg-primary-500' : 'bg-gray-300'"
            @click="launchDisableLua = !launchDisableLua"
          >
            <span
              class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform"
              :class="launchDisableLua ? 'translate-x-6' : 'translate-x-1'"
            />
          </button>
        </div>
        <!-- 使用高性能显卡 -->
        <div class="px-5 py-4 flex items-center justify-between">
          <div class="min-w-0 mr-4">
            <p class="text-sm font-medium text-gray-900">使用高性能显卡</p>
            <p class="text-xs text-gray-500 mt-0.5">自动在 Windows 设置中将 Java 改为使用独立显卡，提升游戏帧率</p>
          </div>
          <button
            class="relative inline-flex h-6 w-11 flex-none items-center rounded-full transition-colors"
            :class="launchUseDedicatedGpu ? 'bg-primary-500' : 'bg-gray-300'"
            @click="launchUseDedicatedGpu = !launchUseDedicatedGpu"
          >
            <span
              class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform"
              :class="launchUseDedicatedGpu ? 'translate-x-6' : 'translate-x-1'"
            />
          </button>
        </div>
      </div>
    </div>

    <!-- 游戏目录 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">游戏目录</h3>
      <div class="divide-y divide-gray-200">
        <div class="px-5 py-4 flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-900">存储路径</p>
            <p class="text-xs text-gray-500 mt-0.5">Minecraft 游戏数据存放位置（固定）</p>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-sm text-gray-600 max-w-xs truncate">{{ gameDir }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
