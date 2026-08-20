<script setup lang="ts">
/**
 * 渐变文字生成器 - 预览 + 输出配置 + 生成结果
 */
import { computed, defineAsyncComponent } from 'vue'
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Checkbox = defineAsyncComponent(() => import('@/components/common/Checkbox.vue'))
const SegmentedButtons = defineAsyncComponent(() => import('@/components/common/SegmentedButtons.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
import {
  buildGradientCharacters,
  generateGradientOutput,
  getMinecraftTextShadow,
  gradientFormatAdapters,
} from '@/utils/gradient-text'
import type { GradientFormatId, GradientTextDocument } from '@/utils/gradient-text'
import { toastSuccess, toastError } from '@/utils/toast'
import { ArrowDownTrayIcon, ClipboardDocumentIcon } from '@heroicons/vue/24/outline'

const props = defineProps<{
  document: GradientTextDocument
  colors: string[]
  adapterId: GradientFormatId
  vanillaCharacter: '&' | '§'
  simplifyGradients: boolean
}>()

const emit = defineEmits<{
  'update:adapter-id': [id: GradientFormatId]
  'update:vanilla-character': [char: '&' | '§']
  'update:simplify-gradients': [value: boolean]
}>()

const previewCharacters = computed(() =>
  buildGradientCharacters(props.document, props.colors),
)

const output = computed(() =>
  generateGradientOutput(props.document, props.colors, props.adapterId, {
    vanillaCharacter: props.vanillaCharacter,
    simplifyGradients: props.simplifyGradients,
  }),
)

const currentAdapter = computed(() =>
  gradientFormatAdapters.find((adapter) => adapter.id === props.adapterId),
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
</script>

<template>
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
          <Select
            :model-value="adapterId"
            :options="adapterOptions"
            @update:model-value="emit('update:adapter-id', String($event) as GradientFormatId)"
          />
          <p class="mt-1 text-xs text-gray-400">示例：{{ currentAdapter?.sample }}</p>
        </div>
        <div class="space-y-2">
          <SegmentedButtons
            v-if="currentAdapter?.supportsVanillaCharacter"
            :model-value="vanillaCharacter"
            :options="[
              { label: '&', value: '&' },
              { label: '§', value: '§' },
            ]"
            @update:model-value="emit('update:vanilla-character', $event as '&' | '§')"
          />
          <Checkbox
            v-if="currentAdapter?.supportsSimplify"
            :model-value="simplifyGradients"
            @update:model-value="emit('update:simplify-gradients', $event)"
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
</template>