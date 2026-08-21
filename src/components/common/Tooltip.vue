<script setup lang="ts">
/**
 * 自定义 Tooltip 组件
 * Teleport 到 body，fixed 定位，自动边界检测
 *
 * 与 Select 下拉框的协调：Tooltip 显示期间通过 MutationObserver 监听
 * body 子节点变化（select-dropdown 通过 teleported 加入/移除 body），
 * 一旦检测到下拉框与拟放置位置重叠，自动切换 top/bottom 方向，
 * 避免遮挡下拉框选项导致无法框选。
 */

import { computed, nextTick, onMounted, onUnmounted, onUpdated, ref } from 'vue'

interface Props {
  text: string
  position?: 'top' | 'bottom' | 'left' | 'right'
  delay?: number
  /** 是否以块级元素渲染（宽度撑满父容器），默认 false（inline-flex，宽度收缩到内容） */
  block?: boolean
  /** 仅当内容被省略（文本溢出）时才显示 tooltip，默认 false（始终显示） */
  overflowOnly?: boolean
  /** 单行显示（nowrap + 不设最大宽度），用于路径/名称需完整一行展示的场景，默认 false */
  singleLine?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  position: 'top',
  delay: 300,
  block: false,
  overflowOnly: false,
  singleLine: false,
})

const visible = ref(false)
const triggerRef = ref<HTMLElement | null>(null)
const tipRef = ref<HTMLElement | null>(null)
const tipStyle = ref<Record<string, string>>({})
const arrowStyle = ref<Record<string, string>>({})
/** overflowOnly 模式：内容是否被省略（存在 scrollWidth > clientWidth 的元素） */
const overflowed = ref(false)
const canShow = computed(() => !props.overflowOnly || overflowed.value)
let timer: ReturnType<typeof setTimeout> | null = null
let hideTimer: ReturnType<typeof setTimeout> | null = null
let observer: MutationObserver | null = null
let resizeObserver: ResizeObserver | null = null

/** 递归查找 trigger 子树中文本溢出（scrollWidth > clientWidth）的元素 */
function detectOverflow(): boolean {
  const root = triggerRef.value
  if (!root) return false
  const stack: Element[] = [...root.children]
  while (stack.length) {
    const el = stack.pop()!
    if (el.scrollWidth > el.clientWidth + 1) return true
    if (el.children.length) stack.push(...el.children)
  }
  return false
}

function refreshOverflow() {
  overflowed.value = detectOverflow()
}

onMounted(() => {
  if (!props.overflowOnly) return
  refreshOverflow()
  if (triggerRef.value) {
    resizeObserver = new ResizeObserver(refreshOverflow)
    resizeObserver.observe(triggerRef.value)
  }
})

onUpdated(refreshOverflow)

function show() {
  // overflowOnly 模式下内容未省略时不需要提示
  if (props.overflowOnly && !overflowed.value) return
  // 清除待执行的隐藏定时器（鼠标从 tooltip body 移回 trigger 时取消隐藏）
  if (hideTimer) { clearTimeout(hideTimer); hideTimer = null }
  timer = setTimeout(() => {
    visible.value = true
    nextTick(() => {
      calcPosition()
      startObserver()
    })
  }, props.delay)
}

function hide() {
  if (timer) { clearTimeout(timer); timer = null }
  // 延迟隐藏，给鼠标从 trigger 移到 tooltip body 的时间（便于复制文字）
  hideTimer = setTimeout(() => {
    visible.value = false
    stopObserver()
  }, 150)
}

/** 鼠标移入 tooltip body 时取消隐藏（支持复制文字） */
function cancelHide() {
  if (hideTimer) { clearTimeout(hideTimer); hideTimer = null }
}

/** 检测拟放置矩形是否与任何 select-dropdown 重叠（用于避让 Select 下拉框） */
function overlapsSelectDropdown(top: number, left: number, tipW: number, tipH: number): boolean {
  const dropdowns = document.querySelectorAll('.select-dropdown')
  for (const dd of dropdowns) {
    const r = dd.getBoundingClientRect()
    // 矩形相交判定
    if (!(top + tipH < r.top || top > r.bottom || left + tipW < r.left || left > r.right)) {
      return true
    }
  }
  return false
}

/** 按指定方向计算位置（不含边界修正），返回 {top,left,arrow} */
function calcByDirection(pos: 'top' | 'bottom' | 'left' | 'right') {
  const r = triggerRef.value!.getBoundingClientRect()
  const tipW = tipRef.value!.offsetWidth
  const tipH = tipRef.value!.offsetHeight
  const gap = 8
  const arrow: Record<string, string> = {}
  let top = 0
  let left = 0

  if (pos === 'top') {
    top = r.top - gap - tipH
    left = r.left + r.width / 2 - tipW / 2
    arrow.bottom = '-4px'
    arrow.left = '50%'
    arrow.transform = 'translateX(-50%) rotate(45deg)'
  } else if (pos === 'bottom') {
    top = r.bottom + gap
    left = r.left + r.width / 2 - tipW / 2
    arrow.top = '-4px'
    arrow.left = '50%'
    arrow.transform = 'translateX(-50%) rotate(45deg)'
  } else if (pos === 'left') {
    top = r.top + r.height / 2 - tipH / 2
    left = r.left - gap - tipW
    arrow.right = '-4px'
    arrow.top = '50%'
    arrow.transform = 'translateY(-50%) rotate(45deg)'
  } else {
    top = r.top + r.height / 2 - tipH / 2
    left = r.right + gap
    arrow.left = '-4px'
    arrow.top = '50%'
    arrow.transform = 'translateY(-50%) rotate(45deg)'
  }
  return { top, left, arrow }
}

function calcPosition() {
  if (!triggerRef.value || !tipRef.value) return
  const vw = window.innerWidth
  const vh = window.innerHeight
  const tipW = tipRef.value.offsetWidth
  const tipH = tipRef.value.offsetHeight

  let { top, left, arrow } = calcByDirection(props.position)

  // 与 select-dropdown 重叠时，top/bottom 方向自动切换避让
  if (overlapsSelectDropdown(top, left, tipW, tipH)) {
    if (props.position === 'top') {
      const alt = calcByDirection('bottom')
      if (!overlapsSelectDropdown(alt.top, alt.left, tipW, tipH)) {
        top = alt.top; left = alt.left; arrow = alt.arrow
      }
    } else if (props.position === 'bottom') {
      const alt = calcByDirection('top')
      if (!overlapsSelectDropdown(alt.top, alt.left, tipW, tipH)) {
        top = alt.top; left = alt.left; arrow = alt.arrow
      }
    }
    // left/right 暂不处理（与 select-dropdown 竖向重叠少见）
  }

  // 边界修正
  // top 下限 56px = 顶部 nav 48px + 8px 间距，避免 tooltip 遮蔽顶部 nav
  if (left < 8) left = 8
  if (left + tipW > vw - 8) left = vw - tipW - 8
  if (top < 56) top = 56
  if (top + tipH > vh - 8) top = vh - tipH - 8

  tipStyle.value = {
    position: 'fixed',
    top: `${top}px`,
    left: `${left}px`,
    zIndex: '9999',
  }
  arrowStyle.value = arrow
}

/** Tooltip 显示期间监听 body 子节点变化（select-dropdown 的出现/消失），自动重新计算位置 */
function startObserver() {
  if (observer) return
  observer = new MutationObserver(() => {
    if (visible.value) calcPosition()
  })
  observer.observe(document.body, { childList: true })
}

function stopObserver() {
  if (observer) {
    observer.disconnect()
    observer = null
  }
}

onUnmounted(() => {
  if (timer) clearTimeout(timer)
  if (hideTimer) clearTimeout(hideTimer)
  stopObserver()
  resizeObserver?.disconnect()
})
</script>

<template>
  <div ref="triggerRef" class="tooltip-trigger" :class="{ 'tooltip-trigger--block': block }" @mouseenter="show" @mouseleave="hide" @focus="show" @blur="hide">
    <slot />
    <teleport to="body">
      <div
        v-if="visible && text && canShow"
        ref="tipRef"
        :style="tipStyle"
        class="tooltip-body"
        :class="{ 'tooltip-body--preline': text.includes('\n'), 'tooltip-body--single': singleLine }"
        @mouseenter="cancelHide"
        @mouseleave="hide"
      >
        {{ text }}
        <div class="tooltip-arrow" :style="arrowStyle"></div>
      </div>
    </teleport>
  </div>
</template>

<style scoped>
.tooltip-trigger {
  display: inline-flex;
}
.tooltip-trigger--block {
  display: flex;
  width: 100%;
}
</style>

<style>
.tooltip-body {
  position: relative;
  padding: 6px 12px;
  background: #1f2937;
  color: #f9fafb;
  font-size: 12px;
  line-height: 1.5;
  border-radius: 6px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  width: max-content;
  max-width: 360px;
  /* 默认 normal：避免 pre-line 触发 Chromium max-content 退化（中文/短文本提前换行）；
     含 \n 的多行文本由 .tooltip-body--preline 单独启用 pre-line */
  white-space: normal;
  /* 支持鼠标移入复制文字 */
  cursor: text;
  user-select: text;
}

/* 含 \n 的多行提示（如设置项说明）启用 pre-line 保留换行 */
.tooltip-body--preline {
  white-space: pre-line;
}

/* 单行显示：路径/名称需完整一行展示（避免在连字符等处自然断行） */
.tooltip-body--single {
  white-space: nowrap;
  max-width: none;
}

.tooltip-arrow {
  position: absolute;
  width: 8px;
  height: 8px;
  background: #1f2937;
  box-shadow: 1px 1px 2px rgba(0, 0, 0, 0.1);
}
</style>
