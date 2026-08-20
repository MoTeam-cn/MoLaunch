<script setup lang="ts">
/**
 * 渐变文字生成器 - 颜色预设管理（保存/加载/导入导出）
 */
import { ref, defineAsyncComponent } from 'vue'
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
import {
  parseGradientPresets,
  serializeGradientPresets,
} from '@/utils/gradient-text'
import type { GradientPreset } from '@/utils/gradient-text'
import { toastSuccess, toastError } from '@/utils/toast'
import { BookmarkIcon, XMarkIcon } from '@heroicons/vue/24/outline'

const props = defineProps<{
  presets: GradientPreset[]
  colors: string[]
}>()

const emit = defineEmits<{
  'update:presets': [presets: GradientPreset[]]
  'update:colors': [colors: string[]]
}>()

const presetName = ref('')
const importText = ref('')

function savePreset() {
  const name = presetName.value.trim()
  if (!name) {
    toastError('请输入预设名称')
    return
  }
  emit('update:presets', [
    {
      id: `${Date.now()}`,
      name: name.slice(0, 80),
      colors: [...props.colors],
      createdAt: new Date().toISOString(),
    },
    ...props.presets,
  ])
  presetName.value = ''
  toastSuccess('预设已保存')
}

function loadPreset(preset: GradientPreset) {
  emit('update:colors', [...preset.colors])
  toastSuccess(`已加载「${preset.name}」`)
}

function removePreset(id: string) {
  emit('update:presets', props.presets.filter((preset) => preset.id !== id))
}

async function exportPresets() {
  try {
    await navigator.clipboard.writeText(serializeGradientPresets(props.presets))
    toastSuccess('预设 JSON 已复制到剪贴板')
  } catch {
    toastError('复制失败')
  }
}

function importPresets() {
  try {
    const parsed = parseGradientPresets(importText.value ? JSON.parse(importText.value) : [])
    if (!parsed.length) {
      toastError('未解析到有效预设')
      return
    }
    emit('update:presets', [...parsed, ...props.presets])
    importText.value = ''
    toastSuccess(`已导入 ${parsed.length} 个预设`)
  } catch {
    toastError('JSON 格式无效')
  }
}
</script>

<template>
  <div class="border-t border-gray-200 bg-gray-50 px-5 py-4">
    <div class="flex items-center gap-2">
      <BookmarkIcon class="h-5 w-5 flex-none text-gray-700" />
      <h4 class="text-sm font-semibold text-gray-900">颜色预设</h4>
    </div>
    <div class="mt-3 flex flex-wrap items-center gap-2">
      <Input
        v-model="presetName"
        class="w-40"
        placeholder="预设名称"
        size="small"
        @keydown.enter="savePreset"
      />
      <Button type="outline" size="small" @click="savePreset">保存</Button>
      <span class="mx-1 h-4 w-px bg-gray-300" />
      <Button type="text" size="small" @click="exportPresets">导出 JSON</Button>
      <Input v-model="importText" class="w-56" placeholder="粘贴预设 JSON" size="small" />
      <Button type="text" size="small" @click="importPresets">导入</Button>
    </div>
    <div v-if="presets.length" class="mt-3 flex flex-wrap gap-2">
      <div
        v-for="preset in presets"
        :key="preset.id"
        class="flex items-center gap-2 rounded-full border border-gray-300 bg-white py-1 pl-3 pr-1"
      >
        <button
          type="button"
          class="text-xs text-gray-700 hover:text-primary-600"
          @click="loadPreset(preset)"
        >
          {{ preset.name }}
        </button>
        <button
          type="button"
          class="rounded-full p-0.5 text-gray-400 hover:bg-gray-100 hover:text-gray-600"
          @click="removePreset(preset.id)"
        >
          <XMarkIcon class="h-3.5 w-3.5" />
        </button>
      </div>
    </div>
  </div>
</template>