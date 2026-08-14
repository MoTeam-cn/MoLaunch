<script setup lang="ts">
/**
 * 渐变文字生成器（创作工具）
 *
 * 输入多行文本与颜色停靠点，预览 Minecraft 阴影渐变效果，
 * 生成 19 种输出格式（Vanilla / MiniMessage / JSON / BBCode 等）并支持复制/下载。
 *
 * 布局：文本编辑（左） + 输出配置（右），底部预设管理。
 */
import { computed, reactive, ref, watch, defineAsyncComponent } from 'vue'
import {
  ArrowDownTrayIcon,
  BookmarkIcon,
  PlusIcon,
  ClipboardDocumentIcon,
  ArrowUpIcon,
  ArrowDownIcon,
  TrashIcon,
  XMarkIcon,
  PencilSquareIcon,
} from '@heroicons/vue/24/outline'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Checkbox = defineAsyncComponent(() => import('@/components/common/Checkbox.vue'))
const ColorPicker = defineAsyncComponent(() => import('@/components/common/ColorPicker.vue'))
const SegmentedButtons = defineAsyncComponent(() => import('@/components/common/SegmentedButtons.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import {
  buildGradientCharacters,
  generateGradientOutput,
  getMinecraftTextShadow,
  gradientFormatAdapters,
  plainTextFromDocument,
  normalizeHexColor,
  parseGradientPresets,
  serializeGradientPresets,
  loadGradientTextState,
  saveGradientTextState,
} from '@/utils/gradient-text'
import type { GradientPreset, GradientTextState } from '@/utils/gradient-text'
import type { TextFormat } from '@/utils/gradient-text'
import { toastSuccess, toastError } from '@/utils/toast'

const state = reactive<GradientTextState>(loadGradientTextState())

const plainText = ref(plainTextFromDocument(state.document))
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const presetName = ref('')
const importText = ref('')

watch(
  state,
  (value) => {
    saveGradientTextState({ ...value })
  },
  { deep: true },
)

function syncDocumentFromPlainText(text: string) {
  const oldLines = state.document.lines
  state.document = {
    lines: text.split('\n').map((lineText, index) => {
      const oldRuns = oldLines[index] ?? []
      if (oldRuns.length && plainTextFromDocument({ lines: [oldRuns] }) === lineText) {
        return oldRuns
      }
      return [{ text: lineText, formats: oldRuns.length ? oldRuns[0].formats : [] }]
    }),
  }
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
  const line = state.document.lines[lineIndex]
  if (!line) return new Set()
  return new Set(line[0]?.formats ?? [])
}

function toggleFormat(format: TextFormat) {
  const textarea = textareaRef.value
  if (!textarea) return
  const before = textarea.value.substring(0, textarea.selectionStart)
  const lineIndex = before.split('\n').length - 1
  const line = state.document.lines[lineIndex]
  if (!line) return
  const allActive = line.every((run) => run.formats.includes(format))
  state.document.lines[lineIndex] = line.map((run) => ({
    ...run,
    formats: allActive
      ? run.formats.filter((f) => f !== format)
      : run.formats.includes(format)
        ? run.formats
        : [...run.formats, format],
  }))
}

const activeFormats = computed(() => currentLineFormats())

function addColor() {
  state.colors.push('#165DFF')
}

function removeColor(index: number) {
  if (state.colors.length <= 1) return
  state.colors.splice(index, 1)
}

function moveColor(index: number, direction: -1 | 1) {
  const target = index + direction
  if (target < 0 || target >= state.colors.length) return
  ;[state.colors[index], state.colors[target]] = [state.colors[target], state.colors[index]]
}

const previewCharacters = computed(() =>
  buildGradientCharacters(state.document, state.colors),
)

const output = computed(() =>
  generateGradientOutput(state.document, state.colors, state.adapterId, {
    vanillaCharacter: state.vanillaCharacter,
    simplifyGradients: state.simplifyGradients,
  }),
)

const currentAdapter = computed(() =>
  gradientFormatAdapters.find((adapter) => adapter.id === state.adapterId),
)

const adapterOptions = computed(() =>
  gradientFormatAdapters.map((adapter) => ({ label: adapter.label, value: adapter.id })),
)

async function copyOutput() {
  try {
    await navigator.clipboard.writeText(output.value)
    toastSuccess('已复制到剪贴板')
  } catch {
    toastError('复制失败，请手动选择复制')
  }
}

function downloadOutput() {
  const adapter = currentAdapter.value
  if (!adapter) return
  const blob = new Blob([output.value], { type: adapter.mimeType })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `gradient-text.${adapter.extension}`
  link.click()
  URL.revokeObjectURL(url)
}

function savePreset() {
  const name = presetName.value.trim()
  if (!name) {
    toastError('请输入预设名称')
    return
  }
  state.presets.unshift({
    id: `${Date.now()}`,
    name: name.slice(0, 80),
    colors: [...state.colors],
    createdAt: new Date().toISOString(),
  })
  presetName.value = ''
  toastSuccess('预设已保存')
}

function loadPreset(preset: GradientPreset) {
  state.colors = [...preset.colors]
  toastSuccess(`已加载「${preset.name}」`)
}

function removePreset(id: string) {
  state.presets = state.presets.filter((preset) => preset.id !== id)
}

async function exportPresets() {
  try {
    await navigator.clipboard.writeText(serializeGradientPresets(state.presets))
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
    state.presets = [...parsed, ...state.presets]
    importText.value = ''
    toastSuccess(`已导入 ${parsed.length} 个预设`)
  } catch {
    toastError('JSON 格式无效')
  }
}

function randomColor() {
  state.colors.push(
    normalizeHexColor(
      `#${Math.floor(Math.random() * 0xffffff)
        .toString(16)
        .padStart(6, '0')}`,
    ) ?? '#165DFF',
  )
}
</script>

<template>
  <section
    class="rounded-lg border border-gray-300 bg-white overflow-hidden"
  >
    <!-- 标题 -->
    <div class="flex items-center justify-between px-5 pt-5 pb-3">
      <div class="flex items-center gap-2">
        <PencilSquareIcon class="h-5 w-5 text-gray-700" />
        <h3 class="text-sm font-semibold text-gray-900">渐变文字生成器</h3>
      </div>
    </div>

    <div class="grid gap-4 px-5 pb-5 lg:grid-cols-2">
      <!-- 左栏：文本编辑 + 颜色 -->
      <div class="space-y-4">
        <!-- 文本输入 -->
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

        <!-- 颜色停靠点 -->
        <div>
          <div class="mb-1.5 flex items-center justify-between">
            <label class="text-xs font-medium text-gray-700">颜色停靠点</label>
            <div class="flex items-center gap-1">
              <Button type="text" size="small" @click="randomColor">
                <template #icon><PlusIcon class="h-3.5 w-3.5" /></template>
                随机
              </Button>
              <Button type="outline" size="small" @click="addColor">
                <template #icon><PlusIcon class="h-3.5 w-3.5" /></template>
                添加
              </Button>
            </div>
          </div>
          <div class="space-y-2">
            <div
              v-for="(_, index) in state.colors"
              :key="index"
              class="flex items-center gap-2 rounded border border-gray-200 bg-gray-50 px-3 py-2"
            >
              <span class="w-6 flex-none text-xs text-gray-400">{{ index + 1 }}</span>
              <ColorPicker v-model="state.colors[index]" />
              <span class="flex-1" />
              <Button type="text" size="small" :disabled="index === 0" @click="moveColor(index, -1)">
                <template #icon><ArrowUpIcon class="h-3.5 w-3.5" /></template>
              </Button>
              <Button
                type="text"
                size="small"
                :disabled="index === state.colors.length - 1"
                @click="moveColor(index, 1)"
              >
                <template #icon><ArrowDownIcon class="h-3.5 w-3.5" /></template>
              </Button>
              <Button type="text" size="small" :disabled="state.colors.length <= 1" @click="removeColor(index)">
                <template #icon><TrashIcon class="h-3.5 w-3.5" /></template>
              </Button>
            </div>
          </div>
        </div>
      </div>

      <!-- 右栏：预览 + 输出 -->
      <div class="space-y-4">
        <!-- 预览 -->
        <div>
          <label class="mb-1.5 block text-xs font-medium text-gray-700">MC 游戏内预览</label>
          <div class="min-h-[96px] rounded border border-gray-200 bg-gray-900 px-4 py-3">
            <p class="break-words text-xl leading-relaxed text-white">
              <template v-for="(character, index) in previewCharacters" :key="index">
                <span
                  v-if="!character.newline"
                  :style="{
                    color: character.color ?? '#FFFFFF',
                    textShadow: `2px 2px 0 ${getMinecraftTextShadow(character.color)}`,
                  }"
                >{{ character.character === ' ' ? '\u00A0' : character.character }}</span>
                <br v-else />
              </template>
            </p>
          </div>
        </div>

        <!-- 输出配置 -->
        <div>
          <label class="mb-1.5 block text-xs font-medium text-gray-700">输出格式</label>
          <div class="flex items-start gap-3">
            <div class="flex-1">
              <Select v-model="state.adapterId" :options="adapterOptions" />
              <p class="mt-1 text-xs text-gray-400">示例：{{ currentAdapter?.sample }}</p>
            </div>
            <div class="space-y-2">
              <SegmentedButtons
                v-if="currentAdapter?.supportsVanillaCharacter"
                v-model="state.vanillaCharacter"
                :options="[
                  { label: '&', value: '&' },
                  { label: '§', value: '§' },
                ]"
              />
              <Checkbox
                v-if="currentAdapter?.supportsSimplify"
                v-model="state.simplifyGradients"
              >
                <span class="text-xs text-gray-600">简化渐变</span>
              </Checkbox>
            </div>
          </div>
        </div>

        <!-- 输出结果 -->
        <div>
          <label class="mb-1.5 block text-xs font-medium text-gray-700">生成结果</label>
          <pre class="max-h-40 overflow-y-auto whitespace-pre-wrap break-all rounded border border-gray-200 bg-gray-50 px-3 py-2 text-xs text-gray-800">{{ output }}</pre>
          <div class="mt-2 flex items-center gap-2">
            <Button type="primary" size="small" @click="copyOutput">
              <template #icon><ClipboardDocumentIcon class="h-3.5 w-3.5" /></template>
              复制
            </Button>
            <Button type="outline" size="small" @click="downloadOutput">
              <template #icon><ArrowDownTrayIcon class="h-3.5 w-3.5" /></template>
              下载 {{ currentAdapter?.extension ?? 'txt' }}
            </Button>
          </div>
        </div>
      </div>
    </div>

    <!-- 预设管理 -->
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
      <div v-if="state.presets.length" class="mt-3 flex flex-wrap gap-2">
        <div
          v-for="preset in state.presets"
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
  </section>
</template>
