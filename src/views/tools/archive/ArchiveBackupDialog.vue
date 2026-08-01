<script setup lang="ts">
/**
 * 存档备份弹窗
 *
 * 从 ArchiveManager 拆分而来，负责单个存档的备份交互：
 * - 选择输出 zip 路径（默认拼接 downloadDir + 存档名 + -backup.zip）
 * - 可选排除玩家数据（导出分享包）
 * - 确认后调用 archiveBackup，成功后 emit('close') 通知父组件关闭弹窗
 *
 * 复用：Button/Checkbox/Input 自定义组件、showConfirm 回调式确认、
 * pickSavePath 文件对话框、formatBytes 字节格式化。
 */
import { ref, watch } from 'vue'
import {
  ArrowUpTrayIcon,
  CheckCircleIcon,
  FolderOpenIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Checkbox from '@/components/common/Checkbox.vue'
import Input from '@/components/common/Input.vue'
import { toastSuccess, toastError } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { archiveBackup } from '@/utils/api/tools'
import type { ArchiveItem } from '@/utils/api/tools'
import { formatBytes } from '@/utils/format'
import { pickSavePath } from '@/utils/fileDialog'

const props = defineProps<{
  target: ArchiveItem | null
  downloadDir: string
  versionId: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const backupOutputPath = ref('')
const backupExcludePlayer = ref(false)
const backing = ref(false)

// 目标变化时重置表单（与原 startBackup 行为一致：拼接默认路径 + 关闭排除玩家数据）
watch(
  () => props.target,
  (target) => {
    if (!target) return
    backupOutputPath.value = props.downloadDir + '\\' + target.name + '-backup.zip'
    backupExcludePlayer.value = false
  },
  { immediate: true },
)

function requestBackup() {
  if (!props.target || !backupOutputPath.value.trim()) return
  const name = props.target.name
  const mode = backupExcludePlayer.value ? '导出分享包（排除玩家数据）' : '完整备份'
  showConfirm(
    '确认备份存档',
    '将备份存档「' + name + '」（' + mode + '）到：' + backupOutputPath.value,
    () => doBackup(),
  )
}

async function doBackup() {
  if (!props.target) return
  backing.value = true
  try {
    const res = await archiveBackup(
      props.target.name,
      backupOutputPath.value.trim(),
      backupExcludePlayer.value,
      props.versionId || undefined,
    )
    if (res.success) {
      toastSuccess('备份成功：' + formatBytes(res.file_size))
      emit('close')
    } else {
      toastError('备份失败，请检查路径和权限')
    }
  } catch (e) {
    toastError('备份失败: ' + (e instanceof Error ? e.message : String(e)))
  } finally {
    backing.value = false
  }
}

async function pickBackupOutput() {
  const path = await pickSavePath({
    title: '选择备份 zip 保存位置',
    defaultPath: backupOutputPath.value || (props.target?.name + '-backup.zip'),
    filters: [{ name: 'ZIP 压缩包', extensions: ['zip'] }],
  })
  if (path) backupOutputPath.value = path
}
</script>

<template>
  <div
    v-if="target"
    class="fixed inset-0 z-[10000] flex items-center justify-center p-4"
    @click.self="emit('close')"
  >
    <div class="absolute inset-0 bg-black/40" />
    <div class="relative w-full max-w-md bg-white rounded-lg shadow-xl">
      <div class="p-5 space-y-4">
        <div class="flex items-center gap-2">
          <ArrowUpTrayIcon class="h-5 w-5 text-gray-700" />
          <h4 class="text-sm font-semibold text-gray-900">备份存档</h4>
        </div>
        <div class="text-xs text-gray-500">
          存档名称：<span class="font-medium text-gray-700">{{ target.name }}</span>
          （{{ formatBytes(target.size) }}）
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
      </div>
      <div class="flex justify-end gap-2 px-5 py-3.5 bg-gray-50 rounded-b-lg">
        <Button type="outline" size="small" @click="emit('close')">取消</Button>
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
</template>
