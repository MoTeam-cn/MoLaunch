<script setup lang="ts">
/**
 * Java 自定义模式 UI（JavaModeSelector 的 custom 模式子组件）
 *
 * - 从已检测到的 Java 列表中选择（末尾含"导入 Java"特殊项）
 * - 刷新 Java 列表按钮
 * - 未找到 Java / 不兼容警告
 *
 * 通过 useVersionSettings 共享 selectedId/personalization，自行调用保存接口。
 */
import { computed } from 'vue'
import { useJavaStore } from '@/stores/java'
import * as tauri from '@/utils/tauri'
import { isJavaCompatible } from '@/utils/api/java'
import { showSuccess, showError } from '@/utils/toast'
import Select from '@/components/common/Select.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import Button from '@/components/common/Button.vue'
import { useVersionSettings } from '@/composables/useVersionSettings'
import type { JavaRequirements } from '@/types/java'

interface Props {
  /** 当前选中的 Java 路径（v-model） */
  customJavaPath: string
  /** 刷新中状态（v-model） */
  refreshingJava: boolean
  /** Java 需求（用于兼容性检查） */
  javaReqs: JavaRequirements | null
}

const props = defineProps<Props>()
const emit = defineEmits<{
  'update:customJavaPath': [string]
  'update:refreshingJava': [boolean]
}>()

/** 导入 Java 特殊值（用于在下拉列表末尾追加"导入 Java"项） */
const IMPORT_JAVA_VALUE = '__import__'

const javaStore = useJavaStore()
const { selectedId, personalization } = useVersionSettings()

/** custom 模式下的 Java 下拉选项（末尾追加"导入 Java"特殊项） */
const javaOptionsForCustom = computed(() => {
  const opts = javaStore.javaList.map(j => {
    const compat = isJavaCompatible(j.major_version, props.javaReqs)
    return {
      value: j.executable,
      label: `Java ${j.version}（${j.major_version}${compat ? ' 兼容' : ' 不兼容'}）`,
    }
  })
  opts.push({ value: IMPORT_JAVA_VALUE, label: '导入 Java' })
  return opts
})

/** custom 模式下选中的 Java 是否兼容 */
const customJavaWarning = computed(() => {
  if (!props.customJavaPath || !props.javaReqs) return ''
  const sel = javaStore.javaList.find(j => j.executable === props.customJavaPath)
  if (!sel) return ''
  const { min_java_version: min, max_java_version: max } = props.javaReqs
  if (min && sel.major_version < min) {
    return `当前版本至少需要 Java ${min}，你选择的 Java ${sel.major_version} 不兼容，可能导致游戏崩溃`
  }
  if (max && sel.major_version > max) {
    return `当前版本最高兼容到 Java ${max}，你选择的 Java ${sel.major_version} 不兼容，可能导致游戏崩溃`
  }
  return ''
})

/** custom 模式：从已找到的 Java 列表中选择，或选择"导入 Java"项触发文件选择器 */
async function handleSelectJavaFromList(value: string) {
  if (!selectedId.value) return
  if (value === IMPORT_JAVA_VALUE) {
    await handleImportJava()
    return
  }
  emit('update:customJavaPath', value)
  try {
    await tauri.updateVersionPersonalization(selectedId.value, { javaPath: value })
    if (personalization.value) personalization.value.java_path = value
    showSuccess('Java 路径已保存')
  } catch (e) { showError('保存失败：' + String(e)) }
}

/** 手动导入 Java（选择 javaw.exe），导入后自动选中并保存 */
async function handleImportJava() {
  const filePath = await tauri.selectFile('选择 Java 可执行文件', [
    { name: 'Java 可执行文件', extensions: ['exe'] },
  ])
  if (!filePath) return
  await javaStore.refreshJava()
  const found = javaStore.javaList.find(j => j.executable === filePath)
  if (!found) {
    showError('所选文件不是有效的 Java 可执行文件')
    return
  }
  emit('update:customJavaPath', filePath)
  try {
    await tauri.updateVersionPersonalization(selectedId.value, { javaPath: filePath })
    if (personalization.value) personalization.value.java_path = filePath
    showSuccess('Java 路径已保存')
  } catch (e) { showError('保存失败：' + String(e)) }
}

/** 刷新 Java 列表 */
async function handleRefreshJavaList() {
  emit('update:refreshingJava', true)
  try {
    await javaStore.refreshJava()
    showSuccess(`已刷新 Java 列表，共找到 ${javaStore.javaList.length} 个 Java`)
  } catch (e) {
    showError('刷新 Java 列表失败：' + String(e))
  } finally {
    emit('update:refreshingJava', false)
  }
}
</script>

<template>
  <div class="space-y-2">
    <div class="flex items-center gap-2">
      <Tooltip
        :text="customJavaPath || '从已找到的 Java 中选择，或选择导入项'"
        position="top"
        :delay="0"
        class="min-w-0 flex-1"
      >
        <Select
          :model-value="customJavaPath"
          :options="javaOptionsForCustom"
          placeholder="从已找到的 Java 中选择"
          @update:model-value="(v: string) => handleSelectJavaFromList(v)"
        />
      </Tooltip>
      <Tooltip text="刷新 Java 列表" position="top" :delay="0">
        <Button
          type="outline"
          :disabled="refreshingJava"
          @click="handleRefreshJavaList"
        >
          <template #icon>
            <svg class="h-4 w-4" :class="{ 'animate-spin': refreshingJava }" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z" clip-rule="evenodd" />
            </svg>
          </template>
        </Button>
      </Tooltip>
    </div>
    <div v-if="javaStore.javaList.length === 0" class="rounded-md bg-amber-50 px-3 py-2 text-xs text-amber-600">
      未找到已安装的 Java，请选择列表中的"导入 Java"项手动选择。
    </div>
    <div v-if="customJavaWarning" class="flex items-start gap-2 rounded-md bg-red-50 px-3 py-2 text-xs text-red-600">
      <svg class="mt-0.5 h-4 w-4 flex-none" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
      </svg>
      <span>{{ customJavaWarning }}</span>
    </div>
  </div>
</template>
