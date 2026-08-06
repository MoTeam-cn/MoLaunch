<script setup lang="ts">
/**
 * AI 提问悬浮卡片
 *
 * 后端 Agent 工具 `ask_user` 通过 `ai-ask-user` 事件请求用户确认（如选择游戏版本）。
 * 采用右下角悬浮卡片（非全局遮挡弹窗）：不阻断页面其他操作，可稍后回答；
 * 选项支持 `label` 文本与 `description` 备注/注释，选中后经底部「提交」确认，
 * 也支持输入自定义答案回填。
 */
import { computed, ref, watch } from 'vue'
import { CheckCircleIcon, SparklesIcon, XMarkIcon } from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import type { AskUserOption } from '@/utils/api/experimental'

const props = defineProps<{
  visible: boolean
  question: string
  options: AskUserOption[]
}>()

const emit = defineEmits<{
  submit: [reply: string]
  cancel: []
}>()

const custom = ref('')
const selectedIndex = ref<number | null>(null)

watch(
  () => props.visible,
  (v) => {
    if (v) {
      custom.value = ''
      selectedIndex.value = null
    }
  },
)

const canSubmit = computed(() => custom.value.trim() !== '' || selectedIndex.value !== null)

function toggleOption(index: number) {
  selectedIndex.value = selectedIndex.value === index ? null : index
}

function submit() {
  const typed = custom.value.trim()
  const selected = selectedIndex.value !== null ? props.options[selectedIndex.value] : undefined
  const reply = typed || selected?.label || ''
  if (!reply) return
  emit('submit', reply)
}
</script>

<template>
  <teleport to="body">
    <transition
      enter-active-class="transition ease-out duration-[180ms]"
      enter-from-class="opacity-0 translate-y-2"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 translate-y-2"
    >
      <div
        v-if="visible"
        class="fixed right-4 bottom-4 z-[10000] w-80 max-w-[calc(100vw-2rem)] overflow-hidden rounded-xl border border-gray-200 bg-white shadow-lg"
        role="dialog"
        aria-label="AI 需要你的确认"
      >
        <div class="bg-gradient-to-r from-primary-500 to-primary-400 px-4 py-3">
          <div class="flex items-center gap-2">
            <SparklesIcon class="h-5 w-5 shrink-0 text-white" />
            <h3 class="min-w-0 flex-1 truncate text-sm font-semibold text-white">需要你的确认</h3>
            <button
              class="shrink-0 rounded-md p-1 text-white/80 transition-colors hover:bg-gray-100 hover:text-gray-600"
              title="稍后回答"
              @click="emit('cancel')"
            >
              <XMarkIcon class="h-4 w-4" />
            </button>
          </div>
        </div>

        <div class="px-4 py-3 text-sm leading-relaxed whitespace-pre-line font-medium text-gray-900">
          {{ question }}
        </div>

        <div v-if="options.length > 0" class="max-h-56 space-y-2 overflow-y-auto px-4 pb-1">
          <button
            v-for="(opt, i) in options"
            :key="opt.label"
            class="flex w-full items-start gap-2.5 rounded-lg border-2 px-3 py-2 text-left transition-colors"
            :class="
              selectedIndex === i
                ? 'border-primary-500 bg-primary-50'
                : 'border-gray-200 hover:border-gray-300'
            "
            @click="toggleOption(i)"
          >
            <CheckCircleIcon
              class="mt-0.5 h-5 w-5 shrink-0 transition-colors"
              :class="selectedIndex === i ? 'text-primary-500' : 'text-gray-300'"
            />
            <span class="min-w-0 flex-1">
              <span class="block text-sm font-medium text-gray-900">{{ opt.label }}</span>
              <span v-if="opt.description" class="mt-0.5 block text-xs leading-relaxed text-gray-500">
                {{ opt.description }}
              </span>
            </span>
          </button>
        </div>

        <div class="flex items-center gap-2 px-4 py-3">
          <Input
            v-model="custom"
            class="flex-1"
            placeholder="输入自定义答案"
            @keydown.enter="submit()"
          />
          <Button type="ghost" size="small" @click="emit('cancel')">取消</Button>
          <Button type="primary" size="small" :disabled="!canSubmit" @click="submit()">提交</Button>
        </div>
      </div>
    </transition>
  </teleport>
</template>
