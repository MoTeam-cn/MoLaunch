<script setup lang="ts">
/**
 * pack.mcmeta 编辑表单（pack_format / 描述）
 *
 * 解析包元信息 JSON 原文，编辑 pack_format 与 description 后写回；
 * 保留原 JSON 中其他顶层字段（overlays / language 等）不破坏。
 */
import { computed, ref, watch, defineAsyncComponent, onMounted } from 'vue'
import { CheckIcon, CubeIcon, ExclamationTriangleIcon } from '@heroicons/vue/24/outline'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
import { toastError, toastSuccess } from '@/utils/toast'
import { rpPackFormatInfo, rpVersionPackFormat, rpWrite } from '@/utils/api/tools'
import { listInstalledVersionsWithType, type InstalledVersionInfo } from '@/utils/api/version'

const props = defineProps<{
  workDir: string
  relPath: string
  /** pack.mcmeta 原文（rp_read 返回） */
  content: string
  mcVersion: string | null
}>()

const emit = defineEmits<{
  (e: 'saved', meta: { packFormat: number; description: string | null }): void
}>()

const packFormat = ref('')
const description = ref('')
/** 加载时的初始值快照，用于判定是否真正有改动 */
const originalPackFormat = ref('')
const originalDescription = ref('')
const dirty = ref(false)
const saving = ref(false)
const loaded = ref(false)
/** 实时查询的 pack_format → MC 版本（输入变化时异步回填） */
const liveVersion = ref<string | null>(null)
const versionKnown = ref(true)
/** 已安装版本（下拉选版本 → 自动推导 pack_format） */
const installedVersions = ref<InstalledVersionInfo[]>([])
const selectedMcVersion = ref('')
const deriving = ref(false)
const versionOptions = computed(() =>
  installedVersions.value.map((v) => ({ label: v.id, value: v.id })),
)

const validFormat = computed(() => {
  const n = Number(packFormat.value)
  return packFormat.value.trim() !== '' && Number.isInteger(n) && n >= 0
})

/** 展示的适用版本：实时查询结果优先，无则用打开包时返回的 mcVersion */
const displayVersion = computed(() => liveVersion.value ?? props.mcVersion ?? '未知')

watch(packFormat, async (v) => {
  liveVersion.value = null
  versionKnown.value = true
  if (!validFormat.value) return
  const n = Number(v)
  try {
    const res = await rpPackFormatInfo(n)
    if (res.error) return
    liveVersion.value = res.mc_version
    versionKnown.value = res.known
  } catch {
    // 查询失败保持默认展示
  }
})

/** 选中已安装版本 → 后端推导 pack_format 回填输入框 */
watch(selectedMcVersion, async (v) => {
  if (!v) return
  deriving.value = true
  try {
    const res = await rpVersionPackFormat(v)
    if (res.error || !res.known) return
    packFormat.value = String(res.pack_format)
  } catch {
    // 推导失败保持原值
  } finally {
    deriving.value = false
  }
})

onMounted(async () => {
  try {
    installedVersions.value = await listInstalledVersionsWithType()
  } catch {
    // 版本列表加载失败则隐藏推导下拉
  }
})

function parseContent() {
  loaded.value = false
  packFormat.value = ''
  description.value = ''
  dirty.value = false
  if (!props.content) return
  try {
    const obj = JSON.parse(props.content) as { pack?: { pack_format?: unknown; description?: unknown } }
    const pack = obj.pack
    if (pack && typeof pack.pack_format === 'number') {
      packFormat.value = String(pack.pack_format)
    }
    if (pack && typeof pack.description === 'string') {
      description.value = pack.description
    }
  } catch {
    // 解析失败保持空表单
  } finally {
    originalPackFormat.value = packFormat.value
    originalDescription.value = description.value
    loaded.value = true
  }
}

watch(() => props.content, parseContent, { immediate: true, flush: 'sync' })
watch([packFormat, description], () => {
  if (loaded.value) {
    dirty.value =
      packFormat.value !== originalPackFormat.value || description.value !== originalDescription.value
  }
})

async function doSave() {
  if (!validFormat.value) {
    toastError('pack_format 必须为非负整数')
    return
  }
  saving.value = true
  try {
    let obj: Record<string, unknown> = {}
    try {
      obj = JSON.parse(props.content || '{}') as Record<string, unknown>
    } catch {
      obj = {}
    }
    const pack = (obj.pack ??= {}) as { pack_format?: unknown; description?: unknown }
    pack.pack_format = Number(packFormat.value)
    pack.description = description.value
    const text = JSON.stringify(obj, null, 2)
    const res = await rpWrite({
      work_dir: props.workDir,
      rel_path: props.relPath,
      kind: 'text',
      content: text,
    })
    if (!res.success) {
      toastError(res.message)
      return
    }
    toastSuccess('pack.mcmeta 已保存')
    originalPackFormat.value = packFormat.value
    originalDescription.value = description.value
    dirty.value = false
    emit('saved', { packFormat: Number(packFormat.value), description: description.value })
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
      <CubeIcon class="h-4 w-4 text-gray-500" />
      <h4 class="text-sm font-medium text-gray-700">pack.mcmeta</h4>
      <span class="text-xs text-gray-400">包元信息</span>
    </div>
    <div class="overflow-hidden rounded border border-gray-200">
      <div class="flex border-b border-gray-200">
        <div class="w-32 shrink-0 bg-gray-50 px-3 py-2 text-xs text-gray-500">pack_format</div>
        <div class="flex-1 px-3 py-1.5">
          <input
            v-model="packFormat"
            type="number"
            min="0"
            step="1"
            class="w-full rounded border border-gray-300 px-2 py-1 text-sm text-gray-700 focus:border-blue-400 focus:outline-none"
            placeholder="如 15"
          />
        </div>
      </div>
      <div v-if="versionOptions.length" class="flex border-b border-gray-200">
        <div class="w-32 shrink-0 bg-gray-50 px-3 py-2 text-xs text-gray-500">按版本推导</div>
        <div class="flex flex-1 items-center gap-2 px-3 py-1.5">
          <Select
            v-model="selectedMcVersion"
            :options="versionOptions"
            placeholder="选择 MC 版本自动填充 pack_format"
            class="w-full"
          />
          <span v-if="deriving" class="shrink-0 text-xs text-gray-400">推导中…</span>
        </div>
      </div>
      <div class="flex border-b border-gray-200">
        <div class="w-32 shrink-0 bg-gray-50 px-3 py-2 text-xs text-gray-500">适用版本</div>
        <div class="flex-1 px-3 py-2 text-sm" :class="versionKnown ? 'text-gray-700' : 'text-red-600'">
          {{ displayVersion }}
          <span v-if="!versionKnown" class="ml-2 text-xs">（未知 pack_format，请核对）</span>
        </div>
      </div>
      <div class="flex">
        <div class="w-32 shrink-0 bg-gray-50 px-3 py-2 text-xs text-gray-500">描述</div>
        <div class="flex-1 px-3 py-1.5">
          <textarea
            v-model="description"
            rows="2"
            class="w-full rounded border border-gray-300 px-2 py-1 text-sm text-gray-700 focus:border-blue-400 focus:outline-none"
            placeholder="资源包描述"
          ></textarea>
        </div>
      </div>
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
        :disabled="!dirty || !validFormat"
        @click="doSave"
      >
        <template #icon><CheckIcon class="h-4 w-4" /></template>
        {{ saving ? '保存中…' : '保存' }}
      </Button>
    </div>
  </div>
</template>
