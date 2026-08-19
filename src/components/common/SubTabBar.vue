<script setup lang="ts">
/**
 * 顶部子菜单切换组件
 *
 * 用于页面内的子页签切换（如：关于 / 鸣谢 / 教程）。
 * 支持 sticky 固定模式，滚动时子菜单栏吸顶。
 * 底部选中指示条为单条滑动动画（跟随 active tab 平滑移动）。
 *
 * 用法：
 * <SubTabBar v-model="activeTab" :tabs="tabs" sticky />
 */
import { ref, nextTick, watch, onMounted } from 'vue'

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

const props = withDefaults(defineProps<Props>(), {
  sticky: false,
})
const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

/** 鼠标是否悬停在子菜单区域（控制细滚动条显隐） */
const hovered = ref(false)
const containerRef = ref<HTMLElement | null>(null)
/** 滑动指示条位置（left/width，相对容器内容原点） */
const indicatorStyle = ref({ left: '0px', width: '0px' })

/** 计算 active tab 的位置，驱动指示条滑动 */
function updateIndicator() {
  nextTick(() => {
    const container = containerRef.value
    if (!container) return
    const btn = container.querySelector<HTMLElement>(`[data-tab-id="${props.modelValue}"]`)
    if (!btn) return
    indicatorStyle.value = {
      left: `${btn.offsetLeft}px`,
      width: `${btn.offsetWidth}px`,
    }
  })
}

function selectTab(id: string) {
  emit('update:modelValue', id)
  // 自动滚动到可见区域：点击右侧被遮挡的 tab 时自动左滑，避免手动拖滚动条
  nextTick(() => {
    const container = containerRef.value
    if (!container) return
    const btn = container.querySelector<HTMLElement>(`[data-tab-id="${id}"]`)
    if (!btn) return
    const cRect = container.getBoundingClientRect()
    const bRect = btn.getBoundingClientRect()
    const pad = 8
    if (bRect.right > cRect.right) {
      container.scrollLeft += bRect.right - cRect.right + pad
    } else if (bRect.left < cRect.left) {
      container.scrollLeft -= cRect.left - bRect.left + pad
    }
  })
}

watch(() => props.modelValue, updateIndicator)
onMounted(updateIndicator)
</script>

<template>
  <div
    ref="containerRef"
    class="sub-tab-bar relative flex items-center gap-1 overflow-x-auto border-b border-gray-200 bg-white px-1"
    :class="[sticky ? 'sticky top-0 z-20 shadow-sm' : '', { 'show-scrollbar': hovered }]"
    @mouseenter="hovered = true"
    @mouseleave="hovered = false"
  >
    <!-- 保留原生 button：Tab 切换项（relative 布局 + active 状态），
         Button.vue 的 scoped size 类与布局不适合带指示线的 Tab 组件 -->
    <button
      v-for="tab in tabs"
      :key="tab.id"
      :data-tab-id="tab.id"
      class="relative flex flex-none items-center gap-1.5 whitespace-nowrap px-4 py-2.5 text-[13px] font-medium transition-colors"
      :class="modelValue === tab.id
        ? 'text-primary-600'
        : 'text-gray-500 hover:text-gray-700'"
      @click="selectTab(tab.id)"
    >
      <component :is="tab.icon" v-if="tab.icon" class="h-4 w-4" />
      {{ tab.label }}
    </button>
    <!-- 底部选中指示条：单条滑动动画，跟随 active tab 平滑移动 -->
    <span
      class="pointer-events-none absolute bottom-0 h-0.5 rounded-full bg-primary-500 transition-all duration-300 ease-out"
      :style="indicatorStyle"
    />
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