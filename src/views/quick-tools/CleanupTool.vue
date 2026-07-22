<script setup lang="ts">
/**
 * 清理游戏垃圾子组件
 *
 * 扫描 .minecraft 下的 logs/crash-reports/.mixin.out/screenshots 等，
 * 用户勾选后清理。
 *
 * UI 布局：
 * - 顶部固定：标题 + 扫描/重新扫描按钮
 * - 中部滚动：扫描结果按分组展示（全局 / 各版本），每组可折叠（文件树形式）
 * - 底部固定：已选汇总 + 清理选中按钮
 *
 * 分组计算与渲染委托给 CleanupGroupList.vue，本组件只负责状态管理与编排。
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
import CleanupGroupList from './CleanupGroupList.vue'

const scanState = ref<'idle' | 'scanning' | 'ready' | 'cleaning'>('idle')
const scanItems = ref<CleanupItem[]>([])
const selectedPaths = ref<Set<string>>(new Set())
const scanTotalSize = ref(0)
const cleanResult = ref<CleanupExecuteResult | null>(null)
const collapsedGroups = ref<Set<string>>(new Set())

const selectedSize = computed(() =>
  scanItems.value
    .filter((item) => selectedPaths.value.has(item.path))
    .reduce((sum, item) => sum + item.size, 0),
)

const selectedCount = computed(() => selectedPaths.value.size)

async function startScan() {
  scanState.value = 'scanning'
  cleanResult.value = null
  selectedPaths.value.clear()
  collapsedGroups.value.clear()
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
    // 默认折叠所有分组：扫描完成后用户先看到分组概览，按需展开查看明细
    const groupKeys = new Set<string>()
    for (const item of result.items) {
      const dashIdx = item.display_name.indexOf(' - ')
      groupKeys.add(dashIdx > 0 ? item.display_name.substring(dashIdx + 3) : '全局')
    }
    collapsedGroups.value = groupKeys
    scanState.value = 'ready'
    if (result.items.length === 0) {
      toastInfo('未发现可清理的文件')
    } else {
      toastInfo(`发现 ${result.items.length} 项可清理内容，共 ${formatBytes(result.total_size)}`)
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
  selectedPaths.value = new Set(selectedPaths.value)
}

function toggleGroup(key: string) {
  if (collapsedGroups.value.has(key)) {
    collapsedGroups.value.delete(key)
  } else {
    collapsedGroups.value.add(key)
  }
  collapsedGroups.value = new Set(collapsedGroups.value)
}

// 全选/取消全选：需要拿到组内所有 item 的 path
// 由于组计算在子组件内，这里通过 scanItems 重新匹配同组项
function toggleGroupSelect(groupKey: string) {
  const groupItems = scanItems.value.filter((item) => {
    const dashIdx = item.display_name.indexOf(' - ')
    const gk = dashIdx > 0 ? item.display_name.substring(dashIdx + 3) : '全局'
    return gk === groupKey
  })
  const allSelected = groupItems.every((item) => selectedPaths.value.has(item.path))
  if (allSelected) {
    for (const item of groupItems) {
      selectedPaths.value.delete(item.path)
    }
  } else {
    for (const item of groupItems) {
      selectedPaths.value.add(item.path)
    }
  }
  selectedPaths.value = new Set(selectedPaths.value)
}

function executeCleanup() {
  if (selectedPaths.value.size === 0) {
    toastError('请至少选择一项要清理的内容')
    return
  }

  showConfirm(
    '确认清理',
    `即将清理 ${selectedCount.value} 项内容，共 ${formatBytes(selectedSize.value)}。此操作不可恢复，确定继续吗？`,
    async () => {
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
        selectedPaths.value.clear()
        scanItems.value = []
      } catch (e) {
        toastError(`清理失败: ${e instanceof Error ? e.message : String(e)}`)
        scanState.value = 'ready'
      }
    },
  )
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white overflow-hidden">
    <!-- 顶部：标题 + 扫描按钮（固定） -->
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

    <!-- 中部：扫描结果（高度限制 + 滚动） -->
    <div class="px-5">
      <!-- 扫描中 -->
      <div v-if="scanState === 'scanning'" class="flex h-24 items-center justify-center pb-5">
        <div class="flex flex-col items-center gap-2 text-gray-400">
          <ArrowPathIcon class="h-6 w-6 animate-spin text-primary-400" />
          <span class="text-xs">正在扫描...</span>
        </div>
      </div>

      <!-- 清理中 -->
      <div v-else-if="scanState === 'cleaning'" class="flex h-24 items-center justify-center pb-5">
        <div class="flex flex-col items-center gap-2 text-gray-400">
          <TrashIcon class="h-6 w-6 animate-pulse text-primary-400" />
          <span class="text-xs">正在清理...</span>
        </div>
      </div>

      <!-- 扫描结果：文件树分组展示（委托给 CleanupGroupList） -->
      <div
        v-else-if="scanState === 'ready' && scanItems.length > 0"
        class="max-h-[400px] overflow-y-auto pr-1"
      >
        <CleanupGroupList
          :items="scanItems"
          :selected-paths="selectedPaths"
          :collapsed-groups="collapsedGroups"
          @toggle-select="toggleSelect"
          @toggle-group="toggleGroup"
          @toggle-group-select="toggleGroupSelect"
        />
      </div>

      <!-- 清理结果 -->
      <div v-else-if="cleanResult" class="space-y-3 pb-5">
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
      <div v-else class="flex h-24 items-center justify-center pb-5">
        <div class="flex flex-col items-center gap-2 text-gray-400">
          <ExclamationCircleIcon class="h-8 w-8" />
          <span class="text-xs">点击「扫描」按钮检查可清理的文件</span>
        </div>
      </div>
    </div>

    <!-- 底部：已选汇总 + 清理按钮（固定） -->
    <div
      v-if="scanState === 'ready' && scanItems.length > 0"
      class="flex items-center justify-between border-t border-gray-200 bg-gray-50 px-5 py-3"
    >
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
  </section>
</template>
