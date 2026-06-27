<script setup lang="ts">
/**
 * 设置页面
 * 左侧分类菜单，右侧设置内容，修改即自动保存
 */

import { ref, computed, watch, onMounted } from 'vue'
import { useSdkStore } from '@/stores/sdk'
import { useSettingsStore } from '@/stores/settings'
import * as tauri from '@/utils/tauri'
import { showInfo, showSuccess } from '@/utils/toast'
import Select from '@/components/common/Select.vue'
import {
  RocketLaunchIcon,
  PaintBrushIcon,
  EllipsisHorizontalIcon,
  FolderOpenIcon,
  ArrowDownTrayIcon,
} from '@heroicons/vue/24/outline'

const sdkStore = useSdkStore()
const settingsStore = useSettingsStore()

const gameDir = ref('')
const maxThreads = ref(8)
const minMemory = ref(512)
const maxMemory = ref(2048)
const logLevel = ref(3)
const loaded = ref(false)
const systemMemory = ref<{ total: number; available: number; usage_percent: number } | null>(null)
const memoryMode = ref<'auto' | 'custom'>('auto')
const showJavaList = ref(false)
const javaSelectorRef = ref<HTMLElement | null>(null)
const mirrorMeta = ref<'official' | 'bmclapi'>('bmclapi')
const mirrorDownload = ref<'official' | 'bmclapi'>('bmclapi')
const maxDownloadSpeed = ref(0)
const speedSlider = ref(0)

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

const activeCategory = ref('launch')

const categories = [
  { id: 'launch', label: '游戏启动', icon: RocketLaunchIcon },
  { id: 'download', label: '下载配置', icon: ArrowDownTrayIcon },
  { id: 'personal', label: '个性化', icon: PaintBrushIcon },
  { id: 'other', label: '其他', icon: EllipsisHorizontalIcon },
]

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

// 内存条各段宽度百分比
const usedPercent = computed(() => totalMemoryMB.value > 0 ? (usedMemoryMB.value / totalMemoryMB.value) * 100 : 0)
const gamePercent = computed(() => totalMemoryMB.value > 0 ? (gameMemoryMB.value / totalMemoryMB.value) * 100 : 0)

function applyAutoMemory() {
  if (!systemMemory.value) return
  const availableMB = systemMemory.value.available / 1024 / 1024
  // 自动分配：可用内存的 75%，上限 8GB
  const suggested = Math.min(Math.round(availableMB * 0.75), 8192)
  maxMemory.value = Math.max(suggested, 512)
  minMemory.value = Math.round(maxMemory.value / 2)
}

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
    const source = await tauri.getDownloadSource()
    if (source === 'mirror') {
      mirrorMeta.value = 'bmclapi'
      mirrorDownload.value = 'bmclapi'
    } else if (source === 'official') {
      mirrorMeta.value = 'official'
      mirrorDownload.value = 'official'
    } else if (source === 'smart') {
      mirrorMeta.value = 'smart'
      mirrorDownload.value = 'smart'
    }
  } catch {
    // ignore
  }
  try {
    maxDownloadSpeed.value = await tauri.getMaxDownloadSpeed()
    // 0 = 不限制(显示21), 否则转为MB/s
    if (maxDownloadSpeed.value === 0) {
      speedSlider.value = 21
    } else {
      speedSlider.value = Math.round(maxDownloadSpeed.value / 1024 / 1024)
    }
  } catch {
    // ignore
  }
  loaded.value = true
})

watch(memoryMode, (mode) => {
  if (mode === 'auto') applyAutoMemory()
})

watch([mirrorMeta, mirrorDownload], async ([meta, dl]) => {
  if (!loaded.value) return
  // 映射到后端 download_source 模式
  // mirror: 两个都是 bmclapi
  // official: 两个都是 official
  // smart: meta=official, download=bmclapi
  // 其他组合: 根据各自的值分别设置 meta 和 download URL
  let source = 'official'
  if (meta === 'bmclapi' && dl === 'bmclapi') source = 'mirror'
  else if (meta === 'official' && dl === 'bmclapi') source = 'smart'
  // 如果用户选了不同的组合，用 smart 模式（官方优先+镜像兜底）
  else if (meta === 'smart' || dl === 'smart') source = 'smart'
  try {
    await tauri.setDownloadSource(source)
  } catch (e) {
    console.error('Failed to set download source:', e)
  }
})

watch(speedSlider, async (val) => {
  if (!loaded.value) return
  // 0 = 1MB/s, 1-20 = 对应MB/s, 21 = 不限制
  const speed = val >= 21 ? 0 : (val === 0 ? 1 : val) * 1024 * 1024
  maxDownloadSpeed.value = speed
  try {
    await tauri.setMaxDownloadSpeed(speed)
  } catch (e) {
    console.error('Failed to set max download speed:', e)
  }
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

let saveTimer: ReturnType<typeof setTimeout> | null = null
function autoSave() {
  if (!loaded.value) return
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => {
    // TODO: 持久化游戏设置到后端
  }, 500)
}

watch([gameDir, maxThreads, minMemory, maxMemory, logLevel], autoSave)

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

function handleReset() {
  gameDir.value = '.minecraft'
  maxThreads.value = 8
  minMemory.value = 512
  maxMemory.value = 2048
  logLevel.value = 3
  sdkStore.javaPath = ''
  settingsStore.setTheme('system')
}
</script>

<template>
  <div class="flex h-full rounded-xl overflow-hidden bg-white shadow-sm">
    <!-- 左侧分类菜单 -->
    <aside class="w-48 bg-white border-r border-gray-200 flex flex-col shrink-0">
      <div class="flex-1 overflow-y-auto py-4">
        <button
          v-for="cat in categories"
          :key="cat.id"
          class="w-full flex items-center px-4 py-2.5 text-sm font-medium transition-colors"
          :class="[
            activeCategory === cat.id
              ? 'bg-primary-50 text-primary-700 border-r-2 border-primary-500'
              : 'text-gray-700 hover:bg-gray-50'
          ]"
          @click="activeCategory = cat.id"
        >
          <component :is="cat.icon" class="w-5 h-5 mr-3" />
          {{ cat.label }}
        </button>
      </div>

      <div class="p-3 border-t border-gray-200">
        <button
          class="w-full flex items-center justify-center px-3 py-2 text-xs text-gray-600 hover:bg-gray-50 rounded-lg transition-colors"
          @click="handleReset"
        >
          重置默认
        </button>
      </div>
    </aside>

    <!-- 右侧内容区 -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <div class="px-6 py-4 bg-white border-b border-gray-200 shrink-0">
        <h2 class="text-lg font-semibold text-gray-900">
          {{ categories.find(c => c.id === activeCategory)?.label }}
        </h2>
        <p class="text-xs text-gray-500 mt-1">
          {{ activeCategory === 'launch' ? 'Java、内存、游戏目录等启动参数' : activeCategory === 'download' ? '下载源、限速、并发等下载配置' : activeCategory === 'personal' ? '主题、布局、语言等外观设置' : '日志、SDK 信息' }}
        </p>
      </div>

      <!-- 设置内容 -->
      <div class="flex-1 overflow-y-auto p-6">

        <!-- ===== 游戏启动 ===== -->
        <div v-if="activeCategory === 'launch'" class="space-y-6">

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
                      class="px-2 py-1 text-xs text-gray-600 bg-gray-100 hover:bg-gray-200 rounded transition-colors"
                      title="自动检测"
                      @click="handleAutoDetectJava"
                    >
                      自动检测
                    </button>
                    <button
                      class="px-2 py-1 text-xs text-gray-600 bg-gray-100 hover:bg-gray-200 rounded transition-colors"
                      title="手动导入"
                      @click="handleManualImportJava"
                    >
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
                <!-- 系统已用 -->
                <div
                  class="h-full bg-orange-400 transition-all duration-500"
                  :style="{ width: usedPercent + '%' }"
                  :title="'系统已用: ' + formatMemory(usedMemoryMB)"
                />
                <!-- 游戏分配 -->
                <div
                  class="h-full bg-primary-500 transition-all duration-500"
                  :style="{ width: gamePercent + '%' }"
                  :title="'游戏分配: ' + formatMemory(gameMemoryMB)"
                />
                <!-- 剩余 -->
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

        <!-- ===== 个性化 ===== -->
        <div v-else-if="activeCategory === 'personal'" class="space-y-6">

          <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
            <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">外观</h3>
            <div class="divide-y divide-gray-200">
              <!-- 主题 -->
              <div class="px-5 py-4 flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-gray-900">主题</p>
                  <p class="text-xs text-gray-500 mt-0.5">选择启动器的颜色方案</p>
                </div>
                <div class="flex gap-2">
                  <button
                    v-for="opt in [
                      { value: 'light' as const, label: '浅色' },
                      { value: 'system' as const, label: '跟随系统' },
                    ]"
                    :key="opt.value"
                    class="px-4 py-1.5 text-sm font-medium rounded-lg border transition-colors"
                    :class="settingsStore.theme === opt.value
                      ? 'border-primary-500 bg-primary-50 text-primary-700'
                      : 'border-gray-300 text-gray-600 hover:border-gray-400'"
                    @click="settingsStore.setTheme(opt.value)"
                  >
                    {{ opt.label }}
                  </button>
                </div>
              </div>
              <!-- 语言 -->
              <div class="px-5 py-4 flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-gray-900">语言</p>
                  <p class="text-xs text-gray-500 mt-0.5">启动器界面语言</p>
                </div>
                <div class="flex gap-2">
                  <button
                    v-for="opt in [
                      { value: 'zh-CN' as const, label: '简体中文' },
                      { value: 'en-US' as const, label: 'English' },
                    ]"
                    :key="opt.value"
                    class="px-4 py-1.5 text-sm font-medium rounded-lg border transition-colors"
                    :class="settingsStore.language === opt.value
                      ? 'border-primary-500 bg-primary-50 text-primary-700'
                      : 'border-gray-300 text-gray-600 hover:border-gray-400'"
                    @click="settingsStore.language = opt.value"
                  >
                    {{ opt.label }}
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- ===== 下载配置 ===== -->
        <div v-else-if="activeCategory === 'download'" class="space-y-6">

          <!-- 下载源 -->
          <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
            <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">下载源</h3>
            <div class="divide-y divide-gray-200">
              <!-- 版本列表源 -->
              <div class="px-5 py-4">
                <div class="flex items-center justify-between mb-2">
                  <div>
                    <p class="text-sm font-medium text-gray-900">版本列表源</p>
                    <p class="text-xs text-gray-500 mt-0.5">版本清单、Forge/Fabric 等加载器列表的获取源</p>
                  </div>
                </div>
                <div class="flex gap-2">
                  <button
                    class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
                    :class="mirrorMeta === 'official'
                      ? 'border-primary-500 bg-primary-50 text-primary-700'
                      : 'border-gray-200 text-gray-600 hover:border-gray-300'"
                    @click="mirrorMeta = 'official'"
                  >
                    官方源
                  </button>
                  <button
                    class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
                    :class="mirrorMeta === 'bmclapi'
                      ? 'border-primary-500 bg-primary-50 text-primary-700'
                      : 'border-gray-200 text-gray-600 hover:border-gray-300'"
                    @click="mirrorMeta = 'bmclapi'"
                  >
                    BMCLAPI
                  </button>
                  <button
                    class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
                    :class="mirrorMeta === 'smart'
                      ? 'border-primary-500 bg-primary-50 text-primary-700'
                      : 'border-gray-200 text-gray-600 hover:border-gray-300'"
                    @click="mirrorMeta = 'smart'"
                  >
                    优先官方
                  </button>
                </div>
                <p class="text-xs text-gray-400 mt-2">
                  <template v-if="mirrorMeta === 'official'">Mojang 官方源，海外快国内可能较慢</template>
                  <template v-else-if="mirrorMeta === 'bmclapi'">BMCLAPI 国内镜像，速度快</template>
                  <template v-else>优先请求官方源，超时后自动切换到镜像源</template>
                </p>
              </div>
              <!-- 文件下载源 -->
              <div class="px-5 py-4">
                <div class="flex items-center justify-between mb-2">
                  <div>
                    <p class="text-sm font-medium text-gray-900">文件下载源</p>
                    <p class="text-xs text-gray-500 mt-0.5">客户端 JAR、库文件、资源文件、加载器安装包</p>
                  </div>
                </div>
                <div class="flex gap-2">
                  <button
                    class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
                    :class="mirrorDownload === 'official'
                      ? 'border-primary-500 bg-primary-50 text-primary-700'
                      : 'border-gray-200 text-gray-600 hover:border-gray-300'"
                    @click="mirrorDownload = 'official'"
                  >
                    官方源
                  </button>
                  <button
                    class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
                    :class="mirrorDownload === 'bmclapi'
                      ? 'border-primary-500 bg-primary-50 text-primary-700'
                      : 'border-gray-200 text-gray-600 hover:border-gray-300'"
                    @click="mirrorDownload = 'bmclapi'"
                  >
                    BMCLAPI
                  </button>
                  <button
                    class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
                    :class="mirrorDownload === 'smart'
                      ? 'border-primary-500 bg-primary-50 text-primary-700'
                      : 'border-gray-200 text-gray-600 hover:border-gray-300'"
                    @click="mirrorDownload = 'smart'"
                  >
                    优先官方
                  </button>
                </div>
                <p class="text-xs text-gray-400 mt-2">
                  <template v-if="mirrorDownload === 'official'">Mojang 官方源，海外快国内可能较慢</template>
                  <template v-else-if="mirrorDownload === 'bmclapi'">BMCLAPI 国内镜像，速度快</template>
                  <template v-else>优先请求官方源，超时后自动切换到镜像源</template>
                </p>
              </div>
            </div>
          </div>

          <!-- 下载控制 -->
          <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
            <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">下载控制</h3>
            <div class="divide-y divide-gray-200">
              <!-- 下载线程数 -->
              <div class="px-5 py-4 flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-gray-900">下载线程数</p>
                  <p class="text-xs text-gray-500 mt-0.5">并发下载线程，数值越大下载越快</p>
                </div>
                <div class="flex items-center gap-3">
                  <input
                    v-model.number="maxThreads"
                    type="range"
                    class="w-32 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
                    min="1"
                    max="16"
                    step="1"
                  />
                  <span class="text-sm font-medium text-primary-600 w-6 text-right">{{ maxThreads }}</span>
                </div>
              </div>
              <!-- 下载限速 -->
              <div class="px-5 py-4">
                <div class="flex items-center justify-between mb-2">
                  <div>
                    <p class="text-sm font-medium text-gray-900">下载限速</p>
                    <p class="text-xs text-gray-500 mt-0.5">拖到最右边为不限制</p>
                  </div>
                  <span class="text-sm font-medium text-primary-600">
                    {{ speedSlider >= 21 ? '不限制' : speedSlider + ' MB/s' }}
                  </span>
                </div>
                <div class="flex items-center gap-3">
                  <span class="text-xs text-gray-400">1</span>
                  <input
                    v-model.number="speedSlider"
                    type="range"
                    class="flex-1 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
                    min="0"
                    max="21"
                    step="1"
                  />
                  <span class="text-xs text-gray-400">不限</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- ===== 其他 ===== -->
        <div v-else-if="activeCategory === 'other'" class="space-y-6">

          <!-- 系统设置 -->
          <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
            <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">系统</h3>
            <div class="divide-y divide-gray-200">
              <div class="px-5 py-4 flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-gray-900">日志级别</p>
                  <p class="text-xs text-gray-500 mt-0.5">控制日志输出的详细程度</p>
                </div>
                <Select
                  :model-value="logLevel"
                  :options="[
                    { label: '关闭', value: 0 },
                    { label: '错误', value: 1 },
                    { label: '警告', value: 2 },
                    { label: '信息', value: 3 },
                    { label: '调试', value: 4 },
                    { label: '跟踪', value: 5 },
                  ]"
                  style="min-width: 100px"
                  @update:model-value="logLevel = Number($event)"
                />
              </div>
            </div>
          </div>

          <!-- SDK 信息 -->
          <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
            <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">SDK 信息</h3>
            <div class="divide-y divide-gray-200">
              <div class="px-5 py-3 flex items-center justify-between">
                <span class="text-sm text-gray-500">平台</span>
                <span class="text-sm text-gray-900">{{ sdkStore.status?.platform || '未知' }}</span>
              </div>
              <div class="px-5 py-3 flex items-center justify-between">
                <span class="text-sm text-gray-500">版本</span>
                <span class="text-sm text-gray-900">{{ sdkStore.version || '未加载' }}</span>
              </div>
              <div class="px-5 py-3 flex items-center justify-between">
                <span class="text-sm text-gray-500">状态</span>
                <span class="text-sm" :class="sdkStore.isReady ? 'text-green-600' : 'text-yellow-600'">
                  {{ sdkStore.isReady ? '就绪' : '加载中' }}
                </span>
              </div>
              <div class="px-5 py-3 flex items-center justify-between">
                <span class="text-sm text-gray-500">设备 ID</span>
                <span class="text-sm text-gray-900 font-mono">{{ sdkStore.deviceId || '未获取' }}</span>
              </div>
              <div class="px-5 py-3 flex items-center justify-between">
                <span class="text-sm text-gray-500">库路径</span>
                <span class="text-sm text-gray-900 truncate ml-4 max-w-xs">{{ sdkStore.status?.library_path || '未知' }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.speed-slider {
  -webkit-appearance: none;
  appearance: none;
  height: 6px;
  background: #e5e7eb;
  border-radius: 3px;
  outline: none;
  cursor: pointer;
}
.speed-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #3b82f6;
  cursor: pointer;
  border: 2px solid #fff;
  box-shadow: 0 1px 3px rgba(0,0,0,0.2);
}
.speed-slider::-moz-range-thumb {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #3b82f6;
  cursor: pointer;
  border: 2px solid #fff;
  box-shadow: 0 1px 3px rgba(0,0,0,0.2);
}
</style>
