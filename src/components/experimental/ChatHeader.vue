<script setup lang="ts">
/**
 * AI 聊天头部：会话标题、模型选择、思考设置（图标 + 右侧抽屉）、清空按钮
 */
import { computed, ref, defineAsyncComponent } from 'vue'
import { TrashIcon, AdjustmentsHorizontalIcon } from '@heroicons/vue/24/outline'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Checkbox = defineAsyncComponent(() => import('@/components/common/Checkbox.vue'))
const Slider = defineAsyncComponent(() => import('@/components/common/Slider.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const Drawer = defineAsyncComponent(() => import('@/components/common/Drawer.vue'))
const ModelIcon = defineAsyncComponent(() => import('@/components/common/ModelIcon.vue'))

const props = defineProps<{
  title: string
  activeId: number
  loading: boolean
  models: string[]
  currentModel: string
  /** 是否启用模型思考模式 */
  enableReasoning: boolean
  /** 思考程度（low/medium/high，对应滑块 0/50/100） */
  reasoningLevel: string
}>()

const emit = defineEmits<{
  'update:currentModel': [model: string]
  'update:enableReasoning': [value: boolean]
  'update:reasoningLevel': [value: string]
  clear: []
}>()

const modelOptions = (models: string[]) => models.map((m) => ({ label: m, value: m }))

/** 思考程度 ↔ 滑块值（0/50/100）双向映射 */
const LEVEL_VALUE: Record<string, number> = { low: 0, medium: 50, high: 100 }
const VALUE_LEVEL: Record<number, string> = { 0: 'low', 50: 'medium', 100: 'high' }
const THINK_MARKS = [
  { value: 0, label: '低' },
  { value: 50, label: '中' },
  { value: 100, label: '高' },
]
const reasoningValue = computed({
  get: () => LEVEL_VALUE[props.reasoningLevel] ?? 50,
  set: (v: number) => emit('update:reasoningLevel', VALUE_LEVEL[v] ?? 'medium'),
})

const reasoningLevelLabel = computed(() => {
  const label = THINK_MARKS.find((m) => m.value === LEVEL_VALUE[props.reasoningLevel])
  return label ? `档位：${label.label}` : '档位：中'
})

// ---- 思考设置抽屉（右侧 Drawer，遮罩点击 / X / ESC 关闭） ----
const settingsOpen = ref(false)
</script>

<template>
  <div class="flex items-center gap-2 border-b border-gray-200 px-4 py-2.5">
    <h3 class="min-w-0 flex-1 truncate text-sm font-semibold text-gray-900">{{ title || 'AI 聊天' }}</h3>
    <Tag v-if="activeId" color="primary" size="small">Agent 模式</Tag>

    <!-- 模型选择（品牌图标识别见 ModelIcon） -->
    <Select
      v-if="models.length > 0"
      :model-value="currentModel"
      :options="modelOptions(models)"
      size="small"
      class="w-60"
      @update:model-value="emit('update:currentModel', String($event))"
    >
      <template #selected="{ label }">
        <span class="flex items-center gap-1.5">
          <ModelIcon :model="label" class="h-4 w-4" />
          <span class="truncate">{{ label }}</span>
        </span>
      </template>
      <template #option="{ option, selected }">
        <span class="flex w-full items-center gap-1.5">
          <ModelIcon :model="option.label" class="h-4 w-4" />
          <span class="select-option-content min-w-0 flex-1 truncate">{{ option.label }}</span>
          <svg v-if="selected" viewBox="0 0 1024 1024" fill="currentColor" class="h-3.5 w-3.5 shrink-0 text-primary-500">
            <path d="M912 192c-12.8 0-25.6 4.266667-34.133333 12.8L384 699.2 234.666667 548.266667c-17.066667-17.066667-46.933333-17.066667-64 0-17.066667 17.066667-17.066667 46.933333 0 64l179.2 179.2c8.533333 8.533333 21.333333 12.8 34.133333 12.8s25.6-4.266667 34.133333-12.8l520.533334-520.533334c17.066667-17.066667 17.066667-46.933333 0-64-8.533333-8.533333-21.333333-12.8-34.133334-12.8z" />
          </svg>
        </span>
      </template>
    </Select>

    <!-- 思考设置：图标入口（开启时 primary 高亮）+ 右侧 Drawer 抽屉（遮罩点击 / X / ESC 关闭） -->
    <Tooltip :text="enableReasoning ? '思考设置' : '思考设置（已关闭）'">
      <button
        type="button"
        class="rounded-md p-1.5 transition-colors"
        :class="enableReasoning ? 'text-primary-500 hover:bg-primary-50' : 'text-gray-400 hover:bg-gray-100 hover:text-gray-600'"
        @click="settingsOpen = !settingsOpen"
      >
        <AdjustmentsHorizontalIcon class="h-4 w-4" />
      </button>
    </Tooltip>

    <Drawer
      v-model:visible="settingsOpen"
      placement="right"
      :width="320"
      title="思考设置"
      render-in-place
      popup-container="#app-content"
    >
      <div class="space-y-4">
        <div class="flex items-center justify-between">
          <span class="text-xs font-medium text-gray-600">思考模式</span>
          <Checkbox
            :model-value="enableReasoning"
            @update:model-value="emit('update:enableReasoning', $event)"
          >开启</Checkbox>
        </div>
        <div class="space-y-1.5">
          <div class="flex items-center justify-between">
            <span class="text-xs font-medium text-gray-600">思考程度</span>
            <span class="text-xs text-gray-400">{{ reasoningLevelLabel }}</span>
          </div>
          <Slider
            v-model="reasoningValue"
            :min="0"
            :max="100"
            :snap="[0, 50, 100]"
            :disabled="!enableReasoning"
            :meteor="enableReasoning"
            class="w-full"
          />
          <p class="text-[11px] leading-relaxed text-gray-400">
            开启后请求将携带 reasoning_effort，数值越高模型思考越深入、耗时更长。
          </p>
        </div>

        <!-- 免责声明：AI 内容由用户配置的模型端点生成，数据外发风险由用户自行承担 -->
        <div class="rounded-md border border-gray-200 bg-gray-50 px-3 py-2.5">
          <p class="text-[11px] font-medium text-gray-600">免责声明</p>
          <p class="mt-0.5 text-[11px] leading-relaxed text-gray-500">
            AI 回复由您自行配置的模型端点生成，MoLaunch 不参与内容生成与审核；请勿输入违法、敏感信息，数据外发与隐私风险请自行评估。
          </p>
        </div>
      </div>
    </Drawer>

    <Button v-if="activeId" type="ghost" size="mini" :disabled="loading" @click="emit('clear')">
      <template #icon><TrashIcon class="h-3.5 w-3.5" /></template>
      清空
    </Button>
  </div>
</template>
