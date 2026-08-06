<script setup lang="ts">
/**
 * 崩溃日志分析
 *
 * 粘贴崩溃日志文本 → 调用 crashAnalyze → 展示识别出的崩溃原因条目。
 * 后端按 6 类模式（Java 版本 / 缺失 Mod / 内存 / 显卡驱动 / Mod 冲突 / 其他）
 * 做大小写不敏感匹配，返回带严重级别与修复建议的条目列表。
 */
import { ref, computed } from 'vue'
import {
  CommandLineIcon,
  MagnifyingGlassIcon,
  ExclamationCircleIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import { SparklesIcon } from '@heroicons/vue/24/outline'
import { toastSuccess, toastError, toastInfo } from '@/utils/toast'
import { crashAnalyze } from '@/utils/api/tools'
import type { CrashAnalyzeResult } from '@/utils/api/tools'

const emit = defineEmits<{ 'ai-followup': [logText: string] }>()

const logText = ref('')
const loading = ref(false)
const result = ref<CrashAnalyzeResult | null>(null)

const canAnalyze = computed(() => logText.value.trim().length > 0)

const categoryLabel: Record<string, string> = {
  java_version: 'Java 版本',
  missing_mod: '缺失 Mod',
  memory: '内存不足',
  driver: '显卡驱动',
  mod_conflict: 'Mod 冲突',
  other: '其他',
}

const severityStyle: Record<string, string> = {
  error: 'bg-red-100 text-red-700',
  warning: 'bg-amber-100 text-amber-700',
  info: 'bg-blue-100 text-blue-700',
}

async function runAnalyze() {
  if (!canAnalyze.value) {
    toastError('请先粘贴崩溃日志内容')
    return
  }
  loading.value = true
  result.value = null
  try {
    result.value = await crashAnalyze(logText.value)
    const items = result.value.analyses
    // 本地引擎没有识别到具体问题（无条目，或仅有 other/info 通用条目）：
    // 只提示用户可点「用 AI 深度分析」按钮深入——不自动弹窗，AI 分析完全由用户主动触发。
    const hasSpecific = items.some((it) => it.category !== 'other' && it.severity !== 'info')
    if (items.length === 0 || !hasSpecific) {
      toastInfo('本地引擎未识别出具体问题，可点击「用 AI 深度分析」进一步诊断')
    } else {
      toastSuccess('分析完成，识别出 ' + items.length + ' 个可能原因')
    }
  } catch (e) {
    toastError(`分析失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    loading.value = false
  }
}

function clearAll() {
  logText.value = ''
  result.value = null
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
        粘贴 Minecraft 崩溃日志，自动识别常见崩溃原因（Java 版本不匹配、缺失 Mod、内存不足、显卡驱动、Mod 冲突等）并给出修复建议。
      </p>

      <!-- 日志输入 -->
      <Input
        v-model="logText"
        textarea
        :rows="8"
        placeholder="在此粘贴崩溃日志全文（crash report 或 latest.log 中的报错片段）..."
      />

      <!-- 操作按钮 -->
      <div class="flex justify-end gap-2">
        <Button type="ghost" size="small" :disabled="!logText" @click="clearAll">清空</Button>
        <Button type="primary" :loading="loading" :disabled="!canAnalyze" @click="runAnalyze">
          <template #icon><MagnifyingGlassIcon class="h-4 w-4" /></template>
          {{ loading ? '分析中...' : '开始分析' }}
        </Button>
      </div>

      <!-- 结果区 -->
      <div v-if="result">
        <div v-if="result.analyses.length === 0" class="flex flex-col items-center justify-center py-8 text-gray-400">
          <ExclamationCircleIcon class="h-8 w-8 mb-2" />
          <span class="text-xs">未识别到已知崩溃模式</span>
        </div>

        <div v-else class="space-y-2">
          <div class="text-sm font-medium text-gray-700">识别出 {{ result.analyses.length }} 个可能原因</div>
          <div
            v-for="(item, idx) in result.analyses"
            :key="idx"
            class="rounded-lg border border-gray-200 px-4 py-3"
          >
            <div class="flex items-center gap-2">
              <span
                class="rounded-full px-2 py-0.5 text-xs font-medium"
                :class="severityStyle[item.severity] ?? 'bg-gray-100 text-gray-700'"
              >
                {{ categoryLabel[item.category] ?? item.category }}
              </span>
              <span class="text-sm font-medium text-gray-900">{{ item.title }}</span>
            </div>
            <div v-if="item.detail" class="mt-1.5 rounded bg-gray-50 px-2 py-1 text-xs text-gray-600 break-all">
              {{ item.detail }}
            </div>
            <div v-if="item.suggestion" class="mt-1.5 text-xs text-green-600">
              建议：{{ item.suggestion }}
            </div>
          </div>

          <!-- 流水线第二步入口：把本地检索到的范围交给 AI 深度分析 -->
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
