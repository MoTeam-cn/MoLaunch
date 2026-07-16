<script setup lang="ts">
/**
 * 横向滚动筛选组件
 * 单行展示筛选项，可横向滚动，选中项高亮
 * 参考 PCL2 PageDownloadCompDetail CardFilter 横向 RadioButton
 *
 * 左右箭头：常驻显示（有空间可滑时），点击平滑滚动
 * 滚动条：默认隐藏，hover 容器时显示
 */

import { ref, watch, nextTick, onMounted, onBeforeUnmount } from 'vue'
import { ChevronLeftIcon, ChevronRightIcon } from '@heroicons/vue/24/outline'

interface Option {
  label: string
  value: string
}

interface Props {
  modelValue: string
  options: Option[]
  /** 全部选项的标签（为空则不显示"全部"按钮） */
  allLabel?: string
  /** 全部选项对应的值 */
  allValue?: string
}

const props = withDefaults(defineProps<Props>(), {
  allLabel: '全部',
  allValue: '',
})

const emit = defineEmits<{
  'update:modelValue': [v: string]
}>()

const scrollRef = ref<HTMLDivElement | null>(null)
const showLeftArrow = ref(false)
const showRightArrow = ref(false)

let resizeObserver: ResizeObserver | null = null

function select(v: string) {
  emit('update:modelValue', v)
}

/** 检查是否需要显示左右箭头 */
function checkArrows() {
  const el = scrollRef.value
  if (!el) return
  showLeftArrow.value = el.scrollLeft > 4
  showRightArrow.value = el.scrollLeft + el.clientWidth < el.scrollWidth - 4
}

/** 滚动一段距离 */
function scrollBy(direction: 'left' | 'right') {
  const el = scrollRef.value
  if (!el) return
  const delta = el.clientWidth * 0.7
  el.scrollBy({ left: direction === 'left' ? -delta : delta, behavior: 'smooth' })
}

/** 将选中项滚动到可见区域 */
async function scrollToSelected() {
  await nextTick()
  const el = scrollRef.value
  if (!el) return
  const active = el.querySelector<HTMLElement>('[data-active="true"]')
  if (active) {
    active.scrollIntoView({ block: 'nearest', inline: 'nearest', behavior: 'smooth' })
  }
  checkArrows()
}

watch(() => props.modelValue, scrollToSelected)
watch(() => props.options, () => nextTick(checkArrows), { deep: true })

onMounted(() => {
  // 用 ResizeObserver 监听容器和内容尺寸变化，确保箭头状态准确
  const el = scrollRef.value
  if (el) {
    resizeObserver = new ResizeObserver(() => checkArrows())
    resizeObserver.observe(el)
    // 也观察内容变化（子元素增减导致宽度变化）
    nextTick(() => {
      if (el.firstElementChild) {
        resizeObserver?.observe(el.firstElementChild)
      }
    })
  }
  checkArrows()
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  resizeObserver = null
})

defineExpose({ checkArrows })
</script>

<template>
  <div class="relative flex items-center group">
    <!-- 左箭头 -->
    <button
      v-if="showLeftArrow"
      class="shrink-0 absolute left-0 z-10 w-6 h-7 flex items-center justify-center bg-white/90 backdrop-blur-sm rounded-md shadow-sm hover:bg-gray-50 transition-opacity"
      @click="scrollBy('left')"
    >
      <ChevronLeftIcon class="w-4 h-4 text-gray-500" />
    </button>

    <!-- 横向滚动容器 -->
    <div
      ref="scrollRef"
      class="flex-1 flex gap-1.5 overflow-x-auto scroll-area"
      @scroll.passive="checkArrows"
    >
      <!-- 全部按钮 -->
      <button
        v-if="allLabel"
        data-option
        :data-active="modelValue === allValue"
        class="shrink-0 px-2.5 py-1 rounded-md text-xs font-medium transition-colors"
        :class="modelValue === allValue
          ? 'bg-primary-600 text-white'
          : 'bg-white text-gray-600 border border-gray-200 hover:border-primary-300 hover:text-primary-600'"
        @click="select(allValue)"
      >
        {{ allLabel }}
      </button>

      <!-- 筛选项 -->
      <button
        v-for="opt in options"
        :key="opt.value"
        data-option
        :data-active="modelValue === opt.value"
        class="shrink-0 px-2.5 py-1 rounded-md text-xs font-medium transition-colors"
        :class="modelValue === opt.value
          ? 'bg-primary-600 text-white'
          : 'bg-white text-gray-600 border border-gray-200 hover:border-primary-300 hover:text-primary-600'"
        @click="select(opt.value)"
      >
        {{ opt.label }}
      </button>
    </div>

    <!-- 右箭头 -->
    <button
      v-if="showRightArrow"
      class="shrink-0 absolute right-0 z-10 w-6 h-7 flex items-center justify-center bg-white/90 backdrop-blur-sm rounded-md shadow-sm hover:bg-gray-50 transition-opacity"
      @click="scrollBy('right')"
    >
      <ChevronRightIcon class="w-4 h-4 text-gray-500" />
    </button>
  </div>
</template>

<style scoped>
/*
 * 滚动条平滑淡入方案：
 * - 滚动条高度始终 6px（不变，避免 height 0→6 瞬变）
 * - thumb 默认 background-color: transparent
 * - hover 时只改变 thumb 颜色（transition: background-color 平滑过渡）
 *
 * 注意：不能用 scrollbar-width: none，Edge/Chromium 会识别该属性并隐藏整个滚动条，
 * 导致 ::-webkit-scrollbar 规则失效。这里用 transparent thumb 实现"默认隐藏"效果。
 */
.scroll-area {
  -ms-overflow-style: none;
}
/* webkit：高度固定，thumb 颜色透明 → hover 时淡入 */
.scroll-area::-webkit-scrollbar {
  height: 6px;
}
.scroll-area::-webkit-scrollbar-track {
  background: transparent;
}
.scroll-area::-webkit-scrollbar-thumb {
  background-color: transparent;
  border-radius: 3px;
  transition: background-color 0.25s ease;
}
.scroll-area:hover::-webkit-scrollbar-thumb {
  background-color: rgba(156, 163, 175, 0.5);
}
.scroll-area:hover::-webkit-scrollbar-thumb:hover {
  background-color: rgba(107, 114, 128, 0.7);
}
</style>
