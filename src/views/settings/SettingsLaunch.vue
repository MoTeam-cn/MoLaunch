<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useSdkStore } from '@/stores/sdk'
import * as tauri from '@/utils/tauri'
import { showInfo, showSuccess } from '@/utils/toast'
import { FolderOpenIcon, ArrowPathIcon, DocumentPlusIcon } from '@heroicons/vue/24/outline'

const sdkStore = useSdkStore()

const gameDir = ref('')
const minMemory = ref(512)
const maxMemory = ref(2048)
const systemMemory = ref<{ total: number; available: number; usage_percent: number } | null>(null)
const memoryMode = ref<'auto' | 'custom'>('auto')
const showJavaList = ref(false)
const javaSelectorRef = ref<HTMLElement | null>(null)
const loaded = ref(false)

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

function formatMemory(mb: number): string {
  if (mb >= 1024) {
    return (mb / 1024).toFixed(1).replace(/\.0$/, '') + ' GB'
  }
  return mb + ' MB'
}

function formatBytes(bytes: number): string {
  const mb = bytes / 1024 / 1024
  if (mb >= 1024) {
    return (mb / 1024).toFixed(1).replace(/\.0$/, '') + ' GB'
  }
  return Math.round(mb) + ' MB'
}

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

watch(memoryMode, (mode) => {
  if (mode === 'auto') applyAutoMemory()
})

async function handleAutoDetectJava() {
  showInfo('正在尝试搜索系统中存在的 Java...')
  await sdkStore.refreshJava()
  if (sdkStore.javaList.length > 0) {
    showSuccess(`已找到 ${sdkStore.javaList.length} 个可用 Java，请自行展开下拉框选择`)
  } else {
    showInfo('未检测到已安装的 Java')
  }
}

async function handleManualImportJava() {
  try {
    const selected = await tauri.selectFile('选择 Java 可执行文件', [
      { name: 'Java 可执行文件', extensions: ['exe'] },
      { name: '所有文件', extensions: ['*'] },
    ])
    if (selected) {
      sdkStore.javaPath = selected
    }
  } catch (e) {
    console.error('Failed to select Java:', e)
  }
}

async function handleSelectFolder() {
  const selected = await tauri.selectFolder()
  if (selected) {
    gameDir.value = selected
    try {
      await tauri.setGameDir(selected)
    } catch (e) {
      console.error('Failed to set game dir:', e)
    }
  }
}

let saveTimer: ReturnType<typeof setTimeout> | null = null
function autoSave() {
  if (!loaded.value) return
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => {
    // TODO: 持久化游戏设置到后端
  }, 500)
}

watch([gameDir, minMemory, maxMemory], autoSave)

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
  loaded.value = true
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
                class="px-2 py-1 text-xs text-gray-600 bg-gray-100 hover:bg-gray-200 rounded transition-colors flex items-center"
                @click="handleAutoDetectJava"
              >
                <ArrowPathIcon class="w-3.5 h-3.5 mr-1" />
                自动检测
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
                  {{ sdkStore.javaPath ? 'Java ' + (sdkStore.javaList.find(j => j.executable === sdkStore.javaPath)?.major_version || '?') : '自动' }}
                </span>
                <span class="text-sm text-gray-900 truncate">
                  {{ sdkStore.javaPath || '启动时自动查找最佳 Java' }}
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
                  :class="{ 'bg-primary-50': !sdkStore.javaPath }"
                  @click="sdkStore.javaPath = ''; showJavaList = false"
                >
                  <div class="flex items-center">
                    <span class="text-xs px-1.5 py-0.5 rounded bg-gray-100 text-gray-600 mr-2">自动</span>
                    <span class="text-sm text-gray-700">启动时自动查找最佳 Java</span>
                  </div>
                  <span v-if="!sdkStore.javaPath" class="text-primary-600 text-xs font-medium">当前</span>
                </div>
                <!-- 已安装列表 -->
                <template v-if="sdkStore.javaList.length > 0">
                  <div
                    v-for="java in sdkStore.javaList"
                    :key="java.executable"
                    class="flex items-center justify-between px-3 py-2.5 border-t border-gray-100 hover:bg-primary-50 cursor-pointer transition-colors"
                    :class="{ 'bg-primary-50': sdkStore.javaPath === java.executable }"
                    @click="sdkStore.javaPath = java.executable; showJavaList = false"
                  >
                    <div class="flex items-center min-w-0">
                      <span class="text-xs px-1.5 py-0.5 rounded bg-blue-100 text-blue-700 mr-2 shrink-0">
                        {{ java.major_version }}
                      </span>
                      <span class="text-sm text-gray-900 truncate">{{ java.executable }}</span>
                    </div>
                    <span v-if="sdkStore.javaPath === java.executable" class="text-primary-600 text-xs font-medium ml-2 shrink-0">当前</span>
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
            :title="'系统已用: ' + formatMemory(usedMemoryMB)"
          />
          <div
            class="h-full bg-primary-500 transition-all duration-500"
            :style="{ width: gamePercent + '%' }"
            :title="'游戏分配: ' + formatMemory(gameMemoryMB)"
          />
          <div
            class="h-full bg-green-400 transition-all duration-500 flex-1"
            :title="'剩余: ' + formatMemory(otherMemoryMB)"
          />
        </div>
        <div class="flex items-center gap-4 mt-2 text-xs">
          <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-orange-400"></span> 系统占用 {{ formatMemory(usedMemoryMB) }}</span>
          <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-primary-500"></span> 游戏分配 {{ formatMemory(gameMemoryMB) }}</span>
          <span class="flex items-center gap-1"><span class="w-2.5 h-2.5 rounded-full bg-green-400"></span> 剩余 {{ formatMemory(otherMemoryMB) }}</span>
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
            <span class="text-lg font-bold text-primary-600">{{ formatMemory(maxMemory) }}</span>
          </div>
        </div>
        <!-- 自定义模式：滑动条 -->
        <div v-else class="px-5 py-4 space-y-4">
          <div>
            <div class="flex items-center justify-between mb-2">
              <p class="text-sm font-medium text-gray-900">最大内存</p>
              <span class="text-sm font-bold text-primary-600">{{ formatMemory(maxMemory) }}</span>
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
              <span>{{ formatMemory(totalMemoryMB > 0 ? Math.min(totalMemoryMB, 16384) : 8192) }}</span>
            </div>
          </div>
          <div>
            <div class="flex items-center justify-between mb-2">
              <p class="text-sm font-medium text-gray-900">最小内存</p>
              <span class="text-sm font-bold text-gray-600">{{ formatMemory(minMemory) }}</span>
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
              <span>{{ formatMemory(maxMemory) }}</span>
            </div>
          </div>
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
            <p class="text-xs text-gray-500 mt-0.5">Minecraft 游戏数据存放位置</p>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-sm text-gray-600 max-w-xs truncate">{{ gameDir }}</span>
            <button
              class="px-3 py-1.5 text-sm font-medium bg-gray-100 border border-gray-300 rounded-lg hover:bg-gray-200 transition-colors flex items-center"
              @click="handleSelectFolder"
            >
              <FolderOpenIcon class="w-4 h-4 mr-1" />
              更改
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
