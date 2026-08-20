<script setup lang="ts">
/**
 * 模组翻译 - 分析结果区（左侧内容区）
 */
import { ref, defineAsyncComponent } from 'vue'
const Alert = defineAsyncComponent(() => import('@/components/common/Alert.vue'))
const Collapse = defineAsyncComponent(() => import('@/components/common/Collapse.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import type { ModTranslationAnalyzeResult } from '@/utils/api/experimental-mod-translation'

const props = defineProps<{
  analyzeResult: ModTranslationAnalyzeResult
}>()

const detailOpen = ref(false)

const loaderLabels: Record<string, string> = {
  fabric: 'Fabric',
  neoforge: 'NeoForge',
  forge: 'Forge',
  unknown: '未知',
}
const kindLabels: Record<string, string> = {
  json: 'JSON 语言文件',
  'key-value': '.lang/.properties',
  'structured-json': '结构化 JSON',
  'free-text': '自由文本',
}
const dispositionLabels: Record<string, string> = {
  standard_language: '标准语言',
  structured_source: '结构化源',
  generated_target: '生成目标',
  class_review: 'class 复核',
  unknown: '未知',
  protected: '保护',
}
</script>

<template>
  <div class="flex-1 min-w-0 flex flex-col bg-white rounded-lg border border-gray-300 overflow-hidden">
    <div class="px-5 pt-4 pb-3 border-b border-gray-100 shrink-0">
      <h3 class="text-sm font-semibold text-gray-900">2. 分析结果</h3>
    </div>
    <div class="flex-1 min-h-0 overflow-y-auto px-5 py-3">
      <div class="space-y-2 text-sm">
        <div class="flex items-center gap-2">
          <span class="text-gray-500 w-16 shrink-0">加载器</span>
          <span class="text-gray-800">{{ loaderLabels[props.analyzeResult.loader] ?? props.analyzeResult.loader }}</span>
        </div>
        <div v-if="props.analyzeResult.modIds.length" class="flex items-center gap-2">
          <span class="text-gray-500 w-16 shrink-0">Mod ID</span>
          <span class="text-gray-800">{{ props.analyzeResult.modIds.join(', ') }}</span>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-gray-500 w-16 shrink-0">条目数</span>
          <span class="text-gray-800">{{ props.analyzeResult.totalEntries }}</span>
        </div>
        <div v-if="props.analyzeResult.version" class="flex items-center gap-2">
          <span class="text-gray-500 w-16 shrink-0">版本</span>
          <span class="text-gray-800">{{ props.analyzeResult.version }}</span>
        </div>
        <div v-if="props.analyzeResult.classCandidates.length" class="flex items-center gap-2">
          <span class="text-gray-500 w-16 shrink-0">class 文本</span>
          <span class="text-gray-800">{{ props.analyzeResult.classCandidates.length }} 个候选</span>
        </div>
        <div v-if="props.analyzeResult.existingChinese.length" class="mt-3">
          <Alert variant="soft" type="warning">
            <p>该模组已包含 {{ props.analyzeResult.existingChinese.length }} 个中文语言文件，翻译将覆盖以下文件：</p>
            <ul class="mt-1 space-y-0.5">
              <li
                v-for="item in props.analyzeResult.existingChinese"
                :key="item.path"
                class="text-yellow-700"
              >
                <Tooltip :text="`${item.locale} · ${item.path}（${item.entries} 条）`" overflow-only>
                  <span class="block truncate">{{ item.locale }} · {{ item.path }}（{{ item.entries }} 条）</span>
                </Tooltip>
              </li>
            </ul>
          </Alert>
        </div>
        <div v-if="props.analyzeResult.signed" class="flex items-center gap-2">
          <span class="text-yellow-600">JAR 含签名文件，重打包后签名将失效</span>
        </div>
        <div v-for="warn in props.analyzeResult.warnings" :key="warn" class="flex items-center gap-2">
          <span class="text-yellow-600">{{ warn }}</span>
        </div>
      </div>

      <div class="mt-4 border border-gray-200 rounded overflow-hidden">
        <div class="bg-gray-50 px-3 py-2 text-xs text-gray-500 flex items-center gap-3 border-b border-gray-200">
          <span class="w-32 shrink-0">类型</span>
          <span class="flex-1 truncate">文件</span>
          <span class="w-16 text-right shrink-0">待译条目</span>
        </div>
        <div class="max-h-48 overflow-y-auto divide-y divide-gray-100">
          <div
            v-for="source in props.analyzeResult.sources"
            :key="source.sourcePath"
            class="px-3 py-2 text-xs flex items-center gap-3"
          >
            <span class="w-32 shrink-0 text-gray-500">{{ kindLabels[source.kind] ?? source.kind }}</span>
            <Tooltip :text="source.targetPath" overflow-only class="flex-1 min-w-0">
              <span class="block truncate text-gray-700">{{ source.targetPath }}</span>
            </Tooltip>
            <span class="w-16 text-right shrink-0 text-gray-700">{{ source.entries }}</span>
          </div>
        </div>
      </div>

      <div class="mt-4">
        <button
          class="flex items-center gap-1 text-xs text-gray-500 hover:text-gray-700"
          @click="detailOpen = !detailOpen"
        >
          <svg
            class="w-3 h-3 transition-transform duration-200"
            :class="detailOpen ? 'rotate-90' : ''"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path fill-rule="evenodd" d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z" clip-rule="evenodd" />
          </svg>
          成本与覆盖分析
        </button>
        <Collapse :open="detailOpen">
          <div class="mt-2 space-y-3 rounded-lg border border-dashed border-gray-300 p-3">
            <div class="grid grid-cols-2 gap-x-4 gap-y-1 text-xs text-gray-600">
              <div>预估 token：{{ props.analyzeResult.quote.estimatedTokens }}</div>
              <div>调用次数：{{ props.analyzeResult.quote.estimatedCalls }}</div>
              <div>语言批次：{{ props.analyzeResult.quote.languageBatches }}</div>
              <div>class 批次：{{ props.analyzeResult.quote.classBatches }}</div>
              <div>预估点数：{{ props.analyzeResult.quote.points }}</div>
            </div>
            <div class="border border-gray-200 rounded overflow-hidden">
              <div class="max-h-40 overflow-y-auto divide-y divide-gray-100">
                <div
                  v-for="item in props.analyzeResult.coverage"
                  :key="item.path"
                  class="px-3 py-1.5 text-xs flex items-center gap-2"
                >
                  <Tooltip :text="item.path" overflow-only class="flex-1 min-w-0">
                    <span class="block truncate text-gray-700">{{ item.path }}</span>
                  </Tooltip>
                  <Tag size="small" color="gray">{{ dispositionLabels[item.disposition] ?? item.disposition }}</Tag>
                </div>
              </div>
            </div>
          </div>
        </Collapse>
      </div>
    </div>
  </div>
</template>