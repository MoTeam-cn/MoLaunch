<script setup lang="ts">
/**
 * 全局消息抽屉
 * 替代原居中的 Modal 弹窗：错误 / 警告 / 信息 / 成功提示 + 确认 + 输入框模式
 * 统一从右侧滑出，复用公共 Drawer 组件（render-in-place 挂载到 #app-content）
 * defineExpose 对外接口与旧 Modal.vue 完全一致，utils/modal.ts 调用方无需改动
 */
import { ref, computed, nextTick } from 'vue'
import {
  ExclamationTriangleIcon,
  XCircleIcon,
  InformationCircleIcon,
  CheckCircleIcon,
  ChevronDownIcon,
} from '@heroicons/vue/24/outline'
import Drawer from '@/components/common/Drawer.vue'
import Button from '@/components/common/Button.vue'
import { copyToClipboard } from '@/utils/clipboard'

export type ModalType = 'error' | 'warning' | 'info' | 'success'

interface ModalOptions {
  type: ModalType
  title: string
  message: string
  details?: string
  confirmText?: string
  showCancel?: boolean
  cancelText?: string
  onConfirm?: () => void
  onCancel?: () => void
  /** 输入框模式：显示输入框，确认时回调接收输入值 */
  showInput?: boolean
  inputValue?: string
  inputPlaceholder?: string
  onConfirmInput?: (value: string) => void
}

const visible = ref(false)
const showDetails = ref(false)
/** 消息队列：同时触发多条时排队，当前一条关闭后依次展示，保证不丢任何一条 */
const queue = ref<ModalOptions[]>([])
const options = ref<ModalOptions>({
  type: 'info',
  title: '',
  message: '',
})
const inputValue = ref('')
const inputRef = ref<HTMLInputElement | null>(null)

const icon = computed(() => {
  switch (options.value.type) {
    case 'error': return XCircleIcon
    case 'warning': return ExclamationTriangleIcon
    case 'success': return CheckCircleIcon
    default: return InformationCircleIcon
  }
})

const iconColor = computed(() => {
  switch (options.value.type) {
    case 'error': return 'text-red-500'
    case 'warning': return 'text-amber-500'
    case 'success': return 'text-green-500'
    default: return 'text-primary-500'
  }
})

/** 展示队首项 */
function openCurrent() {
  const opts = queue.value[0]
  if (!opts) return
  options.value = opts
  showDetails.value = false
  inputValue.value = opts.inputValue ?? ''
  visible.value = true
  // 输入框模式下自动聚焦
  if (opts.showInput) {
    nextTick(() => {
      inputRef.value?.focus()
      inputRef.value?.select()
    })
  }
}

function show(opts: ModalOptions) {
  queue.value.push(opts)
  // 仅在没有正在展示的弹窗时立即打开；否则等当前项关闭后由 onDrawerClose 依次展示
  if (queue.value.length === 1) {
    openCurrent()
  }
}

function handleConfirm() {
  const opts = queue.value[0]
  if (!opts) return
  visible.value = false
  if (opts.showInput) {
    opts.onConfirmInput?.(inputValue.value)
  } else {
    opts.onConfirm?.()
  }
}

function handleCancel() {
  const opts = queue.value[0]
  if (!opts) return
  visible.value = false
  opts.onCancel?.()
}

/** Drawer 关闭（关闭按钮 / 点击遮罩 / ESC）统一视为取消 */
function handleVisibleChange(v: boolean) {
  if (!v) handleCancel()
}

/** Drawer 关闭动画结束：弹出队首，继续展示队列中的下一条 */
function onDrawerClose() {
  queue.value.shift()
  if (queue.value.length > 0) {
    openCurrent()
  }
}

async function copyDetails() {
  if (!options.value.details) return
  await copyToClipboard(options.value.details, { toast: true })
}

defineExpose({
  show,
  error: (title: string, message: string, details?: string) => {
    show({ type: 'error', title, message, details })
  },
  warning: (title: string, message: string, details?: string) => {
    show({ type: 'warning', title, message, details })
  },
  info: (title: string, message: string, details?: string) => {
    show({ type: 'info', title, message, details })
  },
  success: (title: string, message: string, details?: string) => {
    show({ type: 'success', title, message, details })
  },
  confirm: (title: string, message: string, onConfirm: () => void, onCancel?: () => void) => {
    show({
      type: 'warning',
      title,
      message,
      showCancel: true,
      onConfirm,
      onCancel,
    })
  },
  prompt: (
    title: string,
    message: string,
    onConfirm: (value: string) => void,
    opts?: { defaultValue?: string; placeholder?: string; onCancel?: () => void },
  ) => {
    show({
      type: 'info',
      title,
      message,
      showInput: true,
      showCancel: true,
      inputValue: opts?.defaultValue ?? '',
      inputPlaceholder: opts?.placeholder,
      onConfirmInput: onConfirm,
      onCancel: opts?.onCancel,
    })
  },
})
</script>

<template>
  <Drawer
    :visible="visible"
    placement="right"
    :width="520"
    render-in-place
    popup-container="#app-content"
    @update:visible="handleVisibleChange"
    @close="onDrawerClose"
  >
    <template #title>
      <div class="flex items-center gap-1.5 min-w-0">
        <component :is="icon" class="h-4 w-4 shrink-0" :class="iconColor" />
        <span class="truncate">{{ options.title }}</span>
      </div>
    </template>

    <!-- 消息（支持 \n 换行） -->
    <p class="text-sm text-gray-600 leading-relaxed whitespace-pre-line break-words">{{ options.message }}</p>

    <!-- 输入框 -->
    <div v-if="options.showInput" class="mt-4">
      <input
        ref="inputRef"
        v-model="inputValue"
        type="text"
        class="w-full rounded-lg border border-gray-300 px-3.5 py-2.5 text-sm text-gray-900 outline-none transition-colors focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
        :placeholder="options.inputPlaceholder"
        @keyup.enter="handleConfirm"
      >
    </div>

    <!-- 详情 -->
    <div v-if="options.details" class="mt-4">
      <Button
        type="text"
        size="mini"
        :aria-expanded="showDetails"
        @click="showDetails = !showDetails"
      >
        查看详情
        <ChevronDownIcon
          class="h-3.5 w-3.5 transition-transform duration-300 ease-in-out"
          :class="showDetails ? 'rotate-180' : ''"
        />
      </Button>
      <div
        class="grid transition-all duration-300 ease-in-out"
        :class="showDetails ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
      >
        <div class="overflow-hidden">
          <div class="mt-2 p-2.5 bg-gray-50 rounded text-xs text-gray-600 font-mono break-all max-h-40 overflow-y-auto">
            {{ options.details }}
            <Button
              type="text"
              size="mini"
              class="mt-1.5"
              @click="copyDetails"
            >
              复制
            </Button>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center gap-2">
        <!-- 抽屉底部左侧：待看消息计数（灰色小字） -->
        <span v-if="queue.length > 1" class="text-xs text-gray-400">还有 {{ queue.length - 1 }} 个待看</span>
        <div class="ml-auto flex justify-end gap-2">
          <Button
            v-if="options.showCancel"
            type="ghost"
            size="small"
            @click="handleCancel"
          >
            {{ options.cancelText || '取消' }}
          </Button>
          <Button
            type="primary"
            size="small"
            @click="handleConfirm"
          >
            {{ options.confirmText || '确定' }}
          </Button>
        </div>
      </div>
    </template>
  </Drawer>
</template>
