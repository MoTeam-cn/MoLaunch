<script setup lang="ts">
/**
 * 可折叠卡片组件
 *
 * 点击标题可展开/折叠内容，默认折叠状态可配置。
 * 使用 grid-template-rows 0fr→1fr 实现平滑高度过渡动画。
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

const isOpen = ref(props.defaultOpen)

function toggle() {
  isOpen.value = !isOpen.value
}
</script>

<template>
  <div class="overflow-hidden rounded-lg border border-gray-200 bg-white">
    <!-- 标题栏（可点击折叠） -->
    <button
      class="flex w-full items-center justify-between px-4 py-3 text-left transition-colors hover:bg-gray-50"
      @click="toggle"
    >
      <span class="text-sm font-semibold text-gray-800">
        <slot name="title">{{ title }}</slot>
      </span>
      <ChevronDownIcon
        class="h-4 w-4 flex-none text-gray-400 transition-transform duration-300 ease-in-out"
        :class="isOpen ? 'rotate-180' : ''"
      />
    </button>

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
