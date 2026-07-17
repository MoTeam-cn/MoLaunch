<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { useSdkStore } from '@/stores/sdk'
import Select from '@/components/common/Select.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import * as tauri from '@/utils/tauri'
import { getConfigMap, applyConfig } from '@/utils/api/config'
import { showInfo, showSuccess } from '@/utils/toast'

const sdkStore = useSdkStore()
const logLevel = ref(3)
const configPath = ref('')

// 应用版本（从 package.json 注入，由 vite define 提供；点击 5 次解锁开发者模式）
const appVersion = __APP_VERSION__

// 开发者模式解锁状态
const devUnlocked = ref(false)
const versionClickCount = ref(0)
let versionClickTimer: ReturnType<typeof setTimeout> | null = null

// 读取日志级别（统一走 getConfigMap，避免使用调试用 getConfigValue）
async function loadLogLevel() {
  try {
    const cfg = await getConfigMap()
    if (typeof cfg.logLevel === 'number') {
      logLevel.value = cfg.logLevel
    }
  } catch (e) {
    console.error('Failed to get log level:', e)
  }
}

// 保存日志级别（统一走 applyConfig，后端会同步调用 logger::set_level 立即生效）
async function saveLogLevel(level: number) {
  try {
    await applyConfig({ logLevel: level })
  } catch (e) {
    console.error('Failed to save log level:', e)
  }
}

// 监听日志级别变化
watch(logLevel, (newLevel) => {
  saveLogLevel(newLevel)
})

// 版本号点击：连续 5 次解锁开发者模式
async function onVersionClick() {
  if (devUnlocked.value) return

  versionClickCount.value++
  const remaining = 5 - versionClickCount.value

  if (versionClickCount.value >= 5) {
    // 解锁
    try {
      await tauri.unlockDeveloperMode()
      devUnlocked.value = true
      versionClickCount.value = 0
      showSuccess('已解锁开发者模式，可在「高阶配置」中开启')
    } catch (e) {
      console.error('Failed to unlock developer mode:', e)
      showError('解锁失败：' + e)
    }
    return
  }

  // 提示还需点击几次
  showInfo(`再点击 ${remaining} 次解锁开发者模式`)

  // 1.5 秒内未完成 5 次点击则重置计数器
  if (versionClickTimer) clearTimeout(versionClickTimer)
  versionClickTimer = setTimeout(() => {
    versionClickCount.value = 0
    versionClickTimer = null
  }, 1500)
}

onMounted(async () => {
  try {
    configPath.value = await tauri.getConfigPath()
  } catch (e) {
    console.error('Failed to get config path:', e)
    configPath.value = '获取失败'
  }
  await loadLogLevel()
  try {
    devUnlocked.value = await tauri.isDeveloperUnlocked()
  } catch (e) {
    console.error('Failed to check developer unlocked:', e)
  }
})
</script>

<template>
  <div class="space-y-6">
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

    <!-- 配置信息 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">配置信息</h3>
      <div class="divide-y divide-gray-200">
        <div class="px-5 py-3">
          <p class="text-sm text-gray-500 mb-1">配置文件路径</p>
          <p class="text-xs text-gray-900 font-mono bg-gray-50 px-3 py-2 rounded break-all">{{ configPath || '加载中...' }}</p>
        </div>
        <div
          class="px-5 py-3 flex items-center justify-between cursor-pointer select-none hover:bg-gray-50"
          @click="onVersionClick"
        >
          <span class="text-sm text-gray-500">应用版本</span>
          <Tooltip
            :text="devUnlocked ? '开发者模式已解锁' : '连续点击 5 次解锁开发者模式'"
            position="top"
            :delay="200"
          >
            <span class="text-sm text-gray-900 font-mono">v{{ appVersion }}</span>
          </Tooltip>
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
          <span class="text-sm text-gray-500">状态</span>
          <span class="text-sm" :class="sdkStore.isReady ? 'text-green-600' : 'text-yellow-600'">
            {{ sdkStore.isReady ? '就绪' : '加载中' }}
          </span>
        </div>
        <div class="px-5 py-3 flex items-center justify-between">
          <span class="text-sm text-gray-500">设备 ID</span>
          <span class="text-sm text-gray-900 font-mono">{{ sdkStore.deviceId ? sdkStore.deviceId.substring(0, 4) + '****' : '未获取' }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
