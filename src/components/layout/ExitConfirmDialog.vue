<script setup lang="ts">
/**
 * 退出选择弹框：点击关闭按钮时询问"直接退出 / 保留托盘关闭主界面"
 *
 * 勾选「下次不再提醒」后，将本次选择持久化到 closeBehavior 配置（由父组件执行）。
 */
import { ref, watch, defineAsyncComponent } from 'vue'
import { ArrowRightOnRectangleIcon, XMarkIcon } from '@heroicons/vue/24/outline'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Checkbox = defineAsyncComponent(() => import('@/components/common/Checkbox.vue'))

interface Props {
  modelValue: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  confirm: [{ action: 'exit' | 'tray'; remember: boolean }]
}>()

const remember = ref(false)

// 每次打开时重置"记住选择"勾选状态（未勾选则下次继续询问）
watch(
  () => props.modelValue,
  (show) => {
    if (show) remember.value = false
  },
)

function choose(action: 'exit' | 'tray') {
  emit('confirm', { action, remember: remember.value })
}

function cancel() {
  emit('update:modelValue', false)
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
        v-if="modelValue"
        class="fixed inset-0 z-[10000] flex items-center justify-center p-4"
        @click.self="cancel"
      >
        <div class="absolute inset-0 bg-black/40" />

        <div class="relative w-full max-w-md bg-white rounded-lg shadow-xl">
          <!-- 内容 -->
          <div class="p-6">
            <div class="flex items-center gap-3">
              <ArrowRightOnRectangleIcon class="w-6 h-6 shrink-0 text-primary-500" />
              <h3 class="text-base font-semibold text-gray-900 whitespace-nowrap">退出确认</h3>
            </div>
            <p class="mt-2 ml-9 text-sm text-gray-500 leading-relaxed whitespace-nowrap">
              关闭 MoLaunch 主界面后如何处理？
            </p>
          </div>

          <!-- 底部栏：参考 Windows 原生对话框，复选框靠左下角，操作按钮靠右下角 -->
          <div class="flex items-center justify-between gap-2 px-6 py-3.5 bg-gray-50 rounded-b-lg">
            <Checkbox v-model="remember">下次不再提醒</Checkbox>
            <div class="flex flex-none items-center gap-2">
              <Button type="ghost" size="small" @click="choose('tray')">
                保留托盘
              </Button>
              <Button type="primary" size="small" @click="choose('exit')">
                直接退出
              </Button>
            </div>
          </div>

          <!-- 右上角关闭（等同取消） -->
          <!-- 保留原生 button：此处为 28px 图标式关闭钮，Button.vue 的 scoped size 类固定宽高无法覆盖 -->
          <button
            class="absolute top-3.5 right-3.5 w-7 h-7 flex items-center justify-center rounded hover:bg-gray-100 text-gray-400 hover:text-gray-600 transition-colors"
            @click="cancel"
          >
            <XMarkIcon class="w-4 h-4" />
          </button>
        </div>
      </div>
    </transition>
  </teleport>
</template>
