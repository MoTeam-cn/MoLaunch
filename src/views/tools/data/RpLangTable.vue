<script setup lang="ts">
/**
 * 语言文件键值表格（解析 lang/*.json 原文，支持键值编辑 / 增删行 / 保存）
 */
import { computed, ref, watch, defineAsyncComponent } from 'vue'
import { CheckIcon, LanguageIcon, PlusIcon, TrashIcon, ExclamationTriangleIcon } from '@heroicons/vue/24/outline'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
import { toastError, toastSuccess } from '@/utils/toast'
import { rpWrite } from '@/utils/api/tools'

const props = defineProps<{
  workDir: string
  relPath: string
  content: string
}>()

const emit = defineEmits<{ (e: 'saved'): void }>()

interface LangEntry {
  key: string
  value: string
}

const entries = ref<LangEntry[]>([])
const dirty = ref(false)
const saving = ref(false)
const loaded = ref(false)

const total = computed(() => entries.value.length)

function parseContent() {
  loaded.value = false
  entries.value = []
  dirty.value = false
  if (!props.content) return
  try {
    const obj = JSON.parse(props.content) as Record<string, unknown>
    entries.value = Object.entries(obj).map(([k, v]) => ({
      key: k,
      value: typeof v === 'string' ? v : JSON.stringify(v),
    }))
  } catch {
    // 解析失败保持空表格
  } finally {
    loaded.value = true
  }
}

watch(() => props.content, parseContent, { immediate: true, flush: 'sync' })
watch(
  entries,
  () => {
    if (loaded.value) dirty.value = true
  },
  { deep: true },
)

function addRow() {
  entries.value.push({ key: '', value: '' })
}

function removeRow(index: number) {
  entries.value.splice(index, 1)
}

async function doSave() {
  const seen = new Set<string>()
  for (const e of entries.value) {
    if (!e.key.trim()) {
      toastError('存在空键名，请填写或删除该行')
      return
    }
    if (seen.has(e.key)) {
      toastError(`键名重复: ${e.key}`)
      return
    }
    seen.add(e.key)
  }
  saving.value = true
  try {
    const obj: Record<string, string> = {}
    for (const e of entries.value) obj[e.key] = e.value
    const res = await rpWrite({
      work_dir: props.workDir,
      rel_path: props.relPath,
      kind: 'text',
      content: JSON.stringify(obj, null, 2),
    })
    if (!res.success) {
      toastError(res.message)
      return
    }
    toastSuccess('语言文件已保存')
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
  <div class="space-y-3">
    <div class="flex items-center gap-2">
      <LanguageIcon class="h-4 w-4 text-gray-500" />
      <h4 class="text-sm font-medium text-gray-700">语言文件</h4>
      <span class="text-xs text-gray-400">{{ total }} 条键值</span>
      <Button class="ml-auto" type="outline" size="small" @click="addRow">
        <template #icon><PlusIcon class="h-4 w-4" /></template>
        新增条目
      </Button>
    </div>
    <div class="max-h-[400px] overflow-y-auto rounded border border-gray-200">
      <table class="w-full text-left text-xs">
        <thead class="sticky top-0 bg-gray-50 text-gray-500">
          <tr>
            <th class="px-3 py-2 font-medium">键</th>
            <th class="px-3 py-2 font-medium">值</th>
            <th class="w-10 px-2 py-2"></th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100">
          <tr v-for="(e, i) in entries" :key="i" class="align-top">
            <td class="px-2 py-1">
              <input
                v-model="e.key"
                class="w-full rounded border border-gray-300 px-2 py-1 font-mono text-xs text-gray-700 focus:border-blue-400 focus:outline-none"
                placeholder="键名"
              />
            </td>
            <td class="px-2 py-1">
              <input
                v-model="e.value"
                class="w-full rounded border border-gray-300 px-2 py-1 text-xs text-gray-700 focus:border-blue-400 focus:outline-none"
                placeholder="值"
              />
            </td>
            <td class="px-2 py-1">
              <button
                class="rounded p-1 text-gray-400 hover:bg-red-50 hover:text-red-500"
                title="删除"
                @click="removeRow(i)"
              >
                <TrashIcon class="h-3.5 w-3.5" />
              </button>
            </td>
          </tr>
        </tbody>
      </table>
      <p v-if="!entries.length" class="px-3 py-6 text-center text-gray-400">
        解析失败或无内容，可点击「新增条目」添加
      </p>
    </div>
    <div v-if="dirty" class="flex items-center gap-1.5 text-xs text-amber-600">
      <ExclamationTriangleIcon class="h-3.5 w-3.5" />
      有未保存的修改
    </div>
    <div class="flex justify-end">
      <Button
        type="primary"
        size="small"
        :loading="saving"
        :disabled="!dirty"
        @click="doSave"
      >
        <template #icon><CheckIcon class="h-4 w-4" /></template>
        {{ saving ? '保存中…' : '保存' }}
      </Button>
    </div>
  </div>
</template>
