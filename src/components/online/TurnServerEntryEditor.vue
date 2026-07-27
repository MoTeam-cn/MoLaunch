<script setup lang="ts">
/**
 * 单个 TURN 服务器条目编辑器（阶段三子任务 7 阶段 H）
 *
 * 在 [SettingsOnline.vue](src/views/settings/SettingsOnline.vue) 的「ICE 服务器配置」
 * section 中，每个 v-for 项使用此组件。通过 v-model 双向绑定 `IceServerEntry`，
 * 支持 URL/username/credential 三字段编辑 + 移除按钮。
 *
 * # 设计
 *
 * - URL 输入支持逗号/空白分隔多 URL（自动 split 为 `urls` 数组）
 * - URL 校验：必须以 `turn:` / `turns:` / `stun:` 开头，否则显示 error hint
 * - `username` / `credential` 为可选字段（STUN 无需，TURN 必填）
 * - 移除按钮触发 `remove` 事件，父组件负责从列表中删除
 *
 * # 复用约定
 *
 * - 使用 [Input.vue](src/components/common/Input.vue) 而非原生 `<input>`
 * - 使用 [Button.vue](src/components/common/Button.vue) 而非原生 `<button>`
 * - 使用 [Tooltip.vue](src/components/common/Tooltip.vue) 而非原生 `title`
 */
import { computed } from 'vue'
import Input from '@/components/common/Input.vue'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import { TrashIcon } from '@heroicons/vue/24/outline'
import type { IceServerEntry } from '@/types/online'

const props = defineProps<{
  /** 当前编辑的 ICE 服务器条目 */
  modelValue: IceServerEntry
  /** 在列表中的序号（从 0 开始，用于标题展示 #N） */
  index: number
}>()

const emit = defineEmits<{
  'update:modelValue': [IceServerEntry]
  remove: []
}>()

/** URL 输入框值（将 `urls` 数组 join 为单行展示，输入时 split 回数组） */
const urlInput = computed({
  get: () => props.modelValue.urls.join(', '),
  set: (v: string) => {
    const urls = v
      .split(/[\s,]+/)
      .map((s) => s.trim())
      .filter(Boolean)
    emit('update:modelValue', { ...props.modelValue, urls })
  },
})

/** 用户名输入框值（空字符串转 undefined，避免序列化空字段） */
const usernameInput = computed({
  get: () => props.modelValue.username ?? '',
  set: (v: string) => {
    emit('update:modelValue', { ...props.modelValue, username: v || undefined })
  },
})

/** 凭据输入框值（空字符串转 undefined，避免序列化空字段） */
const credentialInput = computed({
  get: () => props.modelValue.credential ?? '',
  set: (v: string) => {
    emit('update:modelValue', { ...props.modelValue, credential: v || undefined })
  },
})

/** URL 校验：每个 URL 必须以 turn:/turns:/stun: 开头 */
const urlHint = computed(() => {
  const urls = props.modelValue.urls
  if (urls.length === 0) return '请输入 TURN 服务器地址（如 turn:turn.example.com:3478）'
  const invalid = urls.find((u) => !/^(turn|turns|stun):/i.test(u))
  if (invalid) return 'URL 格式错误：必须以 turn: / turns: / stun: 开头'
  return ''
})

const urlHintType = computed<'default' | 'error' | 'success'>(() => {
  if (props.modelValue.urls.length === 0) return 'default'
  return urlHint.value ? 'error' : 'success'
})
</script>

<template>
  <div class="px-5 py-4">
    <div class="flex items-center justify-between mb-2">
      <p class="text-sm font-medium text-gray-900">TURN 服务器 #{{ index + 1 }}</p>
      <Tooltip text="移除此条目">
        <Button
          type="ghost"
          size="mini"
          class="!h-7 !w-7 !p-0 text-gray-400 hover:!text-red-500"
          @click="emit('remove')"
        >
          <TrashIcon class="w-4 h-4" />
        </Button>
      </Tooltip>
    </div>
    <div class="space-y-2">
      <Input
        v-model="urlInput"
        placeholder="turn:turn.example.com:3478?transport=udp"
        :hint="urlHint"
        :hint-type="urlHintType"
        class="font-mono"
      />
      <div class="grid grid-cols-2 gap-2">
        <Input
          v-model="usernameInput"
          placeholder="用户名（TURN 必填，STUN 留空）"
        />
        <Input
          v-model="credentialInput"
          placeholder="凭据（TURN 必填，STUN 留空）"
        />
      </div>
    </div>
  </div>
</template>