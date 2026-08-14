<script setup lang="ts">
/**
 * 开发者 - 深链接子页签
 *
 * molaunch:// 协议注册管理：
 * - 安装版由 NSIS 安装时自动注册（本页显示"已注册（指向当前程序）"）
 * - 便携版（未安装）无安装器，启动时会自动尝试注册；本页提供手动
 *   注册/卸载入口，供用户按需抉择（如卸载后协议链接将提示无应用处理）
 *
 * 后端由 system_manager 的 get_deeplink_status / register_deeplink
 * / unregister_deeplink 三个 action 支撑（utils::deeplink::protocol）。
 */
import { ref, onMounted, defineAsyncComponent } from 'vue'
import {
  getDeeplinkStatus,
  registerDeeplink,
  unregisterDeeplink,
  type DeeplinkStatus,
} from '@/utils/api/developer'
import { toastError, toastSuccess } from '@/utils/toast'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))

const status = ref<DeeplinkStatus | null>(null)
const loading = ref(false)

async function refresh() {
  try {
    status.value = await getDeeplinkStatus()
  } catch (e) {
    toastError('获取 deeplink 状态失败：' + e)
  }
}

async function handleRegister() {
  loading.value = true
  try {
    status.value = await registerDeeplink()
    toastSuccess('molaunch:// 协议已注册')
  } catch (e) {
    toastError('注册 deeplink 失败：' + e)
  } finally {
    loading.value = false
  }
}

async function handleUnregister() {
  loading.value = true
  try {
    status.value = await unregisterDeeplink()
    toastSuccess('molaunch:// 协议已卸载')
  } catch (e) {
    toastError('卸载 deeplink 失败：' + e)
  } finally {
    loading.value = false
  }
}

onMounted(refresh)
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">深链接（molaunch://）</h3>

    <div class="px-5 pb-5 space-y-4">
      <!-- 状态 -->
      <div v-if="status" class="space-y-3">
        <div class="flex items-start gap-2">
          <span
            class="mt-0.5 inline-flex items-center px-2 py-0.5 text-xs rounded-full"
            :class="
              status.registered
                ? 'bg-green-100 text-green-700'
                : 'bg-yellow-100 text-yellow-700'
            "
          >
            {{ status.registered ? '已注册' : '未注册' }}
          </span>
          <span class="text-sm text-gray-600">{{ status.message }}</span>
        </div>

        <div class="text-xs text-gray-500 space-y-1">
          <p class="flex items-center gap-1">
            <span class="w-28 shrink-0 text-gray-400">当前程序路径</span>
            <span class="truncate">{{ status.currentExe || '获取失败' }}</span>
          </p>
          <p class="flex items-center gap-1">
            <span class="w-28 shrink-0 text-gray-400">注册表登记路径</span>
            <span class="truncate">{{ status.registeredExe || '（无）' }}</span>
          </p>
          <p class="flex items-center gap-1">
            <span class="w-28 shrink-0 text-gray-400">平台支持</span>
            <span>{{ status.platformSupported ? '支持运行时注册' : '不支持（macOS 由打包 Info.plist 声明）' }}</span>
          </p>
        </div>
      </div>

      <!-- 说明 -->
      <p class="text-xs text-gray-400 leading-relaxed">
        安装版由安装器自动注册 molaunch:// 协议；便携版（绿色版）未经过安装程序，
        可在下方手动注册或卸载。卸载后点击 molaunch:// 链接系统将提示无应用处理。
      </p>

      <!-- 操作 -->
      <div class="flex items-center gap-3">
        <Button
          size="small"
          type="primary"
          :disabled="loading || !status?.platformSupported"
          @click="handleRegister"
        >
          注册协议
        </Button>
        <Button
          size="small"
          type="outline"
          :disabled="loading || !status?.platformSupported || !status?.registered"
          @click="handleUnregister"
        >
          卸载协议
        </Button>
        <Button size="small" type="ghost" :disabled="loading" @click="refresh">
          刷新状态
        </Button>
      </div>
    </div>
  </div>
</template>
