<script setup lang="ts">
/**
 * 自定义布局渲染面板
 *
 * 接收 JSON/XML 格式的布局内容，解析后渲染为启动器风格的组件。
 *
 * 工作流程：
 * 1. 解析布局内容（JSON / XML）→ LayoutSchema
 * 2. 加载数据源（cache / system / versions / history）→ DataContext
 * 3. 根据 schema 逐个渲染 section，使用 context 解析插值
 *
 * 数据刷新策略：进入页面加载一次，之后每 3 秒轮询数据源（不重新解析布局）。
 */
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { parseJsonLayout, parseXmlLayout } from './parser'
import { loadDataContext, resolveValue, formatValue, getListData, type DataContext, type ListEntry } from './datasource'
import type { LayoutSchema, LayoutSection, StatItem, ListField, ValueFormat } from './types'
import type { LayoutFormat } from '@/types/plugin'
import { toastInfo, toastSuccess, toastError, toastWarning } from '@/utils/toast'
import { showInfo, showConfirm, showPrompt } from '@/utils/modal'
import { safeCall, safeCallSync } from '@/utils/async'
import {
  ChartBarIcon,
  CircleStackIcon,
  CpuChipIcon,
  ClockIcon,
  ArrowPathIcon,
} from '@heroicons/vue/24/outline'

const props = defineProps<{
  /** 布局格式 */
  format: LayoutFormat
  /** 布局内容（JSON/XML 字符串） */
  content: string
}>()

/** 解析后的 schema */
const schema = ref<LayoutSchema | null>(null)
/** 解析错误 */
const parseError = ref<string | null>(null)
/** 数据上下文 */
const dataCtx = ref<DataContext>({})
/** 是否正在加载数据 */
const loading = ref(true)
/** 是否正在刷新 */
const refreshing = ref(false)
let pollTimer: ReturnType<typeof setInterval> | null = null

/** 解析布局内容 */
function parseLayout() {
  parseError.value = null
  schema.value = null

  const result = props.format === 'xml'
    ? parseXmlLayout(props.content)
    : parseJsonLayout(props.content)

  if (result.error) {
    parseError.value = result.error
  } else if (result.schema) {
    schema.value = result.schema
  }
}

/** 加载数据源 */
async function loadData() {
  const ctx = await safeCall(() => loadDataContext(), '[CustomLayout] load data context')
  if (ctx) dataCtx.value = ctx
  loading.value = false
  refreshing.value = false
}

/** 手动刷新 */
async function refresh() {
  refreshing.value = true
  await loadData()
}

/** 图标组件映射 */
const iconMap: Record<string, typeof ChartBarIcon> = {
  'chart-bar': ChartBarIcon,
  'circle-stack': CircleStackIcon,
  'cpu-chip': CpuChipIcon,
  'clock': ClockIcon,
}

/** 标题图标 */
const titleIcon = computed(() => {
  if (!schema.value?.icon) return null
  return iconMap[schema.value.icon] ?? null
})

/** 颜色主题映射 */
const colorClassMap: Record<string, string> = {
  primary: 'text-primary-600',
  green: 'text-green-600',
  yellow: 'text-yellow-600',
  red: 'text-red-600',
  gray: 'text-gray-700',
}

const progressBarColorMap: Record<string, string> = {
  primary: 'bg-primary-500',
  green: 'bg-green-500',
  yellow: 'bg-yellow-500',
  red: 'bg-red-500',
  gray: 'bg-gray-500',
}

const textVariantMap: Record<string, string> = {
  default: 'text-gray-700',
  muted: 'text-gray-400',
  warning: 'text-yellow-600',
}

// ==================== 渲染辅助函数 ====================

/** 解析 stat-grid 项的值 */
function resolveStatValue(item: StatItem): string {
  const raw = resolveValue(item.value, dataCtx.value)
  return formatValue(raw, item.format)
}

/** 解析进度条当前值 */
function resolveProgressValue(expr: string): number {
  const val = resolveValue(expr, dataCtx.value)
  return typeof val === 'number' ? val : parseFloat(String(val)) || 0
}

/** 解析进度条最大值（默认 100） */
function resolveProgressMax(expr?: string): number {
  if (!expr) return 100
  const val = resolveValue(expr, dataCtx.value)
  return typeof val === 'number' ? val : parseFloat(String(val)) || 100
}

/** 进度条百分比（0-100） */
function progressPercent(section: Extract<LayoutSection, { type: 'progress' }>): number {
  const val = resolveProgressValue(section.value)
  const max = resolveProgressMax(section.max)
  if (max <= 0) return 0
  return Math.min(100, Math.max(0, (val / max) * 100))
}

/** 列表数据 */
function getListEntries(section: Extract<LayoutSection, { type: 'list' }>): ListEntry[] {
  return getListData(section.source, dataCtx.value)
}

/** 格式化列表字段值 */
function formatFieldValue(entry: ListEntry, field: ListField): string {
  const raw = entry[field.key]
  if (raw === undefined || raw === null) return '-'
  return formatValue(raw, field.format)
}

/**
 * 内置设计系统 CSS（注入到 html section 的 iframe 中）
 *
 * 提供与启动器主界面一致的视觉风格，开发者可直接使用 .btn / .card / .stat 等类名。
 */
const DESIGN_SYSTEM_CSS = `
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;font-size:13px;color:#1f2937;background:transparent;padding:12px}
h1{font-size:18px;font-weight:600;margin-bottom:8px}
h2{font-size:16px;font-weight:600;margin-bottom:6px}
h3{font-size:14px;font-weight:600;margin-bottom:4px}
p{font-size:12px;color:#6b7280;margin-bottom:4px}
/* 按钮 */
.btn{display:inline-flex;align-items:center;gap:4px;padding:6px 12px;border:1px solid #d1d5db;border-radius:4px;background:#fff;color:#374151;font-size:12px;cursor:pointer;transition:background .15s}
.btn:hover{background:#f3f4f6}
.btn-primary{background:#6366f1;border-color:#6366f1;color:#fff}
.btn-primary:hover{background:#4f46e5}
.btn-sm{padding:4px 8px;font-size:11px}
/* 卡片 */
.card{border:1px solid #e5e7eb;border-radius:6px;padding:12px;background:#fff}
.card-title{font-size:12px;font-weight:600;color:#6b7280;margin-bottom:8px}
/* 统计卡片 */
.stat{display:flex;flex-direction:column;gap:2px}
.stat-label{font-size:11px;color:#9ca3af}
.stat-value{font-size:20px;font-weight:700;color:#111827}
.stat-suffix{font-size:12px;color:#6b7280}
/* 网格 */
.grid{display:grid;gap:12px}
.grid-2{grid-template-columns:repeat(2,1fr)}
.grid-3{grid-template-columns:repeat(3,1fr)}
/* 进度条 */
.progress-bar{height:8px;border-radius:4px;background:#e5e7eb;overflow:hidden}
.progress-fill{height:100%;border-radius:4px;background:#6366f1;transition:width .3s}
.progress-fill.green{background:#10b981}
.progress-fill.yellow{background:#f59e0b}
.progress-fill.red{background:#ef4444}
/* 徽章 */
.badge{display:inline-flex;align-items:center;padding:2px 6px;border-radius:3px;font-size:10px;font-weight:500}
.badge-primary{background:#eef2ff;color:#4338ca}
.badge-green{background:#ecfdf5;color:#047857}
.badge-red{background:#fef2f2;color:#b91c1c}
.badge-gray{background:#f3f4f6;color:#6b7280}
/* 文本工具类 */
.text-muted{color:#9ca3af}
.text-sm{font-size:11px}
.text-lg{font-size:15px}
.text-bold{font-weight:600}
.text-center{text-align:center}
.flex{display:flex}
.items-center{align-items:center}
.justify-between{justify-content:space-between}
.gap-2{gap:8px}
.gap-4{gap:16px}
.mt-2{margin-top:8px}
.mt-4{margin-top:16px}
.mb-2{margin-bottom:8px}
.mb-4{margin-bottom:16px}
`

/**
 * 确保 window.molaunch API 已定义
 *
 * shadow DOM 方案中无 iframe，用户脚本直接在主窗口上下文执行，
 * 因此 window.molaunch 直接调用前端组件，无需 postMessage 桥接。
 */
let molaunchApiReady = false
function setupMolaunchApi() {
  if (molaunchApiReady) return
  molaunchApiReady = true

  const molaunch = {
    toast(type: string, text: string) {
      if (type === 'success') toastSuccess(text)
      else if (type === 'error') toastError(text)
      else if (type === 'warning') toastWarning(text)
      else toastInfo(text)
    },
    alert(title: string, message: string) {
      showInfo(title, message)
    },
    confirm(title: string, message: string): Promise<boolean> {
      return new Promise((resolve) => {
        showConfirm(title, message, () => resolve(true), () => resolve(false))
      })
    },
    prompt(title: string, message: string, defaultValue = ''): Promise<string | null> {
      return new Promise((resolve) => {
        showPrompt(title, message, (value: string) => resolve(value), { defaultValue, onCancel: () => resolve(null) })
      })
    },
  }

  ;(window as Record<string, unknown>).molaunch = molaunch
}

/**
 * 用 shadow DOM 渲染 html section
 *
 * 替代 iframe 方案，消除 sandbox="allow-scripts allow-same-origin" 安全警告。
 * shadow DOM 提供 CSS 隔离（设计系统 CSS 不泄漏到主页面），用户脚本通过 new Function 执行。
 */
function renderHtmlShadow(container: HTMLElement, section: Extract<LayoutSection, { type: 'html' }>) {
  // 内容指纹：避免相同内容重复渲染
  const key = section.content + '\0' + (section.script || '') + '\0' + (section.style || '')
  if (container.dataset.renderedKey === key) return
  container.dataset.renderedKey = key

  // 获取或创建 shadow root
  let shadow = container.shadowRoot
  if (!shadow) {
    shadow = container.attachShadow({ mode: 'open' })
  }
  shadow.innerHTML = ''

  // 注入设计系统 CSS
  const styleEl = document.createElement('style')
  styleEl.textContent = DESIGN_SYSTEM_CSS
  shadow.appendChild(styleEl)

  // 注入用户自定义样式
  if (section.style) {
    const userStyle = document.createElement('style')
    userStyle.textContent = section.style
    shadow.appendChild(userStyle)
  }

  // 注入用户 HTML
  const wrapper = document.createElement('div')
  wrapper.innerHTML = section.content
  shadow.appendChild(wrapper)

  // 确保 window.molaunch API 可用
  setupMolaunchApi()

  // 执行用户脚本
  if (section.script) {
    safeCallSync(() => new Function(section.script)(), '[CustomLayout] run html section script')
  }
}

// ==================== 生命周期 ====================

watch(() => [props.format, props.content], () => {
  parseLayout()
  loading.value = true
  loadData()
}, { immediate: false })

onMounted(() => {
  parseLayout()
  loadData()
  // 每 3 秒轮询数据源（不重新解析布局）
  pollTimer = setInterval(loadData, 3000)
})

onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer)
})
</script>

<template>
  <div class="flex h-full flex-col p-6">
    <!-- 解析错误 -->
    <div v-if="parseError" class="flex flex-1 flex-col items-center justify-center">
      <div class="mb-3 rounded-full bg-red-50 p-3">
        <svg class="h-8 w-8 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
        </svg>
      </div>
      <p class="text-sm font-medium text-gray-900">布局解析失败</p>
      <p class="mt-1 max-w-md text-center text-xs text-gray-500">{{ parseError }}</p>
    </div>

    <!-- 加载中 -->
    <div v-else-if="loading" class="flex flex-1 items-center justify-center text-sm text-gray-500">
      加载中...
    </div>

    <!-- 正常渲染 -->
    <template v-else-if="schema">
      <!-- 标题栏（固定） -->
      <div v-if="schema.title" class="flex flex-none items-center justify-between mb-4">
        <div class="flex items-center gap-2">
          <component
            :is="titleIcon"
            v-if="titleIcon"
            class="h-5 w-5 text-primary-500"
          />
          <h3 class="text-base font-semibold text-gray-900">{{ schema.title }}</h3>
        </div>
        <button
          class="inline-flex items-center gap-1 rounded px-2 py-1 text-xs text-gray-500 hover:bg-gray-100 hover:text-gray-700"
          :disabled="refreshing"
          @click="refresh"
        >
          <ArrowPathIcon class="h-3.5 w-3.5" :class="{ 'animate-spin': refreshing }" />
          刷新
        </button>
      </div>

      <!-- sections（可滚动） -->
      <div class="flex-1 space-y-4 overflow-y-auto pr-1">
        <template v-for="(section, idx) in schema.sections" :key="idx">
          <!-- 统计网格 -->
          <div
            v-if="section.type === 'stat-grid'"
            class="grid gap-3"
            :style="{ gridTemplateColumns: `repeat(${section.columns || 3}, minmax(0, 1fr))` }"
          >
            <div
              v-for="(item, itemIdx) in section.items"
              :key="itemIdx"
              class="rounded-md border border-gray-200 p-3"
            >
              <p class="text-[11px] text-gray-500">{{ item.label }}</p>
              <p
                class="mt-1 text-lg font-semibold"
                :class="item.color ? colorClassMap[item.color] : 'text-gray-900'"
              >
                {{ resolveStatValue(item) }}
              </p>
            </div>
          </div>

          <!-- 数据列表 -->
          <div
            v-else-if="section.type === 'list'"
            class="rounded-md border border-gray-200 p-4"
          >
            <div v-if="section.title" class="mb-3 flex items-center justify-between">
              <span class="text-sm font-medium text-gray-900">{{ section.title }}</span>
              <span class="text-xs text-gray-500">{{ getListEntries(section).length }} 条</span>
            </div>
            <div class="space-y-2">
              <div
                v-for="(entry, entryIdx) in getListEntries(section)"
                :key="entryIdx"
                class="flex items-center justify-between rounded bg-gray-50 px-3 py-2"
              >
                <div class="min-w-0 flex-1">
                  <template v-for="(field, fieldIdx) in section.fields" :key="fieldIdx">
                    <span
                      v-if="fieldIdx > 0"
                      class="mx-1.5 text-gray-300"
                    >·</span>
                    <span v-if="field.label" class="text-[10px] text-gray-400">{{ field.label }}: </span>
                    <span class="text-xs text-gray-900">{{ formatFieldValue(entry, field) }}</span>
                  </template>
                </div>
              </div>
              <p
                v-if="getListEntries(section).length === 0"
                class="py-2 text-center text-xs text-gray-400"
              >
                暂无数据
              </p>
            </div>
          </div>

          <!-- 进度条 -->
          <div
            v-else-if="section.type === 'progress'"
            class="rounded-md border border-gray-200 p-4"
          >
            <div v-if="section.label" class="mb-2 flex items-center justify-between">
              <span class="text-xs font-medium text-gray-900">{{ section.label }}</span>
              <span class="text-xs text-gray-500">
                {{ formatValue(resolveProgressValue(section.value), section.format || 'text') }}
                <span v-if="section.max"> / {{ formatValue(resolveProgressMax(section.max), section.format || 'text') }}</span>
              </span>
            </div>
            <div v-else class="mb-2 flex items-center justify-end">
              <span class="text-xs text-gray-500">
                {{ formatValue(resolveProgressValue(section.value), section.format || 'text') }}
                <span v-if="section.max"> / {{ formatValue(resolveProgressMax(section.max), section.format || 'text') }}</span>
              </span>
            </div>
            <div class="h-2 w-full overflow-hidden rounded-full bg-gray-100">
              <div
                class="h-full transition-all duration-500"
                :class="section.color ? progressBarColorMap[section.color] : 'bg-primary-500'"
                :style="{ width: `${progressPercent(section)}%` }"
              />
            </div>
          </div>

          <!-- 文本块 -->
          <p
            v-else-if="section.type === 'text'"
            class="text-xs"
            :class="section.variant ? textVariantMap[section.variant] : 'text-gray-700'"
          >
            {{ section.content }}
          </p>

          <!-- 分割线 -->
          <hr v-else-if="section.type === 'divider'" class="border-gray-200" />

          <!-- 自定义 HTML（shadow DOM 渲染，CSS 隔离 + 内联 JS/CSS 支持） -->
          <div
            v-else-if="section.type === 'html'"
            :ref="(el) => { if (el) renderHtmlShadow(el as HTMLElement, section) }"
            :style="{ height: (section.height || 200) + 'px' }"
            class="w-full overflow-hidden rounded-md border border-gray-200"
          />
        </template>
      </div>
    </template>
  </div>
</template>
