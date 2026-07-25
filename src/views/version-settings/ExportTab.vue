<script setup lang="ts">
/**
 * 版本设置 - 导出子页
 *
 * 布局参考 LoaderSelect.vue：三段式 flex 列布局
 * - 中段：可滚动表单区（基本信息 + 导出选项）
 * - 底段：固定操作栏（左状态提示 + 右导出按钮 + 进度条）
 *
 * 业务逻辑抽取到 `@/composables/useExportTab`，选项列表渲染由
 * `./export-tab/ExportOptions.vue` 负责，本文件仅负责整体布局。
 */
import { onMounted, computed } from 'vue'
import {
  ArrowUpTrayIcon,
  CheckIcon,
  XMarkIcon,
  DocumentArrowDownIcon,
  DocumentArrowUpIcon,
} from '@heroicons/vue/24/outline'
import Input from '@/components/common/Input.vue'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import Select from '@/components/common/Select.vue'
import { useVersionSettings } from '@/composables/useVersionSettings'
import { useExportTab } from '@/composables/useExportTab'
import ExportOptions from './export-tab/ExportOptions.vue'

const { selectedId } = useVersionSettings()
const {
  loading,
  exporting,
  exportOptions,
  packName,
  packVersion,
  exportFormat,
  checkHostedAssets,
  modrinthUploadMode,
  exportProgress,
  exportStage,
  exportMessage,
  currentFormatMeta,
  supportsOnlineCheck,
  formatOptions,
  loadOptions,
  toggleOption,
  handleSaveConfig,
  handleLoadConfig,
  handleExport,
} = useExportTab({ selectedId })

onMounted(() => loadOptions())

/** Select 组件需要的 options 格式 */
const selectFormatOptions = computed(() =>
  formatOptions.map(o => ({ label: o.label, value: o.value })),
)

/** 阶段对应的中文文案（用于按钮文案展示） */
const stageLabel = computed(() => {
  switch (exportStage.value) {
    case 'init': return '初始化中'
    case 'scan': return '扫描文件'
    case 'network': return '联网检查'
    case 'zip': return '打包中'
    case 'done': return '导出完成'
    case 'failed': return '导出失败'
    default: return ''
  }
})

/** 导出按钮是否可点击（未导出且未在加载选项时） */
const fabClickable = computed(() => !exporting.value && !loading.value)

/** 导出按钮 tooltip 文案（导出中显示完整进度文案，其他状态显示格式提示） */
const fabTooltip = computed(() => {
  if (exporting.value) {
    return exportMessage.value || `${stageLabel.value} ${exportProgress.value}%`
  }
  if (exportStage.value === 'done') return '导出完成'
  if (exportStage.value === 'failed') return '导出失败'
  return `导出为 ${currentFormatMeta.value.label} 格式整合包（.${currentFormatMeta.value.extension}）`
})

/** 导出按钮显示文案 */
const fabLabel = computed(() => {
  if (exporting.value) return `${stageLabel.value} ${exportProgress.value}%`
  if (exportStage.value === 'done') return '导出完成'
  if (exportStage.value === 'failed') return '导出失败'
  return '导出整合包'
})

/** 底栏左侧状态提示（参考 LoaderSelect 底部 hint） */
const bottomHint = computed(() => {
  if (exporting.value) return exportMessage.value || `${stageLabel.value} ${exportProgress.value}%`
  if (exportStage.value === 'done') return '导出完成，可在保存位置查看文件'
  if (exportStage.value === 'failed') return '导出失败，请查看日志'
  if (loading.value) return '正在加载导出选项...'
  return `当前格式：${currentFormatMeta.value.label}（.${currentFormatMeta.value.extension}）`
})
</script>

<template>
  <div class="flex h-full flex-col">
    <!-- 可滚动的表单内容区 -->
    <div class="flex-1 overflow-y-auto p-6">
      <div class="mx-auto max-w-2xl space-y-5">
        <!-- 基本信息 -->
        <section class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
          <h3 class="mb-3 text-sm font-semibold text-gray-700">基本信息</h3>
          <div class="space-y-3">
            <div class="flex items-center gap-3">
              <span class="w-24 flex-none text-xs text-gray-500">导出格式</span>
              <div class="flex flex-1 flex-col gap-1">
                <Select
                  v-model="exportFormat"
                  :options="selectFormatOptions"
                  placeholder="选择导出格式"
                />
                <p class="text-xs text-gray-400">{{ currentFormatMeta.description }}</p>
              </div>
            </div>
            <div class="flex items-center gap-3">
              <span class="w-24 flex-none text-xs text-gray-500">整合包名称</span>
              <Input v-model="packName" placeholder="请输入整合包名称" class="flex-1" />
            </div>
            <div class="flex items-center gap-3">
              <span class="w-24 flex-none text-xs text-gray-500">整合包版本</span>
              <Input v-model="packVersion" placeholder="1.0.0" class="flex-1" />
            </div>
            <template v-if="supportsOnlineCheck">
              <div class="flex items-start gap-3">
                <span class="w-24 flex-none pt-1 text-xs text-gray-500">联网检查</span>
                <label class="flex flex-1 cursor-pointer items-start gap-2">
                  <input
                    v-model="checkHostedAssets"
                    type="checkbox"
                    class="mt-0.5 h-4 w-4 rounded border-gray-300 text-primary-500 focus:ring-primary-500"
                  >
                  <span class="text-xs leading-5 text-gray-600">
                    联网查询 Mod 下载地址（Modrinth + CurseForge），不勾选则直接打包文件到 overrides
                  </span>
                </label>
              </div>
              <div class="flex items-start gap-3">
                <span class="w-24 flex-none pt-1 text-xs text-gray-500">仅 Modrinth</span>
                <label class="flex flex-1 cursor-pointer items-start gap-2">
                  <input
                    v-model="modrinthUploadMode"
                    type="checkbox"
                    class="mt-0.5 h-4 w-4 rounded border-gray-300 text-primary-500 focus:ring-primary-500"
                  >
                  <span class="text-xs leading-5 text-gray-600">
                    仅从 Modrinth 查询（跳过 CurseForge），适用于准备上传到 Modrinth 的整合包
                  </span>
                </label>
              </div>
            </template>
          </div>
        </section>

        <!-- 导出选项 -->
        <section class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
          <div class="mb-3 flex items-center justify-between">
            <h3 class="text-sm font-semibold text-gray-700">导出选项</h3>
            <div class="flex gap-2">
              <Button type="text" size="small" @click="handleLoadConfig">
                <template #icon>
                  <DocumentArrowDownIcon class="h-3.5 w-3.5" />
                </template>
                读取配置
              </Button>
              <Button type="text" size="small" @click="handleSaveConfig">
                <template #icon>
                  <DocumentArrowUpIcon class="h-3.5 w-3.5" />
                </template>
                保存配置
              </Button>
            </div>
          </div>

          <div v-if="loading" class="flex h-32 flex-col items-center justify-center gap-2 text-gray-400">
            <svg class="h-8 w-8 animate-spin" viewBox="0 0 24 24" fill="none">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
            </svg>
            <p class="text-xs">加载中...</p>
          </div>

          <ExportOptions
            v-else
            :options="exportOptions"
            @toggle="toggleOption"
          />
        </section>
      </div>
    </div>

    <!-- 底部固定操作栏（参考 LoaderSelect.vue 底栏） -->
    <div class="shrink-0 border-t border-gray-300 bg-white px-6 py-4">
      <div class="mx-auto max-w-2xl">
        <!-- 进度条（仅导出中显示） -->
        <div
          v-if="exporting"
          class="mb-2 h-1 w-full overflow-hidden rounded-full bg-black/10"
        >
          <div
            class="h-full bg-primary-600 transition-all duration-300"
            :style="{ width: `${exportProgress}%` }"
          />
        </div>
        <div class="flex items-center justify-between gap-3">
          <span class="min-w-0 flex-1 truncate text-xs text-gray-400">{{ bottomHint }}</span>
          <Tooltip :text="fabTooltip" position="left">
            <Button
              type="primary"
              :loading="exporting"
              :disabled="!fabClickable"
              @click="handleExport"
            >
              <template #icon>
                <CheckIcon v-if="exportStage === 'done'" class="h-4 w-4" />
                <XMarkIcon v-else-if="exportStage === 'failed'" class="h-4 w-4" />
                <ArrowUpTrayIcon v-else class="h-4 w-4" />
              </template>
              {{ fabLabel }}
            </Button>
          </Tooltip>
        </div>
      </div>
    </div>
  </div>
</template>
