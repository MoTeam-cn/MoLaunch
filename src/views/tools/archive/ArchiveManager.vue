<script setup lang="ts">
/**
 * 存档管理
 *
 * 列出 saves 目录下的存档，支持：
 * - 备份：将存档打包为 zip（可选排除玩家数据）
 * - 恢复：从 zip 解压到 saves/ 目录
 * 默认扫全局 {game_dir}/saves/，可选具体版本按版本隔离配置解析路径。
 * 备份/恢复路径通过 Input 手动填写（与 DataExporter 一致）。
 */
import { ref, computed, onMounted, watch } from 'vue'
import {
  ArchiveBoxIcon,
  ArrowPathIcon,
  ArrowUpTrayIcon,
  ArrowDownTrayIcon,
  CheckCircleIcon,
  ExclamationCircleIcon,
  FolderOpenIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Checkbox from '@/components/common/Checkbox.vue'
import Input from '@/components/common/Input.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import Select from '@/components/common/Select.vue'
import { toastSuccess, toastError } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { archiveList, archiveBackup, archiveRestore, getDownloadDir } from '@/utils/api/tools'
import type { ArchiveItem } from '@/utils/api/tools'
import { listInstalledVersionsWithType, type InstalledVersionInfo } from '@/utils/api/version'
import { getConfigMap } from '@/utils/api/config'
import { formatBytes } from '@/utils/format'
import { pickFile, pickSavePath } from '@/utils/fileDialog'

const items = ref<ArchiveItem[]>([])
const totalSize = ref(0)
const loading = ref(false)
const loaded = ref(false)

// 备份对话框状态
const backupTarget = ref<ArchiveItem | null>(null)
const backupOutputPath = ref('')
const backupExcludePlayer = ref(false)
const backing = ref(false)

// 恢复对话框状态
const restoreZipPath = ref('')
const restoreWorldName = ref('')
const restoring = ref(false)

const downloadDir = ref('')

// 版本选择：'' = 全局（不隔离），其他 = 具体版本 ID
const selectedVersionId = ref<string>('')
const installedVersions = ref<InstalledVersionInfo[]>([])
const versionOptions = computed(() => [
  { label: '全局（不隔离）', value: '' },
  ...installedVersions.value.map((v) => ({ label: v.id, value: v.id })),
])

function formatDate(unixSec: number): string {
  return new Date(unixSec * 1000).toLocaleString('zh-CN', { hour12: false })
}

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

function startBackup(item: ArchiveItem) {
  backupTarget.value = item
  backupOutputPath.value = downloadDir.value + '\\' + item.name + '-backup.zip'
  backupExcludePlayer.value = false
}

function cancelBackup() {
  backupTarget.value = null
}

function requestBackup() {
  if (!backupTarget.value || !backupOutputPath.value.trim()) return
  const name = backupTarget.value.name
  const mode = backupExcludePlayer.value ? '导出分享包（排除玩家数据）' : '完整备份'
  showConfirm(
    '确认备份存档',
    '将备份存档「' + name + '」（' + mode + '）到：' + backupOutputPath.value,
    () => doBackup(),
  )
}

async function doBackup() {
  if (!backupTarget.value) return
  backing.value = true
  try {
    const res = await archiveBackup(
      backupTarget.value.name,
      backupOutputPath.value.trim(),
      backupExcludePlayer.value,
      selectedVersionId.value || undefined,
    )
    if (res.success) {
      toastSuccess('备份成功：' + formatBytes(res.file_size))
      backupTarget.value = null
    } else {
      toastError('备份失败，请检查路径和权限')
    }
  } catch (e) {
    toastError('备份失败: ' + (e instanceof Error ? e.message : String(e)))
  } finally {
    backing.value = false
  }
}

function requestRestore() {
  if (!restoreZipPath.value.trim()) return
  showConfirm(
    '确认恢复存档',
    '将从 zip 文件恢复存档' + (restoreWorldName.value.trim() ? '「' + restoreWorldName.value.trim() + '」' : '') + '，目标已存在时会失败。',
    () => doRestore(),
  )
}

async function doRestore() {
  restoring.value = true
  try {
    const res = await archiveRestore(
      restoreZipPath.value.trim(),
      restoreWorldName.value.trim(),
      selectedVersionId.value || undefined,
    )
    if (res.success) {
      toastSuccess('恢复成功：' + res.world_name)
      restoreZipPath.value = ''
      restoreWorldName.value = ''
      await loadList()
    } else {
      toastError('恢复失败：' + res.message)
    }
  } catch (e) {
    toastError('恢复失败: ' + (e instanceof Error ? e.message : String(e)))
  } finally {
    restoring.value = false
  }
}

async function pickRestoreZip() {
  const path = await pickFile({
    title: '选择存档备份 zip',
    filters: [{ name: 'ZIP 压缩包', extensions: ['zip'] }],
  })
  if (path) restoreZipPath.value = path
}

async function pickBackupOutput() {
  const path = await pickSavePath({
    title: '选择备份 zip 保存位置',
    defaultPath: backupOutputPath.value || (backupTarget.value?.name + '-backup.zip'),
    filters: [{ name: 'ZIP 压缩包', extensions: ['zip'] }],
  })
  if (path) backupOutputPath.value = path
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
      <Button type="outline" size="small" :loading="loading" @click="loadList">
        <template #icon><ArrowPathIcon class="h-4 w-4" /></template>
        刷新
      </Button>
    </div>
    <div class="px-5 pb-5 space-y-4">
      <p class="text-xs text-gray-500">
        管理游戏存档目录，支持备份（打包为 zip）和恢复（从 zip 解压）。
      </p>

      <!-- 存档列表 -->
      <div v-if="items.length > 0" class="max-h-[400px] overflow-y-auto rounded-lg border border-gray-200 divide-y divide-gray-100">
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
                  class="rounded-full bg-green-100 px-1.5 py-0.5 text-xs font-medium text-green-700"
                >有效</span>
                <ExclamationCircleIcon v-else class="h-3.5 w-3.5 text-amber-400" />
              </div>
              <div class="text-xs text-gray-400">{{ formatDate(item.modified) }}</div>
            </div>
          </Tooltip>
          <span class="flex-none text-xs text-gray-500">{{ formatBytes(item.size) }}</span>
          <Button
            type="outline"
            size="small"
            class="flex-none"
            @click="startBackup(item)"
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
      <div class="rounded-lg border border-gray-200 p-4 space-y-3">
        <div class="flex items-center gap-1.5 text-xs font-medium text-gray-700">
          <ArrowDownTrayIcon class="h-4 w-4" />
          从 zip 恢复存档
        </div>
        <Input
          v-model="restoreZipPath"
          placeholder="zip 文件完整路径"
          clearable
        >
          <template #append>
            <FolderOpenIcon
              class="h-4 w-4 cursor-pointer text-gray-500 hover:text-primary-600 transition-colors"
              @click="pickRestoreZip"
            />
          </template>
        </Input>
        <Input
          v-model="restoreWorldName"
          placeholder="恢复后的存档名称（留空则用 zip 文件名）"
          clearable
        />
        <div class="flex justify-end">
          <Button
            type="primary"
            :loading="restoring"
            :disabled="!restoreZipPath.trim()"
            @click="requestRestore"
          >
            <template #icon><ArrowDownTrayIcon class="h-4 w-4" /></template>
            {{ restoring ? '恢复中...' : '恢复' }}
          </Button>
        </div>
      </div>
    </div>

    <!-- 备份弹窗 -->
    <div
      v-if="backupTarget"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/30"
      @click.self="cancelBackup"
    >
      <div class="w-96 rounded-lg bg-white shadow-xl border border-gray-200 p-5 space-y-4">
        <div class="flex items-center gap-2">
          <ArrowUpTrayIcon class="h-5 w-5 text-gray-700" />
          <h4 class="text-sm font-semibold text-gray-900">备份存档</h4>
        </div>
        <div class="text-xs text-gray-500">
          存档名称：<span class="font-medium text-gray-700">{{ backupTarget.name }}</span>
          （{{ formatBytes(backupTarget.size) }}）
        </div>
        <div>
          <label class="mb-1 block text-xs font-medium text-gray-700">输出 zip 路径</label>
          <Input v-model="backupOutputPath" placeholder="输出 zip 完整路径" clearable>
            <template #append>
              <FolderOpenIcon
                class="h-4 w-4 cursor-pointer text-gray-500 hover:text-primary-600 transition-colors"
                @click="pickBackupOutput"
              />
            </template>
          </Input>
        </div>
        <div class="flex items-center gap-2">
          <Checkbox v-model="backupExcludePlayer">排除玩家数据（导出分享包）</Checkbox>
        </div>
        <div class="flex justify-end gap-2">
          <Button type="outline" size="small" @click="cancelBackup">取消</Button>
          <Button
            type="primary"
            size="small"
            :loading="backing"
            :disabled="!backupOutputPath.trim()"
            @click="requestBackup"
          >
            <template #icon><CheckCircleIcon class="h-4 w-4" /></template>
            确认备份
          </Button>
        </div>
      </div>
    </div>
  </section>
</template>
