<script setup lang="ts">
/**
 * 实验性 - 日志分析
 *
 * 两级流水线（单一日志输入）：
 * - 第一级：CrashAnalyzer 本地引擎——页面唯一的日志输入框，粘贴日志识别常见崩溃模式。
 *   识别出具体问题 → 展示本地结果，可点「用 AI 深度分析」深入；
 *   本地无法定位具体问题 → 自动转交 AI。
 * - 第二级：AiLogAnalyzer AI 弹窗——不包含输入框，收到本地初检后的日志文本自动打开
 *   弹窗并发起 AI 深度分析（后端 localAnalyze=true 只把命中关键词前后 ±15 行上下文发给 AI）。
 */
import { ref, defineAsyncComponent } from 'vue'
const AlertV2 = defineAsyncComponent(() => import('@/components/common/AlertV2.vue'))
const CrashAnalyzer = defineAsyncComponent(() => import('@/views/tools/data/CrashAnalyzer.vue'))
const AiLogAnalyzer = defineAsyncComponent(() => import('@/components/experimental/AiLogAnalyzer.vue'))

/** 本地引擎初检完成后传回给 AI 面板的日志文本 */
const aiLogText = ref<string | undefined>(undefined)

function onLocalFollowup(text: string) {
  // 先置空再在下一帧赋值：即使两次日志内容相同也能触发 AiLogAnalyzer 的 watch
  aiLogText.value = undefined
  requestAnimationFrame(() => {
    aiLogText.value = text
  })
}
</script>

<template>
  <div class="space-y-4">
    <AlertV2
      type="info"
      message="在下方输入框中粘贴日志，本地引擎会先检索问题范围；本地无法定位具体问题时，将自动弹出 AI 深度分析弹窗（只发送问题关键词附近上下文，日志全文不会上传）。"
    />

    <CrashAnalyzer @ai-followup="onLocalFollowup" />

    <AiLogAnalyzer :external-log-text="aiLogText" />
  </div>
</template>
