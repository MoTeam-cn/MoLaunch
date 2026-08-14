<script setup lang="ts">
/**
 * 存档管理
 *
 * 列出 saves 目录下的存档，支持：
 * - 备份：将存档打包为 zip（可选排除玩家数据）
 * - 恢复：从 zip 解压到 saves/ 目录
 * 默认扫全局 {game_dir}/saves/，可选具体版本按版本隔离配置解析路径。
 * 备份/恢复路径通过 Input 手动填写。
 *
 * 子组件拆分（避免主文件超 300 行）：
 * - ArchiveBackupDialog：备份弹窗（target/downloadDir/versionId → close）
 * - ArchiveRestorePanel：恢复面板（versionId → restored 触发列表刷新）
 */
import { ref, computed, onMounted, watch } from 'vue'
import {
  ArchiveBoxIcon,
  ArrowPathIcon,
  ArrowUpTrayIcon,
  ExclamationCircleIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import Select from '@/components/common/Select.vue'
import Tag from '@/components/common/Tag.vue'
import { toastSuccess, toastError } from '@/utils/toast'
import { archiveList, getDownloadDir } from '@/utils/api/tools'
import type { ArchiveItem } from '@/utils/api/tools'
import { listInstalledVersionsWithType, type InstalledVersionInfo } from '@/utils/api/version'
import { getConfigMap } from '@/utils/api/config'
import { formatBytes, formatTimestamp } from '@/utils/format'
import ArchiveBackupDialog from './ArchiveBackupDialog.vue'
import ArchiveRestorePanel from './ArchiveRestorePanel.vue'

const items = ref<ArchiveItem[]>([])
const totalSize = ref(0)
const loading = ref(false)
const loaded = ref(false)

// 备份弹窗控制：非 null 即弹窗打开（由列表内「备份」按钮赋值，由弹窗 close 事件置空）
const backupTarget = ref<ArchiveItem | null>(null)

const downloadDir = ref('')

// 版本选择：'' = 全局（不隔离），其他 = 具体版本 ID
const selectedVersionId = ref<string>('')
const installedVersions = ref<InstalledVersionInfo[]>([])
const versionOptions = computed(() => [
  { label: '全局（不隔离）', value: '' },
  ...installedVersions.value.map((v) => ({ label: v.id, value: v.id })),
])

async function loadList() {
  loading.value = true
  try {
    const res = await archiveList(selectedVersionId.value || undefined)
    items.value = res.items
    totalSize.value = res.total_size
    loaded.value = true
  } catch (e) {
    toastError('加载存档列表失败: ' + (e instanceof Error ? e.message : String(e)))
  } finally {
    loading.value = false
  }
}

async function refresh() {
  await loadList()
  toastSuccess('已刷新')
}

async function loadVersions() {
  try {
    installedVersions.value = await listInstalledVersionsWithType()
  } catch {
    toastError('加载版本列表失败')
  }
}

// 版本切换时重新加载（首次加载由 onMounted 触发，跳过初始回调）
watch(selectedVersionId, (newVal, oldVal) => {
  if (oldVal !== '' || newVal !== '') {
    loadList()
  }
})

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
  try {
    downloadDir.value = await getDownloadDir()
  } catch {
    downloadDir.value = ''
  }
})
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <ArchiveBoxIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">存档管理</h3>
      <span class="ml-auto text-xs text-gray-400">
        {{ items.length }} 个存档 · {{ formatBytes(totalSize) }}
      </span>
      <Select
        v-model="selectedVersionId"
        :options="versionOptions"
        class="w-44"
      />
      <Button type="outline" size="small" :loading="loading" @click="refresh">
        <template #icon><ArrowPathIcon class="h-4 w-4" /></template>
        刷新
      </Button>
    </div>
    <div class="px-5 pb-5 space-y-4">
      <p class="text-xs text-gray-500">
        管理游戏存档目录，支持备份（打包为 zip）和恢复（从 zip 解压）。
      </p>

      <!-- 存档列表 -->
      <div v-if="items.length > 0" data-inner-scroll class="max-h-[400px] overflow-y-auto rounded-lg border border-gray-200 divide-y divide-gray-100">
        <div
          v-for="item in items"
          :key="item.path"
          class="flex items-center gap-3 px-3 py-2.5 hover:bg-gray-50 transition-colors"
        >
          <ArchiveBoxIcon class="h-5 w-5 flex-none text-gray-400" />
          <Tooltip :text="item.path" position="top" :delay="200" block>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-1.5">
                <span class="truncate text-sm font-medium text-gray-900">{{ item.name }}</span>
                <span
                  v-if="item.has_level_dat"
                ><Tag size="small" color="green">有效</Tag></span>
                <ExclamationCircleIcon v-else class="h-3.5 w-3.5 text-amber-400" />
              </div>
              <div class="text-xs text-gray-400">{{ formatTimestamp(item.modified) }}</div>
            </div>
          </Tooltip>
          <span class="flex-none text-xs text-gray-500">{{ formatBytes(item.size) }}</span>
          <Button
            type="outline"
            size="small"
            class="flex-none"
            @click="backupTarget = item"
          >
            <template #icon><ArrowUpTrayIcon class="h-3.5 w-3.5" /></template>
            备份
          </Button>
        </div>
      </div>

      <!-- 空状态 -->
      <div
        v-else-if="loaded"
        class="flex flex-col items-center justify-center py-8 text-gray-400"
      >
        <ArchiveBoxIcon class="h-8 w-8 mb-2 text-gray-300" />
        <span class="text-xs">暂无存档</span>
      </div>

      <!-- 恢复区 -->
      <ArchiveRestorePanel
        :version-id="selectedVersionId"
        @restored="loadList"
      />
    </div>

    <!-- 备份弹窗 -->
    <ArchiveBackupDialog
      :target="backupTarget"
      :download-dir="downloadDir"
      :version-id="selectedVersionId"
      @close="backupTarget = null"
    />
  </section>
</template>
