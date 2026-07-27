<script setup lang="ts">
/**
 * 设置 - 联机 Tab（阶段一）
 *
 * 控件：
 * - api-server 地址输入（防抖自动保存 800ms，走统一 applyConfig）
 * - 重置为默认按钮（一键还原官方地址）
 * - 测试连通性按钮（调用 auth_get_server_time，显示服务器时间/时区/偏移/时间差）
 * - ICE 服务器配置：用户自定义 TURN 服务器列表（阶段三子任务 7 阶段 H）
 * - 设备登出 / 清除凭证按钮（调用 online store，复用其 toast 提示）
 *
 * 复用：
 * - useConfigPage：与 SettingsAdvanced 一致的加载 + 防抖保存 + loaded 守卫
 * - useOnlineStore：deviceStatus + logout/clear + setCustomTurnServers（与 Online.vue 共享状态）
 * - showConfirm：全局 Modal 服务（替代 window.confirm）
 * - formatTimestamp：utils/format 中新增的时间戳格式化函数
 */
import { ref, watch, computed, onMounted } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { getServerTime } from '@/utils/api/online-manager'
import { useConfigPage } from '@/composables/useConfigPage'
import { showConfirm } from '@/utils/modal'
import { toastSuccess } from '@/utils/toast'
import { formatTimestamp } from '@/utils/format'
import Input from '@/components/common/Input.vue'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import TurnServerEntryEditor from '@/components/online/TurnServerEntryEditor.vue'
import {
  GlobeAltIcon,
  ArrowPathIcon,
  ArrowRightOnRectangleIcon,
  TrashIcon,
  CheckCircleIcon,
  ExclamationTriangleIcon,
  PlusIcon,
  ServerStackIcon,
} from '@heroicons/vue/24/outline'
import type { IceServerEntry } from '@/types/online'

/** 官方默认 api-server 地址（与后端 OnlineConfig::default 一致） */
const DEFAULT_API_SERVER_URL = 'https://api.molaunch.moiu.cn'

const onlineStore = useOnlineStore()

// ============ api-server 地址（防抖自动保存） ============
const apiUrl = ref('')

const { loaded, markDirty, flushSave } = useConfigPage({
  delay: 800,
  errorLabel: 'save online settings',
  onLoad: (cfg) => {
    apiUrl.value = cfg.onlineApiServerUrl
    // 阶段三子任务 7：加载用户自定义 TURN 列表（默认空数组）
    customTurnServers.value = cfg.onlineCustomTurnServers ?? []
  },
})

watch(apiUrl, (v) => markDirty('onlineApiServerUrl', v))

function handleResetToDefault() {
  apiUrl.value = DEFAULT_API_SERVER_URL
  toastSuccess('已重置为默认地址（自动保存）')
}

// ============ ICE 服务器配置（阶段三子任务 7 阶段 H） ============
const customTurnServers = ref<IceServerEntry[]>([])

// watch deep：任一条目字段变更触发防抖保存 + store 同步（store 为房主创建房间的运行时数据源）
watch(
  customTurnServers,
  (v) => {
    markDirty('onlineCustomTurnServers', v)
    onlineStore.setCustomTurnServers(v)
  },
  { deep: true },
)

function handleAddTurnServer() {
  customTurnServers.value.push({ urls: [], username: undefined, credential: undefined })
}

function handleRemoveTurnServer(index: number) {
  customTurnServers.value.splice(index, 1)
}

function handleUpdateTurnServer(index: number, entry: IceServerEntry) {
  customTurnServers.value[index] = entry
}

// ============ 测试连通性 ============
const testing = ref(false)
interface TestResult {
  success: boolean
  serverTime?: number
  timezone?: string
  offsetSeconds?: number
  message?: string
}
const testResult = ref<TestResult | null>(null)

async function handleTestConnection() {
  // 先 flush 保存，确保后端用的是最新 URL
  flushSave()
  testing.value = true
  testResult.value = null
  try {
    const info = await getServerTime()
    testResult.value = {
      success: true,
      serverTime: info.server_time,
      timezone: info.timezone,
      offsetSeconds: info.offset_seconds,
    }
  } catch (e) {
    testResult.value = {
      success: false,
      message: e instanceof Error ? e.message : String(e),
    }
  } finally {
    testing.value = false
  }
}

/** 偏移秒数 → UTC+08:00 格式 */
function formatOffset(seconds: number): string {
  const sign = seconds >= 0 ? '+' : '-'
  const abs = Math.abs(seconds)
  const h = Math.floor(abs / 3600)
  const m = Math.floor((abs % 3600) / 60)
  return `UTC${sign}${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}`
}

/** 本地与服务器时间差（秒级比较，<60s 视为同步） */
const timeDiff = computed(() => {
  if (!testResult.value?.serverTime) return null
  const diff = Math.floor(Date.now() / 1000) - testResult.value.serverTime
  const abs = Math.abs(diff)
  if (abs < 60) return '本地时间与服务器同步'
  const min = Math.floor(abs / 60)
  return `本地时间比服务器${diff > 0 ? '快' : '慢'}约 ${min} 分钟`
})

// ============ 设备登出 / 清除凭证 ============
async function handleLogout() {
  await onlineStore.logout()
}

function handleClearCredentials() {
  showConfirm(
    '清除设备凭证',
    '此操作将删除本地密钥与 JWT，需要重新注册设备才能继续使用联机功能。\n是否继续？',
    async () => {
      await onlineStore.clear()
    },
  )
}

// 进入页面时拉取设备状态（用于登出/清除按钮的 disabled 判断）
onMounted(() => {
  void onlineStore.refreshStatus()
})
</script>

<template>
  <div class="space-y-6">
    <!-- 加载占位 -->
    <div v-if="!loaded" class="space-y-6">
      <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
        <div class="px-5 py-5">
          <div class="h-4 w-24 bg-gray-200 rounded animate-pulse mb-4" />
          <div class="h-10 bg-gray-100 rounded animate-pulse" />
        </div>
      </div>
    </div>

    <template v-else>
      <!-- api-server 配置 -->
      <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
        <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">api-server 配置</h3>
        <div class="divide-y divide-gray-200">
          <!-- 服务器地址 -->
          <div class="px-5 py-4">
            <div class="flex items-center justify-between mb-2">
              <div>
                <p class="text-sm font-medium text-gray-900">服务器地址</p>
                <p class="text-xs text-gray-500 mt-0.5">联机 api-server 基础 URL，修改后自动保存（800ms 防抖）</p>
              </div>
              <Tooltip text="重置为官方默认地址">
                <Button type="ghost" size="small" @click="handleResetToDefault">
                  <template #icon><ArrowPathIcon class="w-4 h-4" /></template>
                  重置
                </Button>
              </Tooltip>
            </div>
            <Input v-model="apiUrl" placeholder="https://api.molaunch.moiu.cn" class="font-mono" />
            <div class="mt-2 flex items-center gap-2 flex-wrap">
              <Button type="outline" size="small" :loading="testing" @click="handleTestConnection">
                <template #icon><GlobeAltIcon class="w-4 h-4" /></template>
                测试连通性
              </Button>
              <span v-if="apiUrl === DEFAULT_API_SERVER_URL" class="text-xs text-gray-400">（默认地址）</span>
            </div>
          </div>

          <!-- 测试结果 -->
          <div v-if="testResult" class="px-5 py-4">
            <!-- 成功 -->
            <div v-if="testResult.success" class="space-y-2">
              <div class="flex items-center gap-1.5 text-xs text-green-600">
                <CheckCircleIcon class="w-4 h-4" />
                <span>连接成功</span>
              </div>
              <div class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 text-xs">
                <div class="text-gray-500">服务器时间</div>
                <div class="text-gray-900 font-mono">{{ formatTimestamp(testResult.serverTime!) }}</div>
                <div class="text-gray-500">时区</div>
                <div class="text-gray-900">
                  {{ testResult.timezone }}
                  <span class="text-gray-500">({{ formatOffset(testResult.offsetSeconds!) }})</span>
                </div>
                <div v-if="timeDiff" class="text-gray-500">时间偏差</div>
                <div v-if="timeDiff" class="text-gray-900">{{ timeDiff }}</div>
              </div>
            </div>
            <!-- 失败 -->
            <div v-else class="space-y-1">
              <div class="flex items-center gap-1.5 text-xs text-red-600">
                <ExclamationTriangleIcon class="w-4 h-4" />
                <span>连接失败</span>
              </div>
              <p class="text-xs text-gray-600 break-all">{{ testResult.message }}</p>
            </div>
          </div>
        </div>
      </div>

      <!-- ICE 服务器配置（阶段三子任务 7 阶段 H） -->
      <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
        <div class="px-5 pt-5 pb-3 flex items-center justify-between">
          <div>
            <h3 class="text-sm font-semibold text-gray-900">ICE 服务器配置</h3>
            <p class="text-xs text-gray-500 mt-0.5">
              自定义 TURN 服务器（P2P 直连失败时的中转备用方案），自动保存（800ms 防抖）
            </p>
          </div>
        </div>
        <div class="divide-y divide-gray-200">
          <!-- 空状态：icon + text 垂直水平居中 -->
          <div
            v-if="customTurnServers.length === 0"
            class="flex flex-col items-center justify-center py-8"
          >
            <ServerStackIcon class="w-8 h-8 text-gray-300 mb-2" />
            <p class="text-xs text-gray-500">未配置自定义 TURN 服务器</p>
            <p class="text-xs text-gray-400 mt-1">系统将仅使用 STUN + 官方 TURN（如可用）</p>
          </div>
          <!-- TURN 服务器条目列表 -->
          <TurnServerEntryEditor
            v-for="(entry, idx) in customTurnServers"
            :key="idx"
            :model-value="entry"
            :index="idx"
            @update:model-value="(v) => handleUpdateTurnServer(idx, v)"
            @remove="handleRemoveTurnServer(idx)"
          />
          <!-- 添加按钮 -->
          <div class="px-5 py-3">
            <Button type="ghost" size="small" long @click="handleAddTurnServer">
              <template #icon><PlusIcon class="w-4 h-4" /></template>
              添加 TURN 服务器
            </Button>
          </div>
        </div>
      </div>

      <!-- 设备管理 -->
      <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
        <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">设备管理</h3>
        <div class="divide-y divide-gray-200">
          <!-- 登出 -->
          <div class="px-5 py-4 flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">登出设备</p>
              <p class="text-xs text-gray-500 mt-0.5">撤销当前 JWT，保留本地密钥，可重新登录</p>
            </div>
            <Button
              type="outline"
              size="small"
              :loading="onlineStore.loading"
              :disabled="!onlineStore.deviceStatus?.logged_in"
              @click="handleLogout"
            >
              <template #icon><ArrowRightOnRectangleIcon class="w-4 h-4" /></template>
              登出
            </Button>
          </div>
          <!-- 清除凭证 -->
          <div class="px-5 py-4 flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">清除设备凭证</p>
              <p class="text-xs text-gray-500 mt-0.5">删除本地密钥与 JWT，需要重新注册设备</p>
            </div>
            <Button
              type="outline"
              size="small"
              :loading="onlineStore.loading"
              :disabled="!onlineStore.deviceStatus?.registered"
              @click="handleClearCredentials"
            >
              <template #icon><TrashIcon class="w-4 h-4" /></template>
              清除
            </Button>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
