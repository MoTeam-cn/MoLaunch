<script setup lang="ts">
/**
 * 存档恢复面板
 *
 * 从 ArchiveManager 拆分而来，负责从 zip 恢复存档：
 * - 选择 zip 文件 + 填写恢复后的存档名称（可选）
 * - 确认后调用 archiveRestore，成功后清空表单并 emit('restored')
 *   通知父组件刷新存档列表
 *
 * 复用：Button/Input 自定义组件、showConfirm 回调式确认、
 * pickFile 文件对话框。
 */
import { ref } from 'vue'
import {
  ArrowDownTrayIcon,
  FolderOpenIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import { toastSuccess, toastError } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { archiveRestore } from '@/utils/api/tools'
import { pickFile } from '@/utils/fileDialog'

const props = defineProps<{
  versionId: string
}>()

const emit = defineEmits<{
  (e: 'restored'): void
}>()

const restoreZipPath = ref('')
const restoreWorldName = ref('')
const restoring = ref(false)

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
      props.versionId || undefined,
    )
    if (res.success) {
      toastSuccess('恢复成功：' + res.world_name)
      restoreZipPath.value = ''
      restoreWorldName.value = ''
      emit('restored')
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
</script>

<template>
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
</template>
