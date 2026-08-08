<script setup lang="ts">
/**
 * 踢出确认弹窗（联机大厅阶段 6.1）
 *
 * 替代原 handleKick 内的 showConfirm，支持选择封禁时长：
 * - null  → 仅踢出，不封禁（可重新加入）
 * - 0     → 永久封禁
 * - N > 0 → 封禁 N 秒
 *
 * 封禁时长语义由 api-server 约定：ban_duration_seconds=0 表示永久。
 */
import { onMounted, ref } from 'vue'
import { ExclamationTriangleIcon } from '@heroicons/vue/24/outline'
import Drawer from '@/components/common/Drawer.vue'
import Button from '@/components/common/Button.vue'

defineProps<{
  devicePk: string
  virtualIp?: string
}>()

const emit = defineEmits<{
  close: []
  confirm: [banDuration: number | null]
}>()

type BanOption = { label: string; value: number | null; desc: string }
const banOptions: BanOption[] = [
  { label: '仅踢出', value: null, desc: '不封禁，可重新加入' },
  { label: '10 分钟', value: 600, desc: '临时封禁' },
  { label: '1 小时', value: 3600, desc: '临时封禁' },
  { label: '永久', value: 0, desc: '不可重新加入' },
]

const selected = ref<number | null>(null)

const visible = ref(false)
onMounted(() => {
  visible.value = true
})

/** 关闭（取消按钮 / 遮罩 / ESC 统一走此路径，通知父组件卸载） */
function handleClose() {
  visible.value = false
  emit('close')
}

function handleConfirm() {
  emit('confirm', selected.value)
}
</script>

<template>
  <Drawer
    :visible="visible"
    placement="right"
    :width="460"
    render-in-place
    popup-container="#app-content"
    @update:visible="handleClose"
  >
    <template #title>
      <div class="flex items-center gap-1.5">
        <ExclamationTriangleIcon class="h-4 w-4 text-amber-500" />
        <span>踢出参与者</span>
      </div>
    </template>

    <p class="text-sm text-gray-600">
      确定踢出
      <code class="bg-gray-100 px-1.5 py-0.5 rounded text-gray-800">{{ devicePk.slice(0, 12) }}...</code>
      <span v-if="virtualIp" class="text-gray-500">（{{ virtualIp }}）</span>？
    </p>

    <div class="mt-4 space-y-2">
      <div class="text-xs text-gray-500">封禁时长</div>
      <div class="grid grid-cols-2 gap-2">
        <div
          v-for="opt in banOptions"
          :key="String(opt.value)"
          role="radio"
          :aria-checked="selected === opt.value"
          class="px-3 py-2 text-sm rounded border transition-colors cursor-pointer"
          :class="selected === opt.value
            ? 'border-primary-500 bg-primary-50 text-primary-700'
            : 'border-gray-200 hover:border-gray-300 text-gray-700'"
          @click="selected = opt.value"
        >
          <div class="font-medium">{{ opt.label }}</div>
          <div class="text-xs text-gray-400 mt-0.5">{{ opt.desc }}</div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button type="ghost" size="small" @click="handleClose">取消</Button>
        <Button type="primary" size="small" @click="handleConfirm">确认踢出</Button>
      </div>
    </template>
  </Drawer>
</template>
