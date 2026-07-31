<script setup lang="ts">
/**
 * 开发者 - DevTools 子页签
 *
 * 在开发者模式开启时提供「打开/关闭 WebView2 DevTools」按钮。
 * 后端 open_devtools action 内部校验 is_developer_unlocked && is_developer_mode，
 * 普通用户无法绕过此按钮直接调用 IPC。
 *
 * 附加功能：测试版水印隐藏解锁
 * - 测试版构建时全屏显示水印（追溯泄漏源），开发者调试时可临时隐藏
 * - 隐藏前提：DevTools 已打开（后端 AtomicBool 维护状态）
 * - 解锁状态存 sessionStorage，重启后恢复；DevTools 关闭自动恢复水印
 */
import { ref, onMounted, computed } from 'vue'
import Button from '@/components/common/Button.vue'
import Alert from '@/components/common/Alert.vue'
import {
  closeDevTools,
  isDevToolsOpen,
  openDevTools,
} from '@/utils/api/developer'
import { toastError, toastSuccess, toastInfo } from '@/utils/toast'
import { safeCall } from '@/utils/async'
import { useWatermarkUnlock } from '@/composables/useWatermarkUnlock'
import { isPreReleaseBuild } from '@/utils/version'
import {
  CommandLineIcon,
  EyeIcon,
  EyeSlashIcon,
  EyeDropperIcon,
  ArrowPathIcon,
} from '@heroicons/vue/24/outline'

const isOpen = ref(false)
const loading = ref(false)
const watermarkLoading = ref(false)

const { unlocked: watermarkUnlocked, hide: hideWatermark, show: showWatermark } = useWatermarkUnlock()

/** 是否为测试版构建（仅测试版显示水印解锁卡片） */
const isPreRelease = computed(() => isPreReleaseBuild())

async function refreshState() {
  const r = await safeCall(() => isDevToolsOpen(), 'query devtools state', () => toastError('查询 DevTools 状态失败'))
  if (typeof r === 'boolean') isOpen.value = r
}

async function onToggle() {
  if (loading.value) return
  loading.value = true
  try {
    if (isOpen.value) {
      await closeDevTools()
      isOpen.value = false
      toastSuccess('DevTools 已关闭')
    } else {
      await openDevTools()
      isOpen.value = true
      toastSuccess('DevTools 已打开')
    }
  } catch (e) {
    toastError('DevTools 操作失败：' + e)
  } finally {
    loading.value = false
  }
}

/** 隐藏水印（需 DevTools 已打开） */
async function onHideWatermark() {
  if (watermarkLoading.value) return
  watermarkLoading.value = true
  try {
    await hideWatermark()
    toastSuccess('水印已隐藏（DevTools 关闭后自动恢复）')
  } catch (e) {
    toastError('隐藏水印失败：' + e)
  } finally {
    watermarkLoading.value = false
  }
}

/** 恢复水印显示 */
function onShowWatermark() {
  showWatermark()
  toastInfo('水印已恢复显示')
}

onMounted(refreshState)
</script>

<template>
  <div class="space-y-6">
    <!-- DevTools 控制 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3 flex items-center gap-2">
        <CommandLineIcon class="w-4 h-4 text-gray-500" />
        WebView2 开发者工具
      </h3>

      <div class="mx-5 mb-4">
        <Alert
          type="info"
          :truncate="false"
          message="通过此按钮可调出 WebView2 DevTools，用于排查前端问题。开发者模式关闭后此按钮无效（后端拒绝 IPC 调用）。"
        />
      </div>

      <div class="divide-y divide-gray-200">
        <div class="px-5 py-4">
          <div class="flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">DevTools 状态</p>
              <p class="text-xs text-gray-500 mt-0.5">
                {{ isOpen ? '已打开' : '已关闭' }}
              </p>
            </div>
            <Button
              :type="isOpen ? 'secondary' : 'primary'"
              :loading="loading"
              @click="onToggle"
            >
              <template #icon>
                <EyeSlashIcon v-if="isOpen" class="h-4 w-4" />
                <EyeIcon v-else class="h-4 w-4" />
              </template>
              {{ isOpen ? '关闭 DevTools' : '打开 DevTools' }}
            </Button>
          </div>
        </div>
      </div>
    </div>

    <!-- 水印解锁（仅测试版构建显示） -->
    <div v-if="isPreRelease" class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3 flex items-center gap-2">
        <EyeDropperIcon class="w-4 h-4 text-gray-500" />
        测试版水印
      </h3>

      <div class="mx-5 mb-4">
        <Alert
          type="info"
          :truncate="false"
          message="测试版构建会全屏显示追溯水印（含设备 ID 与屏印哈希）。开发者调试时可在 DevTools 打开的前提下临时隐藏，DevTools 关闭后自动恢复。"
        />
      </div>

      <div class="divide-y divide-gray-200">
        <div class="px-5 py-4">
          <div class="flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">水印显示状态</p>
              <p class="text-xs text-gray-500 mt-0.5">
                <template v-if="watermarkUnlocked">已隐藏（DevTools 关闭后自动恢复）</template>
                <template v-else>显示中（默认）</template>
              </p>
            </div>
            <Button
              v-if="!watermarkUnlocked"
              type="secondary"
              :loading="watermarkLoading"
              @click="onHideWatermark"
            >
              <template #icon>
                <EyeSlashIcon class="h-4 w-4" />
              </template>
              隐藏水印
            </Button>
            <Button
              v-else
              type="secondary"
              @click="onShowWatermark"
            >
              <template #icon>
                <ArrowPathIcon class="h-4 w-4" />
              </template>
              恢复水印
            </Button>
          </div>
          <p v-if="!isOpen && !watermarkUnlocked" class="text-xs text-amber-600 mt-2">
            需先打开 DevTools 才能隐藏水印
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
