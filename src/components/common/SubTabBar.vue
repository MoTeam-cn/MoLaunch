<script setup lang="ts">
/**
 * 顶部子菜单切换组件
 *
 * 用于页面内的子页签切换（如：关于 / 鸣谢 / 教程）。
 * 支持 sticky 固定模式，滚动时子菜单栏吸顶。
 *
 * 用法：
 * <SubTabBar v-model="activeTab" :tabs="tabs" sticky />
 */
import { ref } from 'vue'

interface Tab {
  id: string
  label: string
  icon?: any
}

interface Props {
  tabs: Tab[]
  modelValue: string
  sticky?: boolean
}

withDefaults(defineProps<Props>(), {
  sticky: false,
})
const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

/** 鼠标是否悬停在子菜单区域（控制细滚动条显隐） */
const hovered = ref(false)

function selectTab(id: string) {
  emit('update:modelValue', id)
}
</script>

<template>
  <div
    class="sub-tab-bar flex items-center gap-1 overflow-x-auto border-b border-gray-200 bg-white px-1"
    :class="[sticky ? 'sticky top-0 z-20 shadow-sm' : '', { 'show-scrollbar': hovered }]"
    @mouseenter="hovered = true"
    @mouseleave="hovered = false"
  >
    <!-- 保留原生 button：Tab 切换项（relative 布局 + 底部指示线 + active 状态），
         Button.vue 的 scoped size 类与布局不适合带指示线的 Tab 组件 -->
    <button
      v-for="tab in tabs"
      :key="tab.id"
      class="relative flex flex-none items-center gap-1.5 whitespace-nowrap px-4 py-2.5 text-[13px] font-medium transition-colors"
      :class="modelValue === tab.id
        ? 'text-primary-600'
        : 'text-gray-500 hover:text-gray-700'"
      @click="selectTab(tab.id)"
    >
      <component :is="tab.icon" v-if="tab.icon" class="h-4 w-4" />
      {{ tab.label }}
      <!-- 底部选中指示线（所有 tab 均渲染，通过 opacity + scaleX 平滑过渡） -->
      <span
        class="absolute bottom-0 left-2 right-2 h-0.5 origin-center rounded-full bg-primary-500 transition-all duration-200 ease-out"
        :class="modelValue === tab.id ? 'opacity-100' : 'opacity-0'"
        :style="{ transform: modelValue === tab.id ? 'scaleX(1)' : 'scaleX(0)' }"
      />
    </button>
  </div>
</template>

<style scoped>
/* 细滚动条：默认隐藏，鼠标悬停在子菜单区域时显示（JS 控制，避免 CSS :hover 在滚动条上的怪异行为） */
.sub-tab-bar::-webkit-scrollbar {
  height: 4px;
}

.sub-tab-bar::-webkit-scrollbar-track {
  background: transparent;
}

.sub-tab-bar::-webkit-scrollbar-thumb {
  background: transparent;
  border-radius: 9999px;
}

.sub-tab-bar.show-scrollbar::-webkit-scrollbar-thumb {
  background: #d1d5db;
}

.sub-tab-bar.show-scrollbar::-webkit-scrollbar-thumb:hover {
  background: #9ca3af;
}
</style>
