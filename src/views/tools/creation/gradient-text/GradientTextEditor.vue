<script setup lang="ts">
/**
 * 渐变文字生成器 - 文本编辑区（多行输入 + 行格式切换）
 */
import { computed, ref, defineAsyncComponent } from 'vue'
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import { plainTextFromDocument } from '@/utils/gradient-text'
import type { GradientTextDocument, TextFormat } from '@/utils/gradient-text'

const props = defineProps<{
  document: GradientTextDocument
}>()

const emit = defineEmits<{
  'update:document': [document: GradientTextDocument]
}>()

const plainText = ref(plainTextFromDocument(props.document))
const textareaRef = ref<HTMLTextAreaElement | null>(null)

function syncDocumentFromPlainText(text: string) {
  const oldLines = props.document.lines
  emit('update:document', {
    lines: text.split('\n').map((lineText, index) => {
      const oldRuns = oldLines[index] ?? []
      if (oldRuns.length && plainTextFromDocument({ lines: [oldRuns] }) === lineText) {
        return oldRuns
      }
      return [{ text: lineText, formats: oldRuns.length ? oldRuns[0].formats : [] }]
    }),
  })
}

const formatMeta: { format: TextFormat; label: string; title: string }[] = [
  { format: 'bold', label: 'B', title: '粗体' },
  { format: 'italic', label: 'I', title: '斜体' },
  { format: 'underlined', label: 'U', title: '下划线' },
  { format: 'strikethrough', label: 'S', title: '删除线' },
  { format: 'obfuscated', label: 'O', title: '混淆' },
]

function currentLineFormats(): Set<TextFormat> {
  const textarea = textareaRef.value
  if (!textarea) return new Set()
  const before = textarea.value.substring(0, textarea.selectionStart)
  const lineIndex = before.split('\n').length - 1
  const line = props.document.lines[lineIndex]
  if (!line) return new Set()
  return new Set(line[0]?.formats ?? [])
}

function toggleFormat(format: TextFormat) {
  const textarea = textareaRef.value
  if (!textarea) return
  const before = textarea.value.substring(0, textarea.selectionStart)
  const lineIndex = before.split('\n').length - 1
  const line = props.document.lines[lineIndex]
  if (!line) return
  const allActive = line.every((run) => run.formats.includes(format))
  const lines = [...props.document.lines]
  lines[lineIndex] = line.map((run) => ({
    ...run,
    formats: allActive
      ? run.formats.filter((f) => f !== format)
      : run.formats.includes(format)
        ? run.formats
        : [...run.formats, format],
  }))
  emit('update:document', { lines })
}

const activeFormats = computed(() => currentLineFormats())
</script>

<template>
  <div>
    <label class="mb-1.5 block text-xs font-medium text-gray-700">文本内容（多行）</label>
    <textarea
      ref="textareaRef"
      v-model="plainText"
      rows="4"
      class="w-full resize-y rounded border border-gray-300 px-3 py-2 text-sm leading-relaxed text-gray-800 outline-none transition focus:border-primary-500"
      placeholder="输入要生成渐变效果的文字…"
      @input="syncDocumentFromPlainText(($event.target as HTMLTextAreaElement).value)"
    />
    <div class="mt-2 flex items-center gap-1">
      <Tooltip v-for="meta in formatMeta" :key="meta.format" :text="meta.title">
        <button
          type="button"
          class="flex h-7 w-7 items-center justify-center rounded border text-xs font-semibold transition"
          :class="
            activeFormats.has(meta.format)
              ? 'border-primary-500 bg-primary-50 text-primary-600'
              : 'border-gray-300 bg-white text-gray-600 hover:border-primary-400'
          "
          @click="toggleFormat(meta.format)"
        >
          {{ meta.label }}
        </button>
      </Tooltip>
      <span class="ml-2 text-xs text-gray-400">格式作用于光标所在行</span>
    </div>
  </div>
</template>