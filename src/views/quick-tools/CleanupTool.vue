<script setup lang="ts">
/**
 * 清理游戏垃圾子组件
 *
 * 扫描 .minecraft 下的 logs/crash-reports/.mixin.out/screenshots，
 * 用户勾选后清理。
 */
import { ref, computed } from 'vue'
import {
  TrashIcon,
  ArrowPathIcon,
  CheckCircleIcon,
  ExclamationCircleIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import { toastSuccess, toastError, toastInfo } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { cleanupScan, cleanupExecute } from '@/utils/api/tools'
import type { CleanupItem, CleanupExecuteResult } from '@/utils/api/tools'
import { formatBytes } from '@/utils/format'

const scanState = ref<'idle' | 'scanning' | 'ready' | 'cleaning'>('idle')
const scanItems = ref<CleanupItem[]>([])
const selectedPaths = ref<Set<string>>(new Set())
const scanTotalSize = ref(0)
const cleanResult = ref<CleanupExecuteResult | null>(null)

const selectedSize = computed(() => {
  return scanItems.value
    .filter((item) => selectedPaths.value.has(item.path))
    .reduce((sum, item) => sum + item.size, 0)
})

const selectedCount = computed(() => selectedPaths.value.size)

async function startScan() {
  scanState.value = 'scanning'
  cleanResult.value = null
  selectedPaths.value.clear()
  try {
    const result = await cleanupScan()
    scanItems.value = result.items
    scanTotalSize.value = result.total_size
    // 默认选中非"可选"类别的项目
    for (const item of result.items) {
      if (item.category !== '可选') {
        selectedPaths.value.add(item.path)
      }
    }
    scanState.value = 'ready'
    if (result.items.length === 0) {
      toastInfo('未发现可清理的文件')
    }
  } catch (e) {
    toastError(`扫描失败: ${e instanceof Error ? e.message : String(e)}`)
    scanState.value = 'idle'
  }
}

function toggleSelect(path: string) {
  if (selectedPaths.value.has(path)) {
    selectedPaths.value.delete(path)
  } else {
    selectedPaths.value.add(path)
  }
  // 触发响应式更新
  selectedPaths.value = new Set(selectedPaths.value)
}

async function executeCleanup() {
  if (selectedPaths.value.size === 0) {
    toastError('请至少选择一项要清理的内容')
    return
  }

  const confirmed = await showConfirm(
    '确认清理',
    `即将清理 ${selectedCount.value} 项内容，共 ${formatBytes(selectedSize.value)}。此操作不可恢复，确定继续吗？`,
  )
  if (!confirmed) return

  scanState.value = 'cleaning'
  try {
    const result = await cleanupExecute([...selectedPaths.value])
    cleanResult.value = result
    scanState.value = 'idle'
    if (result.failed.length > 0) {
      toastError(`清理完成，但 ${result.failed.length} 项失败`)
    } else {
      toastSuccess(`已清理 ${formatBytes(result.cleaned_size)}，释放 ${result.cleaned_files} 个文件`)
    }
    // 清空选中状态
    selectedPaths.value.clear()
    scanItems.value = []
  } catch (e) {
    toastError(`清理失败: ${e instanceof Error ? e.message : String(e)}`)
    scanState.value = 'ready'
  }
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center justify-between px-5 pt-5 pb-3">
      <div class="flex items-center gap-2">
        <TrashIcon class="h-5 w-5 text-gray-700" />
        <h3 class="text-sm font-semibold text-gray-900">清理游戏垃圾</h3>
      </div>
      <Button
        v-if="scanState === 'idle' || scanState === 'ready'"
        type="outline"
        size="small"
        :disabled="scanState === 'cleaning'"
        @click="startScan"
      >
        <template #icon>
          <ArrowPathIcon class="h-3.5 w-3.5" :class="{ 'animate-spin': scanState === 'scanning' }" />
        </template>
        {{ scanState === 'ready' ? '重新扫描' : '扫描' }}
      </Button>
    </div>

    <div class="px-5 pb-5">
      <!-- 扫描中 -->
      <div v-if="scanState === 'scanning'" class="flex h-24 items-center justify-center">
        <div class="flex flex-col items-center gap-2 text-gray-400">
          <ArrowPathIcon class="h-6 w-6 animate-spin text-primary-400" />
          <span class="text-xs">正在扫描...</span>
        </div>
      </div>

      <!-- 清理中 -->
      <div v-else-if="scanState === 'cleaning'" class="flex h-24 items-center justify-center">
        <div class="flex flex-col items-center gap-2 text-gray-400">
          <TrashIcon class="h-6 w-6 animate-pulse text-primary-400" />
          <span class="text-xs">正在清理...</span>
        </div>
      </div>

      <!-- 扫描结果 -->
      <div v-else-if="scanState === 'ready' && scanItems.length > 0" class="space-y-2">
        <div
          v-for="item in scanItems"
          :key="item.path"
          class="flex items-center gap-3 rounded-lg border px-4 py-3 transition-colors"
          :class="
            selectedPaths.has(item.path)
              ? 'border-primary-300 bg-primary-50'
              : 'border-gray-200 bg-gray-50'
          "
        >
          <button
            class="flex h-4 w-4 flex-none items-center justify-center rounded border transition-colors"
            :class="
              selectedPaths.has(item.path)
                ? 'border-primary-500 bg-primary-500 text-white'
                : 'border-gray-300 bg-white'
            "
            @click="toggleSelect(item.path)"
          >
            <CheckCircleIcon v-if="selectedPaths.has(item.path)" class="h-3 w-3" />
          </button>
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2">
              <span class="text-sm font-medium text-gray-900">{{ item.display_name }}</span>
              <span
                class="rounded-full px-1.5 py-0.5 text-xs font-medium"
                :class="
                  item.category === '可选'
                    ? 'bg-yellow-100 text-yellow-700'
                    : 'bg-blue-100 text-blue-700'
                "
              >
                {{ item.category }}
              </span>
            </div>
            <div class="mt-0.5 truncate text-xs text-gray-400" :title="item.path">
              {{ item.path }}
            </div>
          </div>
          <div class="flex-none text-right">
            <div class="text-sm font-medium text-gray-700">{{ formatBytes(item.size) }}</div>
            <div class="text-xs text-gray-400">{{ item.file_count }} 个文件</div>
          </div>
        </div>

        <!-- 清理操作栏 -->
        <div class="flex items-center justify-between pt-2">
          <span class="text-xs text-gray-500">
            已选 {{ selectedCount }} 项，共 {{ formatBytes(selectedSize) }}
          </span>
          <Button
            type="primary"
            size="small"
            :disabled="selectedCount === 0"
            @click="executeCleanup"
          >
            <template #icon>
              <TrashIcon class="h-3.5 w-3.5" />
            </template>
            清理选中
          </Button>
        </div>
      </div>

      <!-- 清理结果 -->
      <div v-else-if="cleanResult" class="space-y-3">
        <div class="flex items-center gap-3 rounded-lg bg-green-50 px-4 py-3">
          <CheckCircleIcon class="h-5 w-5 flex-none text-green-500" />
          <div class="flex-1">
            <div class="text-sm font-medium text-green-800">
              已清理 {{ formatBytes(cleanResult.cleaned_size) }}，释放 {{ cleanResult.cleaned_files }} 个文件
            </div>
          </div>
        </div>
        <div v-if="cleanResult.failed.length > 0" class="space-y-1">
          <div
            v-for="fail in cleanResult.failed"
            :key="fail.path"
            class="flex items-center gap-2 rounded bg-red-50 px-3 py-2 text-xs text-red-600"
          >
            <ExclamationCircleIcon class="h-4 w-4 flex-none" />
            <span class="truncate">{{ fail.path }}</span>
            <span class="flex-none">{{ fail.error }}</span>
          </div>
        </div>
        <Button type="outline" size="small" @click="startScan">
          <template #icon>
            <ArrowPathIcon class="h-3.5 w-3.5" />
          </template>
          重新扫描
        </Button>
      </div>

      <!-- 空状态 -->
      <div v-else class="flex h-24 items-center justify-center">
        <div class="flex flex-col items-center gap-2 text-gray-400">
          <ExclamationCircleIcon class="h-8 w-8" />
          <span class="text-xs">点击「扫描」按钮检查可清理的文件</span>
        </div>
      </div>
    </div>
  </section>
</template>
