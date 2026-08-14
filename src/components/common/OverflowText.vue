<script setup lang="ts">
/**
 * 自适应省略文本组件
 *
 * 文本超出容器时按 `lines` 省略（单行 truncate / 多行 line-clamp），
 * 仅当实际发生溢出时才启用 Tooltip 展示完整内容（未溢出时不打扰）。
 *
 * 注意：`line-clamp-N` 需静态类名才能被 Tailwind JIT 扫描到，故用映射表；
 * 溢出检测用 ResizeObserver（父容器如 Drawer 展开后尺寸变化会自动重测）。
 */
import { onMounted, onUnmounted, ref, watch, defineAsyncComponent } from 'vue'
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))

interface Props {
  text: string
  /** 显示行数：1=单行省略，>1 多行省略 */
  lines?: number
}

const props = withDefaults(defineProps<Props>(), { lines: 1 })

/** 静态类映射（保证 Tailwind 可扫描到） */
const LINE_CLAMP: Record<number, string> = {
  1: 'line-clamp-1',
  2: 'line-clamp-2',
  3: 'line-clamp-3',
}

const textRef = ref<HTMLElement | null>(null)
const overflowed = ref(false)
let resizeObserver: ResizeObserver | null = null

function checkOverflow() {
  const el = textRef.value
  if (!el) return
  const tooWide = el.scrollWidth > el.clientWidth + 1
  const tooTall = el.scrollHeight > el.clientHeight + 1
  overflowed.value = props.lines <= 1 ? tooWide : tooTall
}

function startObserve() {
  if (resizeObserver || !textRef.value) return
  resizeObserver = new ResizeObserver(() => checkOverflow())
  resizeObserver.observe(textRef.value)
}

onMounted(() => {
  checkOverflow()
  startObserve()
  watch(
    () => props.text,
    () => requestAnimationFrame(checkOverflow),
    { immediate: true },
  )
})

onUnmounted(() => {
  resizeObserver?.disconnect()
  resizeObserver = null
})
</script>

<template>
  <Tooltip :text="overflowed ? text : ''" block position="top">
    <span
      v-if="lines <= 1"
      ref="textRef"
      class="block w-full min-w-0 truncate"
    >{{ text }}</span>
    <span
      v-else
      ref="textRef"
      class="w-full min-w-0"
      :class="LINE_CLAMP[lines] ?? 'line-clamp-1'"
    >{{ text }}</span>
  </Tooltip>
</template>
