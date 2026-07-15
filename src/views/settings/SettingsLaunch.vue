<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useJavaStore } from '@/stores/java'
import * as tauri from '@/utils/tauri'
import { showInfo, showSuccess } from '@/utils/toast'
import { showError } from '@/utils/modal'
import { ArrowPathIcon, DocumentPlusIcon } from '@heroicons/vue/24/outline'
import { formatBytes, formatMemoryMB } from '@/utils/format'
import { useDebouncedSave } from '@/composables/useDebouncedSave'
import Select from '@/components/common/Select.vue'

const javaStore = useJavaStore()

const gameDir = ref('')
const minMemory = ref(512)
const maxMemory = ref(2048)
const systemMemory = ref<{ total: number; available: number; usage_percent: number } | null>(null)
const memoryMode = ref<'auto' | 'custom'>('auto')
const showJavaList = ref(false)
const javaSelectorRef = ref<HTMLElement | null>(null)
const loaded = ref(false)
const detectingJava = ref(false)
const isolationMode = ref(4)

const isolationOptions = [
  { label: '关闭', value: 0 },
  { label: '隔离 Mod 版本', value: 1 },
  { label: '隔离非正式版', value: 2 },
  { label: '隔离非正式版 + Mod 版本', value: 3 },
  { label: '隔离所有版本（推荐）', value: 4 },
]

function handleDocumentClick(e: MouseEvent) {
  if (javaSelectorRef.value && !javaSelectorRef.value.contains(e.target as Node)) {
    showJavaList.value = false
  }
}

watch(showJavaList, (open) => {
  if (open) {
    setTimeout(() => document.addEventListener('click', handleDocumentClick), 0)
  } else {
    document.removeEventListener('click', handleDocumentClick)
  }
})



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


async function handleAutoDetectJava() {
  if (detectingJava.value) return
  detectingJava.value = true
  try {
    showInfo('正在尝试搜索系统中存在的 Java...')
    await javaStore.refreshJava()
    if (javaStore.javaList.length > 0) {
      showSuccess(`已找到 ${javaStore.javaList.length} 个可用 Java，请自行展开下拉框选择`)
    } else {
      showInfo('未检测到已安装的 Java')
    }
  } finally {
    detectingJava.value = false
  }
}

async function handleManualImportJava() {
  try {
    const selected = await tauri.selectFile('选择 javaw.exe', [
      { name: 'Java 可执行文件 (javaw.exe)', extensions: ['exe'] },
    ])
    if (selected) {
      // 验证必须是 javaw.exe
      const fileName = selected.split('\\').pop()?.split('/').pop()?.toLowerCase()
      if (fileName !== 'javaw.exe') {
        showError('提示', '请选择 javaw.exe，而不是 java.exe')
        return
      }
      javaStore.setJavaPath(selected)
    }
  } catch (e) {
    console.error('Failed to select Java:', e)
  }
}

async function flushSave() {
  try {
    await tauri.setMinMemory(minMemory.value)
    await tauri.setMaxMemory(maxMemory.value)
  } catch (e) {
    console.error('Failed to save memory settings:', e)
  }
}

const { scheduleSave } = useDebouncedSave(flushSave, 500)

function autoSave() {
  if (!loaded.value) return
  // 自动模式下不保存内存配置到文件，启动时动态计算
  if (memoryMode.value === 'auto') return

  scheduleSave()
}

watch([minMemory, maxMemory], autoSave)

// 版本隔离模式保存
watch(isolationMode, async (mode) => {
  if (!loaded.value) return
  try {
    await tauri.setIsolationMode(mode)
  } catch (e) {
    console.error('Failed to save isolation mode:', e)
  }
})

// 切换内存模式时，同步到后端
watch(memoryMode, async (mode) => {
  if (mode === 'auto') {
    applyAutoMemory()
  }
  try {
    await tauri.setMemoryMode(mode)
  } catch (e) {
    console.error('Failed to set memory mode:', e)
  }
})

onMounted(async () => {
  try {
    gameDir.value = await tauri.getGameDir()
  } catch (e) {
    console.error('Failed to get game dir:', e)
    gameDir.value = '.minecraft'
  }
  try {
    systemMemory.value = await tauri.getSystemMemory()
    applyAutoMemory()
  } catch (e) {
    console.error('Failed to get system memory:', e)
  }
  try {
    // 从后端读取内存模式
    const mode = await tauri.getMemoryMode()
    memoryMode.value = mode === 'custom' ? 'custom' : 'auto'
    
    if (memoryMode.value === 'custom') {
      // 自定义模式：读取保存的内存值
      const [min, max] = await tauri.getMemoryConfig()
      minMemory.value = min
      maxMemory.value = max
    } else {
      // 自动模式：使用 applyAutoMemory 计算的值
    }
  } catch (e) {
    console.error('Failed to get memory config:', e)
  }
  try {
    isolationMode.value = await tauri.getIsolationMode()
  } catch (e) {
    console.error('Failed to get isolation mode:', e)
  }
  // 等待 watch 回调执行完毕（避免加载值被误判为用户改动触发保存）
  await nextTick()
  loaded.value = true

  // 1秒自动刷新内存（参考PCL2）
  memoryTimer = setInterval(async () => {
    try {
      systemMemory.value = await tauri.getSystemMemory()
    } catch (e) {
      // 静默失败
    }
  }, 1000)
})

// 内存刷新定时器
let memoryTimer: ReturnType<typeof setInterval> | null = null

// 组件卸载时清除定时器并 flush 待保存的内存配置
onUnmounted(() => {
  if (memoryTimer) {
    clearInterval(memoryTimer)
    memoryTimer = null
  }
  // 内存配置的防抖保存由 useDebouncedSave 在组件卸载时自动 flush
})
</script>

<template>
  <div class="space-y-6">
    <!-- 启动参数 -->
    <div class="bg-white rounded-lg border border-gray-300">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">启动参数</h3>
      <div class="divide-y divide-gray-200">
        <!-- Java 路径 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between mb-2">
            <div>
              <p class="text-sm font-medium text-gray-900">Java 路径</p>
              <p class="text-xs text-gray-500 mt-0.5">选择用于启动游戏的 Java 运行时</p>
            </div>
            <div class="flex items-center gap-1.5">
              <button
                class="px-2 py-1 text-xs rounded transition-colors flex items-center"
                :class="detectingJava 
                  ? 'text-gray-400 bg-gray-50 cursor-not-allowed' 
                  : 'text-gray-600 bg-gray-100 hover:bg-gray-200'"
                :disabled="detectingJava"
                @click="handleAutoDetectJava"
              >
                <ArrowPathIcon class="w-3.5 h-3.5 mr-1" :class="{ 'animate-spin': detectingJava }" />
                {{ detectingJava ? '检测中...' : '自动检测' }}
              </button>
              <button
                class="px-2 py-1 text-xs text-gray-600 bg-gray-100 hover:bg-gray-200 rounded transition-colors flex items-center"
                @click="handleManualImportJava"
              >
                <DocumentPlusIcon class="w-3.5 h-3.5 mr-1" />
                手动导入
              </button>
            </div>
          </div>
          <!-- 选择器整体容器 -->
          <div ref="javaSelectorRef" class="relative">
            <!-- 输入框 -->
            <div
              class="flex items-center justify-between px-3 py-2 bg-white border rounded-lg cursor-pointer transition-colors"
              :class="showJavaList ? 'border-primary-500 ring-2 ring-primary-100' : 'border-gray-300 hover:border-gray-400'"
              @click="showJavaList = !showJavaList"
            >
              <div class="flex items-center min-w-0 mr-2">
                <span class="text-xs px-1.5 py-0.5 rounded bg-primary-100 text-primary-700 mr-2 shrink-0">
                  {{ javaStore.javaPath ? 'Java ' + (javaStore.javaList.find(j => j.executable === javaStore.javaPath)?.major_version || '?') : '自动' }}
                </span>
                <span class="text-sm text-gray-900 truncate">
                  {{ javaStore.javaPath || '启动时自动查找最佳 Java' }}
                </span>
              </div>
              <svg class="w-4 h-4 text-gray-400 shrink-0 transition-transform" :class="{ 'rotate-180': showJavaList }" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
              </svg>
            </div>
            <!-- 下拉列表 -->
            <transition
              enter-active-class="transition ease-out duration-150"
              enter-from-class="opacity-0 scale-y-95"
              enter-to-class="opacity-100 scale-y-100"
              leave-active-class="transition ease-in duration-100"
              leave-from-class="opacity-100 scale-y-100"
              leave-to-class="opacity-0 scale-y-95"
            >
              <div
                v-if="showJavaList"
                class="absolute z-10 w-full mt-1 bg-white border border-gray-300 rounded-lg shadow-lg overflow-hidden origin-top"
              >
                <!-- 自动检测 -->
                <div
                  class="flex items-center justify-between px-3 py-2.5 hover:bg-primary-50 cursor-pointer transition-colors"
                  :class="{ 'bg-primary-50': !javaStore.javaPath }"
                  @click="javaStore.setJavaPath(''); showJavaList = false"
                >
                  <div class="flex items-center">
                    <span class="text-xs px-1.5 py-0.5 rounded bg-gray-100 text-gray-600 mr-2">自动</span>
                    <span class="text-sm text-gray-700">启动时自动查找最佳 Java</span>
                  </div>
                  <span v-if="!javaStore.javaPath" class="text-primary-600 text-xs font-medium">当前</span>
                </div>
                <!-- 已安装列表 -->
                <template v-if="javaStore.javaList.length > 0">
                  <div class="border-t border-gray-200 mx-3"></div>
                  <div
                    v-for="java in javaStore.javaList"
                    :key="java.executable"
                    class="flex items-center justify-between px-3 py-2.5 hover:bg-primary-50 cursor-pointer transition-colors"
                    :class="{ 'bg-primary-50': javaStore.javaPath === java.executable }"
                    @click="javaStore.setJavaPath(java.executable); showJavaList = false"
                  >
                    <div class="flex items-center min-w-0">
                      <span class="text-xs px-1.5 py-0.5 rounded bg-blue-100 text-blue-700 mr-2 shrink-0">
                        {{ java.major_version }}
                      </span>
                      <div class="min-w-0">
                        <div class="text-sm text-gray-900 truncate">{{ java.executable }}</div>
                        <div class="text-xs text-gray-500">{{ java.version }} · {{ java.is_64bit ? '64位' : '32位' }} · {{ java.is_jre ? 'JRE' : 'JDK' }}</div>
                      </div>
                    </div>
                    <span v-if="javaStore.javaPath === java.executable" class="text-primary-600 text-xs font-medium ml-2 shrink-0">当前</span>
                  </div>
                </template>
                <div v-else class="px-3 py-2.5 border-t border-gray-100 text-xs text-gray-400">
                  未检测到已安装的 Java，请点击「手动导入」
                </div>
              </div>
            </transition>
          </div>
        </div>
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
