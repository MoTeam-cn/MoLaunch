<script setup lang="ts">
/**
 * 崩溃分析结论卡片
 *
 * 展示一条结论：分类 / 严重级别 / 置信度 / 影响 / 证据行（行号 + 原文）/ 建议 / 可疑 mod / 参考链接。
 * 证据行一律等宽展示且可折叠，方便用户拿去和日志原文逐行对照。
 */
import { ref } from 'vue'
import type { CrashAnalysisItem } from '@/utils/api/tools'

const props = defineProps<{ item: CrashAnalysisItem }>()
const emit = defineEmits<{
  'report-wrong': [item: CrashAnalysisItem]
  'open-ref': [url: string]
}>()

const expanded = ref(false)

const categoryLabel: Record<string, string> = {
  java_version: 'Java 版本',
  missing_mod: '缺失 Mod',
  memory: '内存',
  driver: '显卡驱动',
  mod_conflict: 'Mod 冲突',
  world: '存档 / 世界',
  server: '服务端',
  plugin: '插件',
  other: '其他',
}

const severityStyle: Record<string, string> = {
  error: 'bg-red-100 text-red-700',
  warning: 'bg-amber-100 text-amber-700',
  info: 'bg-blue-100 text-blue-700',
}

const confidenceLabel: Record<string, string> = { high: '高置信', medium: '中置信', low: '低置信' }
const confidenceStyle: Record<string, string> = {
  high: 'bg-green-100 text-green-700',
  medium: 'bg-amber-100 text-amber-700',
  low: 'bg-gray-100 text-gray-600',
}

const evidence = () => props.item.evidence ?? []
const visibleEvidence = () => (expanded.value ? evidence() : evidence().slice(0, 1))
</script>

<template>
  <div class="rounded-lg border border-gray-200 px-4 py-3">
    <div class="flex flex-wrap items-center gap-2">
      <span
        class="rounded-full px-2 py-0.5 text-xs font-medium"
        :class="severityStyle[item.severity] ?? 'bg-gray-100 text-gray-700'"
      >
        {{ categoryLabel[item.category] ?? item.category }}
      </span>
      <span
        v-if="item.confidence"
        class="rounded-full px-2 py-0.5 text-xs font-medium"
        :class="confidenceStyle[item.confidence] ?? 'bg-gray-100 text-gray-600'"
      >
        {{ confidenceLabel[item.confidence] ?? item.confidence }}
      </span>
      <span class="text-sm font-medium text-gray-900">{{ item.title }}</span>
    </div>

    <div v-if="item.impact" class="mt-1.5 text-xs text-gray-600">影响：{{ item.impact }}</div>

    <!-- 证据行：行号 + 日志原文，可折叠 -->
    <div v-if="evidence().length" class="mt-1.5 space-y-1">
      <div
        v-for="line in visibleEvidence()"
        :key="line.lineNo"
        class="flex gap-2 rounded-sm bg-gray-50 px-2 py-1 text-xs text-gray-600"
      >
        <span class="shrink-0 font-mono text-gray-400">L{{ line.lineNo }}</span>
        <span class="min-w-0 break-all font-mono">{{ line.rawLine }}</span>
      </div>
      <button
        v-if="evidence().length > 1"
        type="button"
        class="text-xs text-gray-400 transition-colors hover:text-gray-600"
        @click="expanded = !expanded"
      >
        {{ expanded ? '收起证据' : `展开全部 ${evidence().length} 条证据` }}
      </button>
    </div>

    <div v-if="item.missingEvidence" class="mt-1.5 text-xs text-amber-700">
      还缺证据：{{ item.missingEvidence }}
    </div>

    <div v-if="item.suggestion" class="mt-1.5 text-xs text-green-600">建议：{{ item.suggestion }}</div>

    <div v-if="item.suspects && item.suspects.length" class="mt-1.5 text-xs text-amber-700">
      疑似相关 mod：{{ item.suspects.join('、') }}
    </div>

    <div class="mt-1.5 flex flex-wrap items-center gap-3">
      <button
        v-if="item.ruleId"
        type="button"
        class="text-xs text-gray-400 transition-colors hover:text-gray-600"
        @click="emit('report-wrong', item)"
      >
        这条判断不对？
      </button>
      <button
        v-for="ref in item.refs ?? []"
        :key="ref"
        type="button"
        class="text-xs text-primary-600 underline-offset-2 hover:underline"
        @click="emit('open-ref', ref)"
      >
        参考链接
      </button>
    </div>
  </div>
</template>
