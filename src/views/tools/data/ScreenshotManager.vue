<script setup lang="ts">
/**
 * 截图批量管理
 *
 * 列举 screenshots 目录下的截图文件，支持多选与批量删除。
 * 默认扫全局 {game_dir}/screenshots/，可选具体版本按版本隔离配置解析路径。
 * 删除走 showConfirm 回调式（项目规范：业务逻辑放入 onConfirm 回调）。
 */
import { ref, computed, onMounted, watch } from 'vue'
import {
  PhotoIcon,
  ArrowPathIcon,
  TrashIcon,
  CheckCircleIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import Select from '@/components/common/Select.vue'
import { toastSuccess, toastError } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { screenshotList, screenshotDelete } from '@/utils/api/tools'
import type { ScreenshotItem } from '@/utils/api/tools'
import { listInstalledVersionsWithType, type InstalledVersionInfo } from '@/utils/api/version'
import { getConfigMap } from '@/utils/api/config'
import { formatBytes } from '@/utils/format'

const items = ref<ScreenshotItem[]>([])
const totalSize = ref(0)
const loading = ref(false)
const deleting = ref(false)
const selectedPaths = ref<Set<string>>(new Set())
const loaded = ref(false)

// 版本选择：'' = 全局（不隔离），其他 = 具体版本 ID
const selectedVersionId = ref<string>('')
const installedVersions = ref<InstalledVersionInfo[]>([])
const versionOptions = computed(() => [
  { label: '全局（不隔离）', value: '' },
  ...installedVersions.value.map((v) => ({ label: v.id, value: v.id })),
])

const hasSelection = computed(() => selectedPaths.value.size > 0)

function formatDate(unixSec: number): string {
  return new Date(unixSec * 1000).toLocaleString('zh-CN', { hour12: false })
}

async function loadList() {
  loading.value = true
  try {
    const res = await screenshotList(selectedVersionId.value || undefined)
    items.value = res.items
    totalSize.value = res.total_size
    selectedPaths.value = new Set()
    loaded.value = true
  } catch (e) {
    toastError(`加载截图列表失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    loading.value = false
  }
}

async function loadVersions() {
  try {
    installedVersions.value = await listInstalledVersionsWithType()
  } catch (e) {
    console.warn('加载已安装版本失败', e)
  }
}

// 版本切换时重新加载（首次加载由 onMounted 触发，跳过初始回调）
watch(selectedVersionId, (newVal, oldVal) => {
  if (oldVal !== '' || newVal !== '') {
    loadList()
  }
})

function toggleSelect(path: string) {
  const next = new Set(selectedPaths.value)
  if (next.has(path)) next.delete(path)
  else next.add(path)
  selectedPaths.value = next
}

function toggleSelectAll() {
  if (selectedPaths.value.size === items.value.length) {
    selectedPaths.value = new Set()
  } else {
    selectedPaths.value = new Set(items.value.map((i) => i.path))
  }
}

function requestDelete() {
  if (!hasSelection.value) return
  const count = selectedPaths.value.size
  showConfirm(
    '确认删除截图',
    `将删除 ${count} 张截图（共 ${formatBytes(sumSize(selectedPaths.value))}），此操作不可恢复。`,
    () => doDelete(),
  )
}

function sumSize(paths: Set<string>): number {
  let s = 0
  for (const it of items.value) if (paths.has(it.path)) s += it.size
  return s
}

async function doDelete() {
  deleting.value = true
  try {
    const paths = Array.from(selectedPaths.value)
    const res = await screenshotDelete(paths, selectedVersionId.value || undefined)
    if (res.failed.length > 0) {
      toastError(`${res.failed.length} 张删除失败：${res.failed[0].error}`)
    } else {
      toastSuccess(`已删除 ${res.deleted_count} 张，释放 ${formatBytes(res.freed_bytes)}`)
    }
    selectedPaths.value = new Set()
    await loadList()
  } catch (e) {
    toastError(`删除失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    deleting.value = false
  }
}

onMounted(async () => {
  await loadVersions()
  // 全局隔离模式为 All(4) 时，所有版本都隔离，"全局（不隔离）"选项失去意义
  // 默认选中第一个已安装版本，让用户直接看到版本隔离目录
  const config = await getConfigMap()
  if (config.isolationMode === 4 && installedVersions.value.length > 0) {
    selectedVersionId.value = installedVersions.value[0].id
    // watch 会自动触发 loadList
  } else {
    await loadList()
  }
})
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <PhotoIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">截图批量管理</h3>
      <span class="ml-auto text-xs text-gray-400">
        {{ items.length }} 张 · {{ formatBytes(totalSize) }}
      </span>
      <Select
        v-model="selectedVersionId"
        :options="versionOptions"
        class="w-44"
      />
      <Button type="outline" size="small" :loading="loading" @click="loadList">
        <template #icon><ArrowPathIcon class="h-4 w-4" /></template>
        刷新
      </Button>
    </div>
    <div class="px-5 pb-5 space-y-3">
      <p class="text-xs text-gray-500">
        管理游戏截图目录下的文件，支持多选批量删除。
      </p>

      <!-- 操作栏 -->
      <div v-if="items.length > 0" class="flex items-center gap-2">
        <label class="flex items-center gap-1.5 cursor-pointer text-xs text-gray-600">
          <input
            type="checkbox"
            class="accent-primary-500"
            :checked="selectedPaths.size === items.length && items.length > 0"
            @change="toggleSelectAll"
          />
          全选
        </label>
        <span v-if="hasSelection" class="text-xs text-gray-400">
          已选 {{ selectedPaths.size }} 项 · {{ formatBytes(sumSize(selectedPaths)) }}
        </span>
        <Button
          class="ml-auto"
          type="outline"
          size="small"
          :disabled="!hasSelection"
          :loading="deleting"
          @click="requestDelete"
        >
          <template #icon><TrashIcon class="h-4 w-4" /></template>
          删除选中
        </Button>
      </div>

      <!-- 截图列表 -->
      <div v-if="items.length > 0" class="max-h-[400px] overflow-y-auto rounded-lg border border-gray-200 divide-y divide-gray-100">
        <div
          v-for="item in items"
          :key="item.path"
          class="flex items-center gap-3 px-3 py-2.5 cursor-pointer transition-colors"
          :class="selectedPaths.has(item.path) ? 'bg-primary-50/60' : 'hover:bg-gray-50'"
          @click="toggleSelect(item.path)"
        >
          <input
            type="checkbox"
            class="accent-primary-500 flex-none"
            :checked="selectedPaths.has(item.path)"
            @click.stop="toggleSelect(item.path)"
          />
          <PhotoIcon class="h-5 w-5 flex-none text-gray-400" />
          <Tooltip :text="item.path" position="top" :delay="200" block>
            <div class="flex-1 min-w-0">
              <div class="truncate text-sm font-medium text-gray-900">{{ item.name }}</div>
              <div class="text-xs text-gray-400">{{ formatDate(item.modified) }}</div>
            </div>
          </Tooltip>
          <span class="flex-none text-xs text-gray-500">{{ formatBytes(item.size) }}</span>
        </div>
      </div>

      <!-- 空状态 -->
      <div
        v-else-if="loaded"
        class="flex flex-col items-center justify-center py-8 text-gray-400"
      >
        <CheckCircleIcon class="h-8 w-8 mb-2 text-green-400" />
        <span class="text-xs">暂无截图文件</span>
      </div>
    </div>
  </section>
</template>
