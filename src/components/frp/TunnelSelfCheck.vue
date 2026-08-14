<script setup lang="ts">
/**
 * 隧道自检结果面板
 *
 * 对所有隧道执行 4 项自检（配置完整性 / 服务器可达性 / 本地端口监听 / frpc 就绪），
 * 以卡片形式展示每条隧道的检查结果。onMounted 自动开始检查，支持手动重新检测。
 *
 * 检查逻辑复用 utils/frp-tunnel-check.ts，本组件仅负责展示。
 * 自定义组件：Button / Tooltip（项目约定，不用原生 button / title）。
 * 图标使用 @heroicons/vue/24/outline（项目已有依赖，不用 emoji）。
 */
import { ref, onMounted, defineAsyncComponent } from 'vue'
import { checkTunnels, type TunnelCheckResult, type CheckEntry } from '@/utils/frp-tunnel-check'
import { toastInfo, toastError } from '@/utils/toast'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import {
  XMarkIcon,
  ArrowPathIcon,
  CheckCircleIcon,
  XCircleIcon,
  ShieldCheckIcon,
} from '@heroicons/vue/24/outline'
import type { TunnelWithStatus, ProviderInfo } from '@/types/frp'

const props = defineProps<{
  tunnels: TunnelWithStatus[]
  providers: ProviderInfo[]
}>()

const emit = defineEmits<{ close: [] }>()

const results = ref<TunnelCheckResult[]>([])
const checking = ref(false)
const hasChecked = ref(false)

/** 4 项检查的展示配置（key 对应 TunnelCheckResult 中的 CheckEntry 字段） */
type CheckKey = 'config' | 'serverReachable' | 'localPortListening' | 'frpcReady'
const checkItems: { key: CheckKey; label: string }[] = [
  { key: 'config', label: '配置完整性' },
  { key: 'serverReachable', label: '服务器可达' },
  { key: 'localPortListening', label: '本地端口监听' },
  { key: 'frpcReady', label: 'frpc 就绪' },
]

async function runCheck() {
  // 无隧道时跳过 IPC 调用，直接显示空状态
  if (props.tunnels.length === 0) {
    results.value = []
    hasChecked.value = true
    return
  }
  checking.value = true
  try {
    results.value = await checkTunnels(props.tunnels, props.providers)
    toastInfo('自检完成')
  } catch (e) {
    toastError('自检失败：' + (e instanceof Error ? e.message : String(e)))
  } finally {
    checking.value = false
    hasChecked.value = true
  }
}

onMounted(() => {
  void runCheck()
})

function entryClass(entry: CheckEntry): string {
  return entry.ok ? 'text-green-600' : 'text-red-500'
}
</script>

<template>
  <div class="rounded-lg border border-gray-200 bg-white">
    <!-- 顶部标题栏 -->
    <div class="flex items-center justify-between border-b border-gray-200 px-4 py-3">
      <div class="flex items-center gap-2">
        <ShieldCheckIcon class="w-5 h-5 text-primary-500" />
        <h3 class="text-sm font-semibold text-gray-900">隧道自检结果</h3>
        <span v-if="hasChecked && !checking && tunnels.length > 0" class="text-xs text-gray-400">
          共 {{ results.length }} 条
        </span>
      </div>
      <div class="flex items-center gap-1.5">
        <Tooltip text="重新检测">
          <Button
            type="ghost"
            size="mini"
            :loading="checking"
            @click="runCheck"
          >
            <template #icon><ArrowPathIcon class="w-3.5 h-3.5" /></template>
          </Button>
        </Tooltip>
        <Tooltip text="关闭">
          <Button
            type="ghost"
            size="mini"
            @click="emit('close')"
          >
            <template #icon><XMarkIcon class="w-3.5 h-3.5" /></template>
          </Button>
        </Tooltip>
      </div>
    </div>

    <!-- 内容区 -->
    <div class="p-4">
      <!-- 空状态：无隧道时 icon + text 垂直水平居中 -->
      <div v-if="tunnels.length === 0" class="flex flex-col items-center justify-center py-16">
        <ShieldCheckIcon class="w-12 h-12 text-gray-300 mb-3" />
        <p class="text-sm font-medium text-gray-500">暂无隧道</p>
        <p class="text-xs text-gray-400 mt-1">请先创建隧道再执行自检</p>
      </div>

      <!-- 首次检测中：loading spinner -->
      <div v-else-if="checking && !hasChecked" class="flex items-center justify-center py-16">
        <ArrowPathIcon class="w-6 h-6 text-gray-400 animate-spin" />
        <span class="ml-2 text-sm text-gray-400">正在检测...</span>
      </div>

      <!-- 结果列表 -->
      <div v-else class="space-y-3">
        <div
          v-for="r in results"
          :key="r.tunnelId"
          class="rounded-lg border border-gray-200 p-3"
        >
          <!-- 隧道标题行 + 整体状态徽章 -->
          <div class="flex items-center justify-between gap-2">
            <span class="text-sm font-semibold text-gray-900 truncate">{{ r.tunnelName }}</span>
            <span
              class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-xs font-medium shrink-0"
              :class="r.overall
                ? 'bg-green-50 text-green-700'
                : 'bg-red-50 text-red-600'"
            >
              <CheckCircleIcon v-if="r.overall" class="w-3.5 h-3.5" />
              <XCircleIcon v-else class="w-3.5 h-3.5" />
              {{ r.overall ? '通过' : '未通过' }}
            </span>
          </div>

          <!-- 4 项检查结果，每项一行 -->
          <div class="mt-2 space-y-1.5">
            <div
              v-for="item in checkItems"
              :key="item.key"
              class="flex items-center gap-2 text-xs"
            >
              <CheckCircleIcon
                v-if="r[item.key].ok"
                class="w-3.5 h-3.5 shrink-0 text-green-600"
              />
              <XCircleIcon
                v-else
                class="w-3.5 h-3.5 shrink-0 text-red-500"
              />
              <span class="text-gray-500 w-20 shrink-0">{{ item.label }}</span>
              <span :class="entryClass(r[item.key])">{{ r[item.key].message }}</span>
              <span v-if="r[item.key].detail" class="text-gray-400">({{ r[item.key].detail }})</span>
            </div>
          </div>
        </div>

        <!-- 重新检测中提示（保留旧结果可见） -->
        <div v-if="checking && hasChecked" class="flex items-center justify-center py-2 text-xs text-gray-400">
          <ArrowPathIcon class="w-3.5 h-3.5 animate-spin mr-1.5" />
          正在重新检测...
        </div>
      </div>
    </div>
  </div>
</template>
