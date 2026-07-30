<script setup lang="ts">
/**
 * 系统信息子页签：应用版本 + 开发者模式解锁 + SDK 信息
 *
 * 迁移自 SettingsOther.vue（已删除）：移除配置文件路径展示，
 * 保留版本号 5 次点击解锁开发者模式与 SDK 状态展示。
 *
 * 开发者模式解锁触发点（任选其一，互不冲突）：
 * 1. 连续点击应用版本号 5 次（1.5 秒内完成）
 * 2. 连续双击设备 ID 行 5 次（4 秒内完成）
 * 第二种方式作为备用触发，避免单一触发点失效时无法解锁。
 */
import { ref, onMounted } from 'vue'
import { useSdkStore } from '@/stores/sdk'
import Tooltip from '@/components/common/Tooltip.vue'
import Alert from '@/components/common/Alert.vue'
import * as tauri from '@/utils/tauri'
import { toastInfo, toastSuccess } from '@/utils/toast'
import { safeCall } from '@/utils/async'

const sdkStore = useSdkStore()

// 应用版本（从 package.json 注入，由 vite define 提供；点击 5 次解锁开发者模式）
const appVersion = __APP_VERSION__

// 开发者模式解锁状态
const devUnlocked = ref(false)
const versionClickCount = ref(0)
let versionClickTimer: ReturnType<typeof setTimeout> | null = null

// 设备 ID 双击解锁：连续双击 5 次（4 秒内完成）
const deviceIdDblClickCount = ref(0)
let deviceIdDblClickTimer: ReturnType<typeof setTimeout> | null = null

// 设备 ID 显示状态：默认打码，双击切换全额显示
// 设备 ID 用于本地数据加密存储，请勿泄露
const deviceIdRevealed = ref(false)

function onDeviceIdDblClick() {
  // 已解锁状态下双击仅切换全额显示
  if (devUnlocked.value) {
    if (!sdkStore.deviceId) return
    deviceIdRevealed.value = !deviceIdRevealed.value
    return
  }

  // 未解锁状态下双击累计 5 次解锁
  deviceIdDblClickCount.value++
  const remaining = 5 - deviceIdDblClickCount.value

  if (deviceIdDblClickCount.value >= 5) {
    unlockDevMode()
    deviceIdDblClickCount.value = 0
    return
  }

  toastInfo(`再双击 ${remaining} 次解锁开发者模式`)

  // 4 秒内未完成 5 次双击则重置计数器（双击间隔较长，给用户更宽裕的时间）
  if (deviceIdDblClickTimer) clearTimeout(deviceIdDblClickTimer)
  deviceIdDblClickTimer = setTimeout(() => {
    deviceIdDblClickCount.value = 0
    deviceIdDblClickTimer = null
  }, 4000)
}

// 计算设备 ID 显示值（打码 / 全额）
function getDeviceIdDisplay(): string {
  const id = sdkStore.deviceId
  if (!id) return '未获取'
  // 去除 mcsdk- 前缀后展示
  const display = id.startsWith('mcsdk-') ? id.slice(6) : id
  if (deviceIdRevealed.value) return display
  return display.length > 8 ? display.substring(0, 4) + '****' + display.substring(display.length - 4) : '****'
}

// 版本号点击：连续 5 次解锁开发者模式
async function onVersionClick() {
  if (devUnlocked.value) return

  versionClickCount.value++
  const remaining = 5 - versionClickCount.value

  if (versionClickCount.value >= 5) {
    await unlockDevMode()
    versionClickCount.value = 0
    return
  }

  // 提示还需点击几次
  toastInfo(`再点击 ${remaining} 次解锁开发者模式`)

  // 1.5 秒内未完成 5 次点击则重置计数器
  if (versionClickTimer) clearTimeout(versionClickTimer)
  versionClickTimer = setTimeout(() => {
    versionClickCount.value = 0
    versionClickTimer = null
  }, 1500)
}

/** 调用后端解锁开发者模式（共用函数，版本号点击和设备 ID 双击共用） */
async function unlockDevMode() {
  const ok = await safeCall(() => tauri.unlockDeveloperMode(), 'unlock developer mode')
  if (ok !== undefined) {
    devUnlocked.value = true
    toastSuccess('已解锁开发者模式，可在「进阶设置」中开启')
  }
}

onMounted(async () => {
  const unlocked = await safeCall(() => tauri.isDeveloperUnlocked(), 'check developer unlocked')
  if (unlocked) devUnlocked.value = unlocked
})
</script>

<template>
  <div class="space-y-6">
    <!-- 应用版本 + 开发者模式解锁入口 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">应用信息</h3>
      <div class="divide-y divide-gray-200">
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
        <div
          class="px-5 py-3 flex items-center justify-between cursor-pointer select-none hover:bg-gray-50"
          @dblclick="onDeviceIdDblClick"
        >
          <div class="flex items-center gap-2">
            <span class="text-sm text-gray-500">设备 ID</span>
            <Tooltip
              :text="devUnlocked
                ? '双击切换全额显示 / 打码'
                : '连续双击 5 次解锁开发者模式（备用入口）'"
              position="top"
              :delay="200"
            >
              <svg class="w-3.5 h-3.5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </Tooltip>
          </div>
          <span
            class="text-sm font-mono"
            :class="deviceIdRevealed ? 'text-primary-700' : 'text-gray-900'"
          >{{ getDeviceIdDisplay() }}</span>
        </div>
      </div>
      <!-- 设备 ID 防泄露常驻提示（仅当全额显示时显示，切到其他页面时随组件卸载自动消失） -->
      <div v-if="deviceIdRevealed" class="mx-5 mb-4 mt-2">
        <Alert
          type="warning"
          :truncate="false"
          message="设备 ID 已全额显示。本 ID 用于本地数据加密存储，请勿截图外传或泄露给他人，否则可能导致本地账号、配置等敏感数据被还原。"
        />
      </div>
    </div>
  </div>
</template>
