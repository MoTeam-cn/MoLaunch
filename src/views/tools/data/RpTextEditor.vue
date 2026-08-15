<script setup lang="ts">
/**
 * 包内 JSON / 文本文件编辑器（复用版本 JSON 编辑模式）
 *
 * json / model 类型保存前先校验 JSON 语法；text 类型原文保存。
 */
import { computed, ref, watch, defineAsyncComponent } from 'vue'
import { CheckIcon, DocumentTextIcon, ExclamationTriangleIcon } from '@heroicons/vue/24/outline'
import Tooltip from '@/components/common/Tooltip.vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
import { toastError, toastSuccess } from '@/utils/toast'
import { rpWrite } from '@/utils/api/tools'

const props = defineProps<{
  workDir: string
  relPath: string
  name: string
  /** 文件类型：json / model / text */
  fileType: string
  content: string
}>()

const emit = defineEmits<{ (e: 'saved'): void }>()

const text = ref('')
/** 加载时的初始内容快照，用于判定是否真正有改动 */
const original = ref('')
const dirty = ref(false)
const saving = ref(false)
const loaded = ref(false)

const isJson = computed(() => props.fileType === 'json' || props.fileType === 'model')
const jsonError = computed(() => {
  if (!isJson.value || !dirty.value) return ''
  try {
    JSON.parse(text.value)
    return ''
  } catch (e) {
    return e instanceof Error ? e.message : String(e)
  }
})

watch(
  () => props.content,
  (v) => {
    loaded.value = false
    text.value = v
    original.value = v
    dirty.value = false
    loaded.value = true
  },
  { immediate: true, flush: 'sync' },
)
watch(text, () => {
  if (loaded.value) dirty.value = text.value !== original.value
})

async function doSave() {
  if (isJson.value) {
    try {
      JSON.parse(text.value)
    } catch {
      toastError('JSON 语法错误，无法保存')
      return
    }
  }
  saving.value = true
  try {
    const res = await rpWrite({
      work_dir: props.workDir,
      rel_path: props.relPath,
      kind: 'text',
      content: text.value,
    })
    if (!res.success) {
      toastError(res.message)
      return
    }
    toastSuccess('已保存')
    original.value = text.value
    dirty.value = false
    emit('saved')
  } catch (e) {
    toastError(`保存失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <div class="space-y-2">
    <div class="flex items-center gap-2">
      <DocumentTextIcon class="h-4 w-4 shrink-0 text-gray-500" />
      <Tooltip :text="name" class="min-w-0 flex-1 truncate" overflow-only>
        <h4 class="w-full truncate text-sm font-medium text-gray-700">{{ name }}</h4>
      </Tooltip>
      <span class="min-w-0 truncate text-xs text-gray-400">{{ relPath }}</span>
    </div>
    <div v-if="dirty" class="flex items-center gap-1.5 text-xs text-amber-600">
      <ExclamationTriangleIcon class="h-3.5 w-3.5" />
      有未保存的修改
    </div>
    <div v-if="dirty && jsonError" class="rounded border border-red-200 bg-red-50 px-2 py-1 text-xs text-red-600">
      {{ jsonError }}
    </div>
    <textarea
      v-model="text"
      rows="12"
      spellcheck="false"
      class="w-full resize-y rounded border border-gray-200 bg-gray-50 p-3 font-mono text-xs text-gray-700 focus:border-blue-400 focus:outline-none"
      placeholder="文件内容..."
    ></textarea>
    <div class="flex items-center gap-2">
      <slot name="actions" />
      <Button
        type="primary"
        size="small"
        class="ml-auto"
        :loading="saving"
        :disabled="!dirty || (isJson && !!jsonError)"
        @click="doSave"
      >
        <template #icon><CheckIcon class="h-4 w-4" /></template>
        {{ saving ? '保存中…' : '保存' }}
      </Button>
    </div>
  </div>
</template>
