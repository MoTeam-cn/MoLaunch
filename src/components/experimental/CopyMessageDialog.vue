<script setup lang="ts">
/**
 * 复制消息弹窗
 *
 * 点击消息操作栏的复制按钮弹出：预览消息内容（Input 只读文本域，展示去除
 * Markdown 标记后的纯文本），选择复制为「渲染后文本」或「Markdown 原文」。
 * 点击遮罩外部或按 ESC 关闭；组件风格与项目统一（Button / AlertV2 / Input）。
 */
import { ref, watch } from 'vue'
import { XMarkIcon } from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import AlertV2 from '@/components/common/AlertV2.vue'
import Input from '@/components/common/Input.vue'
import { markdownToPlainText } from '@/utils/markdown'
import { copyToClipboard } from '@/utils/clipboard'
import { toastError, toastSuccess } from '@/utils/toast'

const props = defineProps<{
  /** 消息 Markdown 原文 */
  content: string
  /** 是否显示（v-model） */
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [v: boolean]
}>()

/** 预览文本（默认渲染后纯文本，打开时刷新） */
const preview = ref('')

watch(
  () => props.modelValue,
  (open) => {
    if (open) {
      preview.value = markdownToPlainText(props.content)
      window.addEventListener('keydown', onKey)
    } else {
      window.removeEventListener('keydown', onKey)
    }
  },
)

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') close()
}

function close() {
  emit('update:modelValue', false)
}

async function copyPlain() {
  close()
  const ok = await copyToClipboard(preview.value)
  if (ok) toastSuccess('已复制渲染后文本')
  else toastError('复制失败')
}

async function copyMarkdown() {
  close()
  const ok = await copyToClipboard(props.content)
  if (ok) toastSuccess('已复制 Markdown 原文')
  else toastError('复制失败')
}
</script>

<template>
  <Teleport to="body">
    <Transition name="dialog">
      <div
        v-if="modelValue"
        class="fixed inset-0 z-[100] flex items-center justify-center bg-black/25 p-4"
        @click.self="close"
      >
        <div class="dialog-card w-[26rem] max-w-full rounded-xl bg-white p-4 shadow-2xl">
          <!-- 标题 -->
          <div class="mb-3 flex items-center justify-between">
            <span class="text-sm font-semibold text-gray-800">复制消息</span>
            <button
              type="button"
              class="rounded p-1 text-gray-400 transition-colors hover:bg-gray-100 hover:text-gray-600"
              @click="close"
            >
              <XMarkIcon class="h-4 w-4" />
            </button>
          </div>

          <!-- 提示 -->
          <AlertV2 type="info" message="下方为去除 Markdown 标记后的纯文本预览，可按需选择复制格式" />

          <!-- 预览 -->
          <div class="mt-3">
            <Input v-model="preview" textarea :rows="6" readonly />
          </div>

          <!-- 操作 -->
          <div class="mt-4 flex justify-end gap-2">
            <Button type="outline" size="small" @click="copyMarkdown">复制 Markdown</Button>
            <Button type="primary" size="small" @click="copyPlain">复制渲染后文本</Button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
/* 遮罩淡入淡出 */
.dialog-enter-active,
.dialog-leave-active {
  transition: opacity 0.18s ease;
}

.dialog-enter-from,
.dialog-leave-to {
  opacity: 0;
}

/* 卡片轻微上移 + 缩放 */
.dialog-card {
  transition: transform 0.18s ease;
}

.dialog-enter-from .dialog-card,
.dialog-leave-to .dialog-card {
  transform: translateY(6px) scale(0.98);
}
</style>
