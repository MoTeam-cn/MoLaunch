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
import { ref } from 'vue'
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

function handleConfirm() {
  emit('confirm', selected.value)
}
</script>

<template>
  <teleport to="body">
    <transition
      enter-active-class="transition ease-out duration-150"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition ease-in duration-100"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        class="modal-shell"
        @click.self="emit('close')"
      >
        <div class="absolute inset-0 bg-black/40" />
        <div class="modal-body max-w-md mt-2">
          <!-- 标题栏 -->
          <div class="px-5 py-3.5 border-b border-gray-200">
            <h3 class="text-base font-semibold text-gray-900">踢出参与者</h3>
          </div>
          <!-- 内容区 -->
          <div class="modal-scroll px-5 py-4 space-y-3">
            <p class="text-sm text-gray-600">
              确定踢出
              <code class="bg-gray-100 px-1.5 py-0.5 rounded text-gray-800">{{ devicePk.slice(0, 12) }}...</code>
              <span v-if="virtualIp" class="text-gray-500">（{{ virtualIp }}）</span>？
            </p>
            <div class="space-y-2">
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
          </div>
          <!-- 底部按钮栏 -->
          <div class="flex justify-end gap-2 px-5 py-3.5 bg-gray-50 rounded-b-lg">
            <Button type="ghost" size="small" @click="emit('close')">取消</Button>
            <Button type="primary" size="small" @click="handleConfirm">确认踢出</Button>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>
