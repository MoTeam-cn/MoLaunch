<script setup lang="ts">
/**
 * 资源包编辑器 - 内容分发面板（mcmeta 表单 / 纹理预览 / 语言表格 / 声音 / 模型 / 文本编辑）
 */
import { computed, ref, watch, defineAsyncComponent } from 'vue'
const RpMcmetaForm = defineAsyncComponent(() => import('./RpMcmetaForm.vue'))
const RpTexturePreview = defineAsyncComponent(() => import('./RpTexturePreview.vue'))
const RpLangTable = defineAsyncComponent(() => import('./RpLangTable.vue'))
const RpSoundPreview = defineAsyncComponent(() => import('./RpSoundPreview.vue'))
const RpTextEditor = defineAsyncComponent(() => import('./RpTextEditor.vue'))
const RpModelPreview = defineAsyncComponent(() => import('./RpModelPreview.vue'))
import { toastWarning } from '@/utils/toast'
import { CubeIcon } from '@heroicons/vue/24/outline'
import type { RpReadResult, RpTreeNode } from '@/utils/api/tools'

const props = defineProps<{
  selectedNode: RpTreeNode | null
  fileContent: RpReadResult | null
  reading: boolean
  workDir: string
  mcVersion: string | null
}>()

const emit = defineEmits<{
  'mcmeta-saved': [meta: { packFormat: number; description: string | null }]
}>()

/** 模型文件视图模式：false = 3D 预览，true = JSON 文本编辑 */
const modelEditMode = ref(false)
/** 3D 预览失败自动回退标记（切换文件时重置，避免手动切回 3D 后反复自动跳转） */
const modelFallbacked = ref(false)

watch(() => props.selectedNode, () => {
  modelEditMode.value = false
  modelFallbacked.value = false
})

const textContent = computed(() =>
  props.fileContent?.kind === 'text' ? props.fileContent.content : '',
)
const mediaContent = computed(() =>
  props.fileContent?.kind === 'data_uri' ? props.fileContent.content : '',
)
const canEditText = computed(() =>
  ['json', 'model', 'text'].includes(props.selectedNode?.file_type ?? ''),
)
/** 模型 / blockstate JSON → 3D 预览 */
const isModelFile = computed(() => {
  const t = props.selectedNode?.file_type
  if (t === 'model') return true
  return t === 'json' && (props.selectedNode?.rel_path ?? '').includes('/blockstates/')
})

/** 3D 预览加载失败时自动切回 JSON 编辑（仅首次，用户手动切回 3D 后不再自动跳转） */
function onModelPreviewFailed(message: string) {
  if (modelEditMode.value || modelFallbacked.value) return
  modelFallbacked.value = true
  modelEditMode.value = true
  toastWarning(`3D 预览不可用：${message}，已切换为 JSON 编辑`)
}
</script>

<template>
  <div class="max-h-[400px] overflow-y-auto p-4">
    <RpMcmetaForm
      v-if="selectedNode?.file_type === 'mcmeta'"
      :work-dir="workDir"
      :rel-path="selectedNode.rel_path"
      :content="textContent"
      :mc-version="mcVersion"
      @saved="(meta) => emit('mcmeta-saved', meta)"
    />
    <RpTexturePreview
      v-else-if="selectedNode?.file_type === 'png'"
      :work-dir="workDir"
      :rel-path="selectedNode.rel_path"
      :src="mediaContent"
      :animated="selectedNode.animated"
      :name="selectedNode.name"
    />
    <RpLangTable
      v-else-if="selectedNode?.file_type === 'lang'"
      :work-dir="workDir"
      :rel-path="selectedNode.rel_path"
      :content="textContent"
    />
    <RpSoundPreview
      v-else-if="selectedNode?.file_type === 'ogg'"
      :src="mediaContent"
    />
    <!-- 模型 / blockstate：3D 预览 ⇄ JSON 文本编辑 -->
    <div v-else-if="selectedNode && isModelFile">
      <RpModelPreview
        v-if="!modelEditMode"
        :work-dir="workDir"
        :rel-path="selectedNode.rel_path"
        :name="selectedNode.name"
        @failed="onModelPreviewFailed"
      />
      <div v-else class="space-y-2">
        <p class="text-xs text-gray-400">JSON 文本编辑（切换后内容即时生效）</p>
        <RpTextEditor
          :work-dir="workDir"
          :rel-path="selectedNode.rel_path"
          :name="selectedNode.name"
          :file-type="selectedNode.file_type"
          :content="textContent"
        >
          <template #actions>
            <button
              class="flex items-center gap-1 text-xs text-blue-600 hover:text-blue-700"
              @click="modelEditMode = false"
            >
              返回 3D 预览
            </button>
          </template>
        </RpTextEditor>
      </div>
      <button
        v-if="!modelEditMode"
        class="mt-2 flex items-center gap-1 text-xs text-blue-600 hover:text-blue-700"
        @click="modelEditMode = true"
      >
        编辑 JSON
      </button>
    </div>
    <div v-else-if="selectedNode && canEditText">
      <p v-if="reading && !fileContent" class="py-8 text-center text-sm text-gray-400">读取中…</p>
      <RpTextEditor
        v-else
        :work-dir="workDir"
        :rel-path="selectedNode.rel_path"
        :name="selectedNode.name"
        :file-type="selectedNode.file_type"
        :content="textContent"
      />
    </div>
    <div v-else-if="selectedNode" class="flex flex-col items-center justify-center gap-1 py-16 text-gray-400">
      <p class="text-sm">暂不支持预览该类型文件</p>
      <p class="text-xs">{{ selectedNode.file_type }}</p>
    </div>
    <div v-else-if="reading" class="py-16 text-center text-sm text-gray-400">读取中…</div>
    <div v-else class="flex flex-col items-center justify-center gap-2 py-16 text-gray-400">
      <CubeIcon class="h-9 w-9 text-gray-300" />
      <p class="text-sm">在左侧选择文件以预览</p>
    </div>
  </div>
</template>