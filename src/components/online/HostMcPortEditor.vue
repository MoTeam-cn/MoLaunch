<script setup lang="ts">
/**
 * 房主 MC 端口编辑器
 *
 * 自动捕获显示 + 手动覆盖：手动指定端口为最高可信度（自动捕获不再覆盖），
 * 确认后由父级经 HOST_MC_PORT 控制消息广播给参与者。
 */
import { ref, defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
import { PencilSquareIcon } from '@heroicons/vue/24/outline'

const props = defineProps<{
  /** 当前端口值 */
  value: number
  /** 是否手动指定（手动时自动捕获不再覆盖） */
  manual: boolean
}>()

const emit = defineEmits<{
  /** 手动确认端口 */
  confirm: [port: number]
  /** 清除手动标记，恢复自动捕获 */
  clear: []
}>()

const editing = ref(false)
const draft = ref('')

function startEdit() {
  draft.value = String(props.value)
  editing.value = true
}

function confirm() {
  const port = Number(draft.value)
  if (!Number.isInteger(port) || port <= 0 || port > 65535) return
  emit('confirm', port)
  editing.value = false
}

function cancel() {
  editing.value = false
}
</script>

<template>
  <div class="flex items-center gap-2">
    <template v-if="editing">
      <Input v-model="draft" type="number" placeholder="端口" width="110px" />
      <Button type="primary" size="small" @click="confirm">确定</Button>
      <Button type="ghost" size="small" @click="cancel">取消</Button>
    </template>
    <template v-else>
      <span class="text-xs text-gray-900">{{ value || '-' }}</span>
      <Tag v-if="manual" size="small" color="arcoblue">手动</Tag>
      <Button v-if="manual" type="text" size="small" @click="emit('clear')">恢复自动</Button>
      <Button v-else type="ghost" size="small" @click="startEdit">
        <template #icon><PencilSquareIcon class="w-3.5 h-3.5" /></template>
        编辑
      </Button>
    </template>
  </div>
</template>
