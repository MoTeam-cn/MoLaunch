<script setup lang="ts">
/**
 * 实验性 - 日志分析
 *
 * 两级流水线：本地规则引擎先检索定位问题范围 → 交由 AI 引擎深度分析原因与修复方案。
 * - 第一级：CrashAnalyzer（本地引擎，粘贴日志 → 识别常见崩溃模式条目）
 * - 第二级：AiLogAnalyzer（AI 深度分析，接收本地初检后的日志文本自动发起，后端注入预检范围）
 */
import { ref } from 'vue'
import { InformationCircleIcon } from '@heroicons/vue/24/outline'
import CrashAnalyzer from '@/views/tools/data/CrashAnalyzer.vue'
import AiLogAnalyzer from '@/components/experimental/AiLogAnalyzer.vue'

/** 本地引擎初检完成后传回给 AI 分析器的日志文本 */
const aiLogText = ref<string | undefined>(undefined)

function onLocalFollowup(text: string) {
  // 传新引用触发 AiLogAnalyzer 的 watch
  aiLogText.value = undefined
  requestAnimationFrame(() => {
    aiLogText.value = text
  })
}

function onAiConsumed() {
  aiLogText.value = undefined
}
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-start gap-2 rounded-lg border border-gray-200 bg-gray-50 px-4 py-3">
      <InformationCircleIcon class="mt-0.5 h-4 w-4 shrink-0 text-gray-400" />
      <p class="text-xs leading-relaxed text-gray-600">
        流程：先由<b>本地规则引擎</b>检索日志问题范围，再交由<b>AI 引擎</b>深入分析具体原因与修复方案（日志过长时只会把初检范围发给 AI）。
      </p>
    </div>

    <CrashAnalyzer @ai-followup="onLocalFollowup" />

    <AiLogAnalyzer :external-log-text="aiLogText" @consumed="onAiConsumed" />
  </div>
</template>
