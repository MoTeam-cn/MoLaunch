<script setup lang="ts">
/**
 * 版本 JSON 编辑
 *
 * 选择已安装版本 → 读取 {game_dir}/versions/{id}/{id}.json → 编辑 → 保存。
 * 保存前后端会先校验 JSON 合法性，校验失败返回详细解析错误。
 * 因误改可能导致版本无法启动，保存走 showConfirm 回调式二次确认。
 */
import { ref, computed, onMounted, watch } from 'vue'
import {
  DocumentTextIcon,
  FolderOpenIcon,
  CheckIcon,
  ExclamationTriangleIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Select from '@/components/common/Select.vue'
import Input from '@/components/common/Input.vue'
import { toastSuccess, toastError } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { versionJsonRead, versionJsonSave } from '@/utils/api/tools'
import { listInstalledVersionsWithType } from '@/utils/api/version'
import type { InstalledVersionInfo } from '@/utils/api/version'

const versions = ref<InstalledVersionInfo[]>([])
const selectedVersion = ref<string>('')
const versionOptions = ref<{ label: string; value: string }[]>([])

const content = ref('')
const filePath = ref('')
const loading = ref(false)
const saving = ref(false)
const dirty = ref(false)

const canSave = computed(() => selectedVersion.value !== '' && dirty.value && content.value.trim() !== '')

watch(content, () => {
  if (!loading.value) dirty.value = true
})

onMounted(async () => {
  try {
    versions.value = await listInstalledVersionsWithType()
    versionOptions.value = versions.value.map((v) => ({ label: v.id, value: v.id }))
  } catch (e) {
    toastError(`加载版本列表失败: ${e instanceof Error ? e.message : String(e)}`)
  }
})

async function loadJson() {
  if (!selectedVersion.value) return
  loading.value = true
  content.value = ''
  filePath.value = ''
  dirty.value = false
  try {
    const res = await versionJsonRead(selectedVersion.value)
    content.value = res.content
    filePath.value = res.path
    dirty.value = false
  } catch (e) {
    toastError(`读取失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    loading.value = false
  }
}

watch(selectedVersion, () => {
  if (selectedVersion.value) loadJson()
})

function requestSave() {
  if (!canSave.value) return
  showConfirm(
    '确认保存版本 JSON',
    '将覆盖写入「' + selectedVersion.value + '」的版本 JSON 文件。错误的修改可能导致该版本无法启动，请确认内容正确。',
    () => doSave(),
  )
}

async function doSave() {
  saving.value = true
  try {
    await versionJsonSave(selectedVersion.value, content.value)
    toastSuccess('保存成功')
    dirty.value = false
  } catch (e) {
    toastError('保存失败: ' + (e instanceof Error ? e.message : String(e)))
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <DocumentTextIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">版本 JSON 编辑</h3>
    </div>
    <div class="px-5 pb-5 space-y-3">
      <p class="text-xs text-gray-500">
        直接编辑版本的 JSON 文件（含 inheritsFrom、mainClass、libraries、arguments 等字段）。保存前会校验 JSON 合法性。
      </p>

      <!-- 版本选择 -->
      <div class="flex items-end gap-3">
        <div class="flex-1">
          <label class="mb-1 block text-xs font-medium text-gray-700">选择版本</label>
          <Select v-model="selectedVersion" :options="versionOptions" placeholder="请选择版本" />
        </div>
        <Button type="outline" :loading="loading" :disabled="!selectedVersion" @click="loadJson">
          <template #icon><FolderOpenIcon class="h-4 w-4" /></template>
          重新读取
        </Button>
      </div>

      <!-- 文件路径提示 -->
      <div v-if="filePath" class="flex items-center gap-1.5 text-xs text-gray-400">
        <DocumentTextIcon class="h-3.5 w-3.5" />
        <span class="truncate">{{ filePath }}</span>
      </div>

      <!-- 未保存提示 -->
      <div v-if="dirty" class="flex items-center gap-1.5 text-xs text-amber-600">
        <ExclamationTriangleIcon class="h-3.5 w-3.5" />
        有未保存的修改
      </div>

      <!-- 编辑器 -->
      <Input
        v-if="content || loading"
        v-model="content"
        textarea
        :rows="14"
        placeholder="版本 JSON 内容..."
      />

      <!-- 保存按钮 -->
      <div v-if="content || filePath" class="flex justify-end">
        <Button type="primary" :loading="saving" :disabled="!canSave" @click="requestSave">
          <template #icon><CheckIcon class="h-4 w-4" /></template>
          {{ saving ? '保存中...' : '保存' }}
        </Button>
      </div>
    </div>
  </section>
</template>
