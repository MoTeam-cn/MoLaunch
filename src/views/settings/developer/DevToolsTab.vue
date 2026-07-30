<script setup lang="ts">
/**
 * 开发者 - DevTools 子页签
 *
 * 在开发者模式开启时提供「打开/关闭 WebView2 DevTools」按钮。
 * 后端 open_devtools action 内部校验 is_developer_unlocked && is_developer_mode，
 * 普通用户无法绕过此按钮直接调用 IPC。
 *
 * 使用方式：在 SettingsDeveloper.vue 顶部 subTabs 中追加 'devtools' 子页签
 */
import { ref, onMounted } from 'vue'
import Button from '@/components/common/Button.vue'
import Alert from '@/components/common/Alert.vue'
import {
  closeDevTools,
  isDevToolsOpen,
  openDevTools,
} from '@/utils/api/developer'
import { toastError, toastSuccess } from '@/utils/toast'
import { safeCall } from '@/utils/async'
import {
  CommandLineIcon,
  EyeIcon,
  EyeSlashIcon,
} from '@heroicons/vue/24/outline'

const isOpen = ref(false)
const loading = ref(false)

async function refreshState() {
  const r = await safeCall(() => isDevToolsOpen(), 'query devtools state')
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

onMounted(refreshState)
</script>

<template>
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
</template>
