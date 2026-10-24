<script setup lang="ts">
/**
 * 崩溃日志分析
 *
 * 粘贴崩溃日志 → 本地规则引擎（关键词/正则检索 → 规则映射 → 诊断输出）产出结论。
 * 每条结论都带置信度与证据行（行号 + 原文）；拿不到证据的进「待确认」区，不下结论。
 */
import { ref, computed, defineAsyncComponent } from 'vue'
import { CommandLineIcon, MagnifyingGlassIcon, ExclamationCircleIcon, SparklesIcon } from '@heroicons/vue/24/outline'
import { open } from '@tauri-apps/plugin-shell'
import { toastSuccess, toastError, toastInfo } from '@/utils/toast'
import { crashAnalyze, crashRuleFeedback } from '@/utils/api/tools'
import type { CrashAnalyzeResult, CrashAnalysisItem } from '@/utils/api/tools'
import CrashAnalysisCard from './crash-analysis/CrashAnalysisCard.vue'

const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))

const emit = defineEmits<{ 'ai-followup': [logText: string] }>()

const logText = ref('')
const loading = ref(false)
const result = ref<CrashAnalyzeResult | null>(null)
const showPending = ref(false)

const canAnalyze = computed(() => logText.value.trim().length > 0)
const analyses = computed(() => result.value?.analyses ?? [])
const pending = computed(() => result.value?.pending ?? [])

async function runAnalyze() {
  if (!canAnalyze.value) {
    toastError('请先粘贴崩溃日志内容')
    return
  }
  loading.value = true
  result.value = null
  showPending.value = false
  try {
    result.value = await crashAnalyze(logText.value)
    const items = result.value.analyses
    const hasSpecific = items.some((it) => it.category !== 'other' && it.severity !== 'info')
    if (items.length === 0) {
      toastInfo('本地引擎未识别出具体问题，可点击「用 AI 深度分析」进一步诊断')
    } else if (!hasSpecific) {
      toastInfo('只识别到通用信息，可点击「用 AI 深度分析」进一步诊断')
    } else {
      toastSuccess('分析完成，识别出 ' + items.length + ' 个可能原因')
    }
  } catch (e) {
    toastError(`分析失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    loading.value = false
  }
}

/** 标记这条判断有误（只写本地日志，不上报） */
async function reportWrong(item: CrashAnalysisItem) {
  if (!item.rule_id) {
    toastInfo('该结论来自通用兜底逻辑，暂无规则 id')
    return
  }
  try {
    await crashRuleFeedback(item.rule_id, item.category)
    toastInfo('已记录到本地日志，反馈问题时附上启动器日志即可帮我们修正这条规则')
  } catch (e) {
    toastError(`记录失败: ${e instanceof Error ? e.message : String(e)}`)
  }
}

/** 打开规则自带的参考链接（规则库内置 URL，属可信来源） */
async function openRef(url: string) {
  try {
    await open(url)
  } catch {
    toastError('打开链接失败')
  }
}

function clearAll() {
  logText.value = ''
  result.value = null
  showPending.value = false
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <CommandLineIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">崩溃日志分析</h3>
    </div>
    <div class="px-5 pb-5 space-y-3">
      <p class="text-xs text-gray-500">
        粘贴 Minecraft 崩溃日志，本地引擎会检索日志原文并逐条给出结论；每条结论都附带行号与原文，方便你回日志里核对。
      </p>

      <Input
        v-model="logText"
        textarea
        :rows="8"
        placeholder="在此粘贴崩溃日志全文（crash report 或 latest.log 中的报错片段）..."
      />

      <div class="flex justify-end gap-2">
        <Button type="ghost" size="small" :disabled="!logText" @click="clearAll">清空</Button>
        <Button type="primary" :loading="loading" :disabled="!canAnalyze" @click="runAnalyze">
          <template #icon><MagnifyingGlassIcon class="h-4 w-4" /></template>
          {{ loading ? '分析中...' : '开始分析' }}
        </Button>
      </div>

      <div v-if="result">
        <div v-if="analyses.length === 0" class="flex flex-col items-center justify-center py-8 text-gray-400">
          <ExclamationCircleIcon class="h-8 w-8 mb-2" />
          <span class="text-xs">未识别到已知崩溃模式</span>
        </div>

        <div v-else class="space-y-2">
          <div class="flex items-center justify-between text-sm font-medium text-gray-700">
            <span>识别出 {{ analyses.length }} 个可能原因</span>
            <span v-if="result.line_count" class="text-xs font-normal text-gray-400">
              日志共 {{ result.line_count }} 行
            </span>
          </div>

          <CrashAnalysisCard
            v-for="(item, idx) in analyses"
            :key="item.cause + '-' + idx"
            :item="item"
            @report-wrong="reportWrong"
            @open-ref="openRef"
          />

          <!-- 待确认：命中关键词但拿不到证据行，不作为正式结论 -->
          <div v-if="pending.length" class="rounded-lg border border-dashed border-gray-300 px-4 py-3">
            <button
              type="button"
              class="flex w-full items-center justify-between text-xs text-gray-500"
              @click="showPending = !showPending"
            >
              <span>另有 {{ pending.length }} 条线索证据不足，未计入结论</span>
              <span>{{ showPending ? '收起' : '展开' }}</span>
            </button>
            <div v-if="showPending" class="mt-2 space-y-1">
              <div v-for="(item, idx) in pending" :key="'pending-' + idx" class="text-xs text-gray-500">
                <span class="text-gray-700">{{ item.title }}</span>
                <span v-if="item.missing_evidence" class="text-gray-400">：{{ item.missing_evidence }}</span>
              </div>
            </div>
          </div>

          <div class="mt-3 flex items-center gap-2 rounded-lg bg-primary-50/60 px-3 py-2.5">
            <SparklesIcon class="h-4 w-4 shrink-0 text-primary-500" />
            <span class="min-w-0 flex-1 text-xs text-gray-600">
              已定位问题范围，下一步由 AI 引擎深入分析具体原因与修复方案。
            </span>
            <Button type="primary" size="small" @click="emit('ai-followup', logText)">
              <template #icon><SparklesIcon class="h-4 w-4" /></template>
              用 AI 深度分析
            </Button>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
