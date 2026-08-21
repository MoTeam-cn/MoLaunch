<template>
  <Drawer
    :visible="visible"
    placement="right"
    :width="560"
    render-in-place
    popup-container="#app-content"
    :undo-ms="3000"
    @update:visible="visible = $event"
  >
    <template #title>
      <div class="flex items-center gap-1.5">
        <ExclamationTriangleIcon class="h-4 w-4 text-amber-500" />
        <span>游戏运行出错</span>
      </div>
    </template>

    <!-- 崩溃原因概要：按类别着色，左侧色条区分 -->
    <div class="rounded-md border-l-4 py-3 pl-3.5 pr-3" :class="bannerClass">
      <Tag :color="categoryTagColor" size="small">{{ categoryLabel }}</Tag>
      <p class="mt-2 text-sm font-medium leading-relaxed break-all text-gray-900">{{ reason }}</p>
      <p
        v-if="problematicMod"
        class="mt-2 flex items-start gap-1 text-xs leading-relaxed text-gray-500"
      >
        <span class="shrink-0 font-medium">相关 Mod：</span>
        <span class="break-all">{{ problematicMod }}</span>
      </p>
    </div>

    <!-- 解决方案 -->
    <div v-if="suggestion" class="mt-5">
      <p class="mb-1.5 text-xs font-medium text-gray-500">解决方案</p>
      <div class="flex items-start gap-2.5 rounded-md border border-primary-100 bg-primary-50 px-3 py-2.5">
        <LightBulbIcon class="mt-0.5 h-4 w-4 shrink-0 text-primary-500" />
        <div class="min-w-0 flex-1 space-y-1.5">
          <p
            v-for="(line, index) in suggestionLines"
            :key="index"
            class="text-sm leading-relaxed text-gray-700"
          >
            {{ line }}
          </p>
        </div>
      </div>
    </div>

    <!-- 崩溃报告文件 -->
    <div v-if="crashReportPath" class="mt-5">
      <p class="mb-1.5 text-xs font-medium text-gray-500">崩溃报告</p>
      <div class="flex items-center gap-2 rounded-md border border-gray-200 bg-gray-50 px-3 py-2">
        <DocumentTextIcon class="h-4 w-4 shrink-0 text-gray-400" />
        <span class="min-w-0 flex-1 truncate font-mono text-xs text-gray-600">{{ crashReportPath }}</span>
        <Button type="text" size="mini" @click="openCrashReport">打开</Button>
      </div>
    </div>

    <!-- 日志详情（可折叠） -->
    <div v-if="hasLogDetails" class="mt-5">
      <Button type="text" size="mini" @click="showDetails = !showDetails">
        <template #icon>
          <ChevronRightIcon
            class="h-3.5 w-3.5 transition-transform"
            :class="{ 'rotate-90': showDetails }"
          />
        </template>
        查看日志详情（{{ logLineCount }} 行）
      </Button>

      <Collapse :open="showDetails">
        <div class="mt-2.5 space-y-3">
          <div v-if="errorLines.length > 0">
            <p class="mb-1.5 text-xs text-gray-400">错误日志（{{ errorLines.length }} 行）</p>
            <pre class="max-h-52 overflow-y-auto rounded-lg bg-gray-900 p-3 font-mono text-xs leading-5 whitespace-pre-wrap break-all text-red-300">{{ errorLines.join('\n') }}</pre>
          </div>
          <div v-if="logTail.length > 0">
            <p class="mb-1.5 text-xs text-gray-400">游戏日志尾部（{{ logTail.length }} 行）</p>
            <pre class="max-h-52 overflow-y-auto rounded-lg bg-gray-900 p-3 font-mono text-xs leading-5 whitespace-pre-wrap break-all text-gray-300">{{ logTail.join('\n') }}</pre>
          </div>
        </div>
      </Collapse>
    </div>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button v-if="crashReportPath" type="outline" size="small" @click="openCrashReport">
          查看报告
        </Button>
        <Button type="ghost" size="small" @click="exportReport">导出报告</Button>
        <div class="relative">
          <Button
            type="ghost"
            size="small"
            :loading="sharing !== null"
            @click="shareMenuOpen = !shareMenuOpen"
          >
            分享日志
          </Button>
          <div
            v-if="shareMenuOpen"
            class="absolute bottom-full right-0 z-20 mb-1 w-60 rounded-md border border-gray-200 bg-white p-1 shadow-lg"
            @mouseleave="shareMenuOpen = false"
          >
            <button
              v-for="item in SHARE_PROVIDERS"
              :key="item.value"
              class="flex w-full items-start gap-2 rounded px-2 py-1.5 text-left text-xs hover:bg-gray-50"
              @click="shareTo(item.value)"
            >
              <div class="min-w-0">
                <p class="font-medium text-gray-700">{{ item.label }}</p>
                <p class="text-gray-400">{{ item.desc }}</p>
              </div>
            </button>
          </div>
        </div>
        <Button type="primary" size="small" @click="visible = false">关闭</Button>
      </div>
    </template>
  </Drawer>
</template>

<script setup lang="ts">
import { computed, ref, defineAsyncComponent } from 'vue'
import {
  ChevronRightIcon,
  DocumentTextIcon,
  ExclamationTriangleIcon,
  LightBulbIcon,
} from '@heroicons/vue/24/outline'
const Drawer = defineAsyncComponent(() => import('@/components/common/Drawer.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const Collapse = defineAsyncComponent(() => import('@/components/common/Collapse.vue'))
import { pickSavePath } from '@/utils/fileDialog'
import { openPath, writeTextFile } from '@/utils/api/system'
import { toastError, toastSuccess } from '@/utils/toast'
import { open } from '@tauri-apps/plugin-shell'
import { sanitizeShareLog, uploadLogShare } from '@/utils/logShare'
import type { LogShareProvider } from '@/utils/logShare'
import type { CrashCategory, CrashInfo } from '@/types/version'

/** 崩溃类别展示元数据：标签文案、Tag 颜色、原因横幅的左侧色条与底色 */
interface CategoryMeta {
  label: string
  tagColor: string
  bannerClass: string
}

const CATEGORY_META: Record<CrashCategory, CategoryMeta> = {
  Java: { label: 'Java 环境', tagColor: 'blue', bannerClass: 'border-blue-400 bg-blue-50' },
  Memory: { label: '内存不足', tagColor: 'red', bannerClass: 'border-red-400 bg-red-50' },
  Graphics: { label: '显卡 / 渲染', tagColor: 'purple', bannerClass: 'border-purple-400 bg-purple-50' },
  Mod: { label: 'Mod 冲突', tagColor: 'orangered', bannerClass: 'border-orange-400 bg-orange-50' },
  Forge: { label: 'Forge', tagColor: 'orange', bannerClass: 'border-orange-400 bg-orange-50' },
  Fabric: { label: 'Fabric', tagColor: 'gold', bannerClass: 'border-yellow-400 bg-yellow-50' },
  OptiFine: { label: 'OptiFine', tagColor: 'green', bannerClass: 'border-green-400 bg-green-50' },
  ResourcePack: { label: '资源包', tagColor: 'cyan', bannerClass: 'border-cyan-400 bg-cyan-50' },
  Shader: { label: '光影', tagColor: 'cyan', bannerClass: 'border-cyan-400 bg-cyan-50' },
  Unknown: { label: '未知原因', tagColor: 'gray', bannerClass: 'border-gray-300 bg-gray-50' },
}

const visible = ref(false)
const showDetails = ref(false)
const crashInfo = ref<CrashInfo | null>(null)
const shareMenuOpen = ref(false)
const sharing = ref<LogShareProvider | null>(null)

/** 日志分享服务选项 */
const SHARE_PROVIDERS: { value: LogShareProvider; label: string; desc: string }[] = [
  { value: 'mclogs', label: 'mclo.gs', desc: '国际主流日志分享，自带分析' },
  { value: 'logshare', label: 'logshare.cn', desc: '国内访问快，支持 AI 分析' },
]

/** 未展示崩溃数据时的兜底值 */
const EMPTY_CRASH: CrashInfo = {
  reason: '',
  category: 'Unknown',
  log_lines: [],
  suggestion: '',
  problematic_mod: null,
  log_tail: [],
}

/** 最近一次崩溃数据（未展示时使用兜底值，避免 null 解构） */
const lastCrash = computed(() => crashInfo.value ?? EMPTY_CRASH)

const errorLines = computed(() => lastCrash.value.log_lines ?? [])
const logTail = computed(() => lastCrash.value.log_tail ?? [])
const logLineCount = computed(() => errorLines.value.length + logTail.value.length)
const hasLogDetails = computed(() => logLineCount.value > 0)
const reason = computed(() => lastCrash.value.reason || '未知原因')
const suggestion = computed(() => lastCrash.value.suggestion?.trim() ?? '')

/**
 * 建议文本拆分为多行展示：
 * 规则引擎建议以 \n 分行；AI 长文本常把「建议：N.」多条方案挤在同一行，
 * 这里依次按 换行 / 「。建议：」/「建议：N.」编号 /「；」拆分，保证每条方案独立成行。
 */
const suggestionLines = computed(() => {
  const text = suggestion.value
  if (!text) return []
  return text
    .replace(/。\s*建议[:：]/g, '。\n建议：')
    .replace(/(建议[:：])\s*(\d+[.、．])/g, '$1\n$2')
    .split('\n')
    .flatMap((line) => line.split('；'))
    .map((line) => line.trim())
    .filter(Boolean)
})
const problematicMod = computed(() => lastCrash.value.problematic_mod ?? null)
const crashReportPath = computed(() => lastCrash.value.crash_report_path ?? null)

const category = computed<CrashCategory>(() => lastCrash.value.category ?? 'Unknown')
const categoryMeta = computed(() => CATEGORY_META[category.value] ?? CATEGORY_META.Unknown)
const categoryLabel = computed(() => categoryMeta.value.label)
const categoryTagColor = computed(() => categoryMeta.value.tagColor)
const bannerClass = computed(() => categoryMeta.value.bannerClass)

function show(info: CrashInfo) {
  crashInfo.value = info
  showDetails.value = false
  visible.value = true
}

/** 打开崩溃报告文件（复用系统 shell 模块） */
async function openCrashReport() {
  if (!crashReportPath.value) return
  try {
    await openPath(crashReportPath.value)
  } catch (e) {
    toastError('打开文件失败：' + String(e))
  }
}

/** 导出错误报告到本地文件 */
async function exportReport() {
  const path = await pickSavePath({ defaultPath: 'error-report.txt' })
  if (!path) return
  try {
    await writeTextFile(path, buildReportContent())
    toastSuccess('错误报告已导出')
  } catch (e) {
    toastError('导出失败：' + String(e))
  }
}

/** 组装错误报告文本（日志 + 崩溃信息） */
function buildReportContent(): string {
  const c = lastCrash.value
  const lines = [
    '==== MoLaunch 错误报告 ====',
    '',
    `类别：${category.value}`,
    `原因：${c.reason ?? ''}`,
    c.problematic_mod ? `相关 Mod：${c.problematic_mod}` : '',
    '\n== 错误日志 ==',
    ...(c.log_lines ?? []),
    '\n== 游戏日志尾部 ==',
    ...(c.log_tail ?? []),
  ]
  return lines.filter(Boolean).join('\n')
}

/** 组装分享用日志（以日志主体为主，便于第三方平台分析） */
function buildShareContent(): string {
  const c = lastCrash.value
  const lines = [
    `==== MoLaunch 崩溃日志（${category.value}） ====`,
    ...(c.log_lines ?? []),
    ...(c.log_tail ?? []),
  ]
  return lines.filter(Boolean).join('\n')
}

/** 分享日志到云端服务：脱敏 → 上传 → 打开分享页 */
async function shareTo(provider: LogShareProvider) {
  shareMenuOpen.value = false
  sharing.value = provider
  try {
    const url = await uploadLogShare(sanitizeShareLog(buildShareContent()), provider)
    await open(url)
    toastSuccess('日志已分享，已打开分享页面')
  } catch (e) {
    toastError('分享失败：' + String(e))
  } finally {
    sharing.value = null
  }
}

defineExpose({ show })
</script>
