<script setup lang="ts">
/**
 * AI 提问抽屉
 *
 * 后端 Agent 工具 `ask_user` 通过 `ai-ask-user` 事件请求用户确认（如选择游戏版本）。
 * 采用右侧 Drawer 抽屉（不打断主流程，可稍后回答）：
 * 选项支持 `label` 文本与 `description` 备注/注释，选中后经底部「提交」确认，
 * 也支持输入自定义答案回填；关闭（X / ESC / 点击遮罩）视为取消。
 */
import { computed, ref, watch } from 'vue'
import { CheckCircleIcon } from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import Drawer from '@/components/common/Drawer.vue'
import OverflowText from '@/components/common/OverflowText.vue'
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

/** 抽屉被关闭（X / ESC / 遮罩点击）统一视为取消提问 */
function onVisibleChange(v: boolean) {
  if (!v) emit('cancel')
}
</script>

<template>
  <Drawer
    :visible="visible"
    placement="right"
    :width="400"
    title="需要你的确认"
    render-in-place
    popup-container="#app-content"
    unmount-on-close
    @update:visible="onVisibleChange"
  >
    <OverflowText
      class="text-sm leading-relaxed font-medium text-gray-900"
      :text="question"
      :lines="3"
    />

    <div v-if="options.length > 0" class="mt-3 space-y-2">
      <div
        v-for="(opt, i) in options"
        :key="opt.label"
        role="button"
        tabindex="0"
        class="flex w-full cursor-pointer items-start gap-2.5 rounded-lg border-2 px-3 py-2 text-left transition-colors"
        :class="selectedIndex === i ? 'border-primary-500 bg-primary-50' : 'border-gray-200 hover:border-gray-300'"
        @click="toggleOption(i)"
        @keydown.enter="toggleOption(i)"
        @keydown.space.prevent="toggleOption(i)"
      >
        <CheckCircleIcon
          class="mt-0.5 h-5 w-5 shrink-0 transition-colors"
          :class="selectedIndex === i ? 'text-primary-500' : 'text-gray-300'"
        />
        <span class="min-w-0 flex-1">
          <OverflowText :text="opt.label" :lines="1" class="text-sm font-medium text-gray-900" />
          <OverflowText
            v-if="opt.description"
            :text="opt.description"
            :lines="2"
            class="mt-0.5 text-xs leading-relaxed text-gray-500"
          />
        </span>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center gap-2">
        <Input
          v-model="custom"
          class="flex-1"
          placeholder="输入自定义答案"
          @keydown.enter="submit()"
        />
        <Button type="ghost" size="small" @click="emit('cancel')">取消</Button>
        <Button type="primary" size="small" :disabled="!canSubmit" @click="submit()">提交</Button>
      </div>
    </template>
  </Drawer>
</template>
