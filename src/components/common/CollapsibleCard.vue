<script setup lang="ts">
/**
 * 可折叠卡片组件
 *
 * 点击标题可展开/折叠内容，默认折叠状态可配置。
 * 使用 grid-template-rows 0fr→1fr 实现平滑高度过渡动画。
 *
 * 展开时 emit `expand` 事件，父组件可据此做懒加载（首次展开才请求数据）。
 */
import { ref } from 'vue'
import { ChevronDownIcon } from '@heroicons/vue/24/outline'

interface Props {
  title?: string
  defaultOpen?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  title: '',
  defaultOpen: false,
})

const emit = defineEmits<{
  (e: 'expand'): void
}>()

const isOpen = ref(props.defaultOpen)

function toggle() {
  isOpen.value = !isOpen.value
  if (isOpen.value) emit('expand')
}
</script>

<template>
  <div class="overflow-hidden rounded-lg border border-gray-200 bg-white">
    <!-- 标题栏（可点击折叠） -->
    <!-- 用 div + @click 而非 button，避免 actions slot 中的按钮产生嵌套 button -->
    <div
      class="flex w-full items-center justify-between px-4 py-3 text-left transition-colors hover:bg-gray-50 cursor-pointer"
      @click="toggle"
    >
      <span class="text-sm font-semibold text-gray-800">
        <slot name="title">{{ title }}</slot>
      </span>
      <div class="flex items-center gap-2" @click.stop>
        <slot name="actions" />
        <ChevronDownIcon
          class="h-4 w-4 flex-none text-gray-400 transition-transform duration-300 ease-in-out"
          :class="isOpen ? 'rotate-180' : ''"
        />
      </div>
    </div>

    <!-- 内容区（grid-template-rows 动画） -->
    <div
      class="grid transition-all duration-300 ease-in-out"
      :class="isOpen ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
    >
      <div class="overflow-hidden">
        <div class="border-t border-gray-100 px-4 py-3">
          <slot />
        </div>
      </div>
    </div>
  </div>
</template>
