<script setup lang="ts">
/**
 * 纹理 PNG 2D 预览（data URI）+ 纹理替换（导入本地图片 → rp_write base64 写回）
 * + 像素画板编辑（RpPixelEditor，canvas 原生逐像素绘制后写回）
 */
import { ref, watch, defineAsyncComponent } from 'vue'
import { ArrowUpTrayIcon, PaintBrushIcon, PhotoIcon } from '@heroicons/vue/24/outline'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const RpPixelEditor = defineAsyncComponent(() => import('./RpPixelEditor.vue'))
import { toastError, toastSuccess } from '@/utils/toast'
import { rpWrite } from '@/utils/api/tools'

const props = defineProps<{
  workDir: string
  relPath: string
  src: string
  animated: boolean
  name: string
}>()

const emit = defineEmits<{ (e: 'saved'): void }>()

const displaySrc = ref('')
const replacing = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)
/** 像素画板编辑模式 */
const editMode = ref(false)

watch(
  () => props.src,
  (v) => {
    displaySrc.value = v
    editMode.value = false
  },
  { immediate: true },
)

function openPicker() {
  fileInput.value?.click()
}

function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  if (!file.type.startsWith('image/')) {
    toastError('请选择图片文件')
    return
  }
  const reader = new FileReader()
  reader.onload = () => {
    const dataUri = typeof reader.result === 'string' ? reader.result : ''
    if (!dataUri) {
      toastError('读取图片失败')
      return
    }
    replaceTexture(dataUri)
  }
  reader.onerror = () => toastError('读取图片失败')
  reader.readAsDataURL(file)
}

async function replaceTexture(dataUri: string) {
  replacing.value = true
  try {
    const res = await rpWrite({
      work_dir: props.workDir,
      rel_path: props.relPath,
      kind: 'base64',
      content: dataUri,
    })
    if (!res.success) {
      toastError(res.message)
      return
    }
    displaySrc.value = dataUri
    toastSuccess('纹理已替换')
    emit('saved')
  } catch (e) {
    toastError(`替换失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    replacing.value = false
  }
}
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center gap-2">
      <PhotoIcon class="h-4 w-4 text-gray-500" />
      <h4 class="text-sm font-medium text-gray-700">{{ name }}</h4>
      <span v-if="animated" class="rounded bg-purple-100 px-1.5 py-0.5 text-xs text-purple-600">
        动画纹理
      </span>
      <div class="ml-auto flex items-center gap-2">
        <Button
          v-if="!editMode"
          type="outline"
          size="small"
          :loading="replacing"
          @click="openPicker"
        >
          <template #icon><ArrowUpTrayIcon class="h-4 w-4" /></template>
          替换纹理
        </Button>
        <Button v-if="!editMode" size="small" @click="editMode = true">
          <template #icon><PaintBrushIcon class="h-4 w-4" /></template>
          像素画板
        </Button>
      </div>
      <input ref="fileInput" type="file" accept="image/*" class="hidden" @change="onFileChange" />
    </div>
    <RpPixelEditor
      v-if="editMode"
      :work-dir="workDir"
      :rel-path="relPath"
      :src="displaySrc"
      :name="name"
      @saved="emit('saved')"
      @close="editMode = false"
    />
    <div v-else-if="displaySrc" class="grid place-items-center rounded border border-gray-200 bg-gray-50 p-4">
      <img :src="displaySrc" class="max-h-96 max-w-full object-contain" alt="纹理预览" />
    </div>
    <p v-else class="text-sm text-gray-400">纹理加载失败</p>
  </div>
</template>
