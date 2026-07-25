<script setup lang="ts">
/**
 * 自定义布局配置区（格式 / 来源 / 内联编辑器 / URL 加载 / 示例导出）
 *
 * 由 HomePanelModeSection 在 panelMode === 'custom' 时渲染。
 */
import { ref, computed, onMounted } from 'vue'
import { usePluginStore } from '@/stores/plugins'
import { writeTextFile } from '@/utils/api/system'
import { readLayoutSample } from '@/utils/api/plugins'
import { pickSavePath } from '@/utils/fileDialog'
import { toastInfo, toastSuccess, toastError, toastWarning } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import Select from '@/components/common/Select.vue'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import {
  ArrowPathIcon,
  ArrowDownTrayIcon,
  DocumentArrowDownIcon,
} from '@heroicons/vue/24/outline'

const pluginStore = usePluginStore()

/** 自定义布局配置（从 store 读取） */
const customConfig = computed(() => pluginStore.customLayoutConfig)

/** 内联编辑器占位文本（根据格式返回示例） */
const inlinePlaceholder = computed(() => {
  switch (customConfig.value.format) {
    case 'json':
      return '{\n  "title": "我的面板",\n  "sections": [\n    { "type": "text", "content": "Hello" }\n  ]\n}'
    case 'xml':
      return '<panel title="我的面板">\n  <text>Hello</text>\n</panel>'
    case 'html':
    default:
      return '<div>\n  <h3>我的面板</h3>\n  <p>Hello</p>\n</div>'
  }
})

/** 布局格式选项 */
const formatOptions = [
  { label: 'JSON（结构化布局）', value: 'json' },
  { label: 'HTML（直接渲染）', value: 'html' },
  { label: 'XML（结构化布局）', value: 'xml' },
]

/** 布局来源选项 */
const sourceOptions = [
  { label: '内联（直接编辑）', value: 'inline' },
  { label: 'URL（远程加载）', value: 'url' },
]

/** JSON/XML 内联内容编辑器（本地 ref，防抖同步到 store） */
const inlineContentDraft = ref('')
let inlineSyncTimer: ReturnType<typeof setTimeout> | null = null

/** 初始化内联内容 draft */
function initInlineDraft() {
  if (customConfig.value.source === 'inline') {
    inlineContentDraft.value = customConfig.value.inlineContent
  }
}

/** 内联内容变更（防抖 500ms 同步到 store） */
function onInlineContentChange() {
  if (inlineSyncTimer) clearTimeout(inlineSyncTimer)
  inlineSyncTimer = setTimeout(async () => {
    await pluginStore.setCustomLayoutConfig({ inlineContent: inlineContentDraft.value })
  }, 500)
}

/** URL 刷新中 */
const urlRefreshing = ref(false)

/** 刷新 URL 缓存 */
async function onRefreshUrl() {
  if (urlRefreshing.value) return
  if (!customConfig.value.url) {
    toastError('请先填写 URL 地址')
    return
  }
  urlRefreshing.value = true
  try {
    toastInfo('正在刷新布局缓存...')
    await pluginStore.refreshCustomLayoutCache()
    toastSuccess('布局缓存已刷新')
  } catch (e) {
    toastError(String(e))
  } finally {
    urlRefreshing.value = false
  }
}

/** 切换布局格式 */
async function onFormatChange(value: string | number) {
  await pluginStore.setCustomLayoutConfig({ format: String(value) as 'json' | 'html' | 'xml' })
}

/** 切换布局来源 */
async function onSourceChange(value: string | number) {
  const source = String(value) as 'inline' | 'url'
  await pluginStore.setCustomLayoutConfig({ source })
  if (source === 'inline') {
    initInlineDraft()
  }
}

/** URL 输入防抖同步 */
let urlSyncTimer: ReturnType<typeof setTimeout> | null = null
function onUrlInput(event: Event) {
  const value = (event.target as HTMLInputElement).value
  if (urlSyncTimer) clearTimeout(urlSyncTimer)
  urlSyncTimer = setTimeout(async () => {
    await pluginStore.setCustomLayoutConfig({ url: value })
  }, 500)
}

/** 缓存时间格式化 */
const cachedTimeText = computed(() => {
  if (!customConfig.value.cachedAt) return '未缓存'
  const d = new Date(customConfig.value.cachedAt)
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`
})

/** 根据当前格式从后端读取示例布局并导出 */
async function onExportSampleLayout() {
  const format = customConfig.value.format
  const ext = format
  const defaultName = `layout-sample.${ext}`
  try {
    const content = await readLayoutSample(format)
    const savePath = await pickSavePath({
      title: '保存示例布局文件',
      defaultPath: defaultName,
      filters: [{ name: `${ext.toUpperCase()} 文件`, extensions: [ext] }],
    })
    if (!savePath) return
    await writeTextFile(savePath, content)
    toastSuccess(`示例布局已导出至：${savePath}`)
  } catch (e) {
    toastError('导出示例失败：' + e)
  }
}

/**
 * 填入示例模板到内联编辑器
 *
 * 直接从后端读取当前格式的示例布局内容，填入内联编辑器并同步到 store，
 * 省去用户「导出文件 → 打开文件 → 复制内容 → 粘贴到编辑器」的繁琐流程。
 *
 * 保护逻辑：
 * - 来源为 URL 时提示先切换到内联模式（URL 模式下内联编辑器不可见）
 * - 内联编辑器已有内容时弹窗确认避免覆盖
 */
const fillingTemplate = ref(false)
async function onFillTemplate() {
  if (customConfig.value.source !== 'inline') {
    toastWarning('请先切换内容来源为「内联」模式')
    return
  }
  if (inlineContentDraft.value.trim()) {
    const confirmed = await new Promise<boolean>((resolve) => {
      showConfirm(
        '覆盖现有内容',
        '内联编辑器中已有内容，填入模板将覆盖现有内容，是否继续？',
        () => resolve(true),
        () => resolve(false),
      )
    })
    if (!confirmed) return
  }

  fillingTemplate.value = true
  try {
    const content = await readLayoutSample(customConfig.value.format)
    inlineContentDraft.value = content
    if (inlineSyncTimer) clearTimeout(inlineSyncTimer)
    await pluginStore.setCustomLayoutConfig({ inlineContent: content })
    toastSuccess('已填入示例模板')
  } catch (e) {
    toastError('填入示例失败：' + e)
  } finally {
    fillingTemplate.value = false
  }
}

onMounted(() => {
  initInlineDraft()
})
</script>

<template>
  <div class="space-y-4">
    <!-- 格式 + 来源（上下堆叠，避免并列时文字被截断） -->
    <div class="space-y-3">
      <div class="flex items-center justify-between gap-4">
        <div class="min-w-0">
          <p class="text-sm font-medium text-gray-900">布局格式</p>
          <p class="text-xs text-gray-500 mt-0.5">JSON/XML 结构化，HTML 直接渲染</p>
        </div>
        <div class="flex-none w-40">
          <Select
            :model-value="customConfig.format"
            :options="formatOptions"
            @update:model-value="onFormatChange"
          />
        </div>
      </div>
      <div class="flex items-center justify-between gap-4">
        <div class="min-w-0">
          <p class="text-sm font-medium text-gray-900">内容来源</p>
          <p class="text-xs text-gray-500 mt-0.5">内联直接编辑，URL 远程加载并缓存</p>
        </div>
        <div class="flex-none w-40">
          <Select
            :model-value="customConfig.source"
            :options="sourceOptions"
            @update:model-value="onSourceChange"
          />
        </div>
      </div>
    </div>

    <!-- 示例模板操作（填入内联编辑器 / 导出文件） -->
    <div class="flex items-center justify-between rounded border border-dashed border-gray-300 bg-white/50 px-3 py-2">
      <div class="min-w-0">
        <p class="text-xs font-medium text-gray-700">示例模板（{{ customConfig.format.toUpperCase() }}）</p>
        <p class="mt-0.5 text-[11px] text-gray-400">
          填入到内联编辑器快速开始，或导出为文件供外部编辑
        </p>
      </div>
      <div class="flex flex-none gap-2">
        <Button
          v-if="customConfig.source === 'inline'"
          type="outline"
          size="small"
          :disabled="fillingTemplate"
          @click="onFillTemplate"
        >
          <DocumentArrowDownIcon class="mr-1 h-3.5 w-3.5" />
          填入模板
        </Button>
        <Button type="outline" size="small" @click="onExportSampleLayout">
          <ArrowDownTrayIcon class="mr-1 h-3.5 w-3.5" />
          导出文件
        </Button>
      </div>
    </div>

    <!-- 内联编辑器 -->
    <div v-if="customConfig.source === 'inline'">
      <div class="mb-2 flex items-center justify-between">
        <p class="text-sm font-medium text-gray-900">
          {{ customConfig.format === 'html' ? 'HTML' : customConfig.format === 'xml' ? 'XML' : 'JSON' }} 内容
        </p>
        <span class="text-[11px] text-gray-400">编辑后自动保存（防抖 500ms）</span>
      </div>
      <Input
        v-model="inlineContentDraft"
        textarea
        :rows="16"
        resize="vertical"
        :placeholder="inlinePlaceholder"
        class="custom-layout-editor"
        @input="onInlineContentChange"
      />
    </div>

    <!-- URL 加载 -->
    <div v-else class="space-y-3">
      <div>
        <div class="mb-2 flex items-center justify-between">
          <p class="text-sm font-medium text-gray-900">布局 URL</p>
          <span class="text-[11px] text-gray-400">缓存时间：{{ cachedTimeText }}</span>
        </div>
        <div class="flex gap-2">
          <input
            :value="customConfig.url"
            type="text"
            placeholder="https://example.com/layout.json"
            class="flex-1 rounded border border-gray-300 bg-white px-3 py-1.5 text-xs text-gray-900 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
            @input="onUrlInput"
          />
          <Button
            type="outline"
            size="small"
            :disabled="urlRefreshing"
            @click="onRefreshUrl"
          >
            <template #icon><ArrowPathIcon class="w-3.5 h-3.5" :class="{ 'animate-spin': urlRefreshing }" /></template>
            刷新缓存
          </Button>
        </div>
      </div>
      <p class="text-[11px] text-gray-400">
        URL 内容会下载并缓存到本地，启动器重启后自动加载缓存；点击「刷新缓存」可强制更新
      </p>
    </div>
  </div>
</template>

<style scoped>
/* 代码编辑器：等宽字体 + 小字号，对齐原原生 textarea 的代码输入体验 */
.custom-layout-editor :deep(.textarea-inner) {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 12px;
}
</style>
