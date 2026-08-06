<!--
  MoLaunch - Minecraft Launcher
  Copyright (C) 2026 MoTeam

  This file is derived from Arco Design Vue (https://arco.design/).
  Original code licensed under the MIT License.

  MIT License full text will be added here
-->
<script setup lang="ts">
/**
 * 弹出触发器组件（参考 Arco Design Trigger 的定位/箭头设计思路，API 为项目自定义）
 *
 * 能力：
 * - 4 种触发方式：hover（悬浮，可延迟）/ click（点击切换）/ focus（聚焦）/ contextMenu（右键）
 * - 12 个弹出位置：top/tl/tr、bottom/bl/br、left/lt/lb、right/rt/rb
 * - 弹出层 Teleport 到 popupContainer（默认 body），fixed 定位 + 视口边界钳制
 * - 可选箭头（showArrow），自动跟随触发元素中心
 * - 受控（v-model:visible）/ 非受控（defaultVisible）双模式
 *
 * 用法：
 * <Trigger v-model:visible="show" position="bottom" trigger="click">
 *   <Button>点我</Button>
 *   <template #content>
 *     <div class="...">弹层内容</div>
 *   </template>
 * </Trigger>
 */
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { onClickOutside } from '@/utils/click-outside'

interface Props {
  /** 受控显示状态（v-model:visible） */
  visible?: boolean
  /** 非受控初始显示状态 */
  defaultVisible?: boolean
  /** 触发方式 */
  trigger?: 'hover' | 'click' | 'focus' | 'contextMenu'
  /** 弹出位置（12 向） */
  position?: 'top' | 'tl' | 'tr' | 'bottom' | 'bl' | 'br' | 'left' | 'lt' | 'lb' | 'right' | 'rt' | 'rb'
  /** 弹出层与触发元素的间距（px） */
  offset?: number
  /** 是否禁用 */
  disabled?: boolean
  /** 是否显示箭头 */
  showArrow?: boolean
  /** hover 移出触发元素后，是否允许移入弹出层保持显示 */
  hoverStay?: boolean
  /** hover 显示的延迟（ms） */
  showDelay?: number
  /** hover 隐藏的延迟（ms） */
  hideDelay?: number
  /** 内容区自定义 class（外观样式由调用方控制） */
  contentClass?: string
  /** 弹出层附加内联样式 */
  popupStyle?: Record<string, string>
  /** 挂载容器（选择器或元素），默认 body */
  popupContainer?: string | HTMLElement
}

const props = withDefaults(defineProps<Props>(), {
  defaultVisible: false,
  trigger: 'hover',
  position: 'bottom',
  offset: 6,
  disabled: false,
  showArrow: false,
  hoverStay: true,
  showDelay: 100,
  hideDelay: 100,
  contentClass: '',
  popupStyle: undefined,
  popupContainer: 'body',
})

const emit = defineEmits<{
  'update:visible': [visible: boolean]
  visibleChange: [visible: boolean]
  show: []
  hide: []
}>()

const localVisible = ref(props.defaultVisible)
const visible = computed(() => props.visible ?? localVisible.value)
const triggerRef = ref<HTMLElement | null>(null)
const popupRef = ref<HTMLElement | null>(null)
const positionStyle = ref<Record<string, string>>({})
const arrowStyle = ref<Record<string, string>>({})
const placement = ref(props.position)

let enterTimer: number | undefined
let leaveTimer: number | undefined

function clearTimers() {
  window.clearTimeout(enterTimer)
  window.clearTimeout(leaveTimer)
}

function apply(v: boolean) {
  if (v === visible.value) return
  localVisible.value = v
  emit('update:visible', v)
  emit('visibleChange', v)
  if (v) nextTick(updatePopup)
}

function setVisible(v: boolean, delay = 0) {
  if (props.disabled) return
  clearTimers()
  if (delay) {
    const timer = window.setTimeout(() => apply(v), delay)
    if (v) enterTimer = timer
    else leaveTimer = timer
  } else {
    apply(v)
  }
}

/** 解析 12 向位置为 主轴（axis）+ 交叉轴对齐（align） */
function parsePosition(pos: string) {
  let axis: 'top' | 'bottom' | 'left' | 'right'
  if (['top', 'tl', 'tr'].includes(pos)) axis = 'top'
  else if (['bottom', 'bl', 'br'].includes(pos)) axis = 'bottom'
  else if (['left', 'lt', 'lb'].includes(pos)) axis = 'left'
  else axis = 'right'
  const second = pos.slice(1)
  const align = second === 'l' || second === 't' ? 'start' : second === 'r' || second === 'b' ? 'end' : 'center'
  return { axis, align }
}

function updatePopup() {
  const t = triggerRef.value
  const p = popupRef.value
  if (!t || !p) return
  const tr = t.getBoundingClientRect()
  const pw = p.offsetWidth
  const ph = p.offsetHeight
  const gap = props.offset
  const { axis, align } = parsePosition(placement.value)
  const vw = window.innerWidth
  const vh = window.innerHeight
  let top = 0
  let left = 0

  if (axis === 'top') {
    top = tr.top - ph - gap
    if (align === 'start') left = tr.left
    else if (align === 'end') left = tr.right - pw
    else left = tr.left + tr.width / 2 - pw / 2
  } else if (axis === 'bottom') {
    top = tr.bottom + gap
    if (align === 'start') left = tr.left
    else if (align === 'end') left = tr.right - pw
    else left = tr.left + tr.width / 2 - pw / 2
  } else if (axis === 'left') {
    left = tr.left - pw - gap
    if (align === 'start') top = tr.top
    else if (align === 'end') top = tr.bottom - ph
    else top = tr.top + tr.height / 2 - ph / 2
  } else {
    left = tr.right + gap
    if (align === 'start') top = tr.top
    else if (align === 'end') top = tr.bottom - ph
    else top = tr.top + tr.height / 2 - ph / 2
  }

  // 视口边界钳制（保留 8px 边距）
  left = Math.min(Math.max(left, 8), Math.max(8, vw - pw - 8))
  top = Math.min(Math.max(top, 8), Math.max(8, vh - ph - 8))

  positionStyle.value = { position: 'fixed', top: `${top}px`, left: `${left}px`, zIndex: '1000', ...props.popupStyle }

  if (props.showArrow) {
    const size = 8
    if (axis === 'top' || axis === 'bottom') {
      const x = Math.min(Math.max(tr.left + tr.width / 2 - left, size + 4), pw - size - 4)
      arrowStyle.value = { left: `${x}px`, ...(axis === 'top' ? { bottom: '0' } : { top: '0' }) }
    } else {
      const y = Math.min(Math.max(tr.top + tr.height / 2 - top, size + 4), ph - size - 4)
      arrowStyle.value = { top: `${y}px`, ...(axis === 'left' ? { right: '0' } : { left: '0' }) }
    }
  }
}

// ===== 触发事件 =====
function onMouseEnter() {
  if (props.disabled || props.trigger !== 'hover') return
  setVisible(true, props.showDelay)
}
function onMouseLeave() {
  if (props.disabled || props.trigger !== 'hover') return
  setVisible(false, props.hideDelay)
}
function onPopupEnter() {
  if (props.hoverStay && props.trigger === 'hover') clearTimers()
}
function onPopupLeave() {
  if (props.trigger === 'hover') setVisible(false, props.hideDelay)
}
function onClick() {
  if (props.disabled || props.trigger !== 'click') return
  setVisible(!visible.value)
}
function onFocusIn() {
  if (props.disabled || props.trigger !== 'focus') return
  setVisible(true)
}
function onFocusOut() {
  if (props.disabled || props.trigger !== 'focus') return
  setVisible(false)
}
function onContextMenu() {
  if (props.disabled || props.trigger !== 'contextMenu') return
  setVisible(!visible.value)
}

function onScrollOrResize() {
  if (visible.value) updatePopup()
}

onClickOutside(
  triggerRef,
  () => {
    if (['click', 'contextMenu', 'focus'].includes(props.trigger)) setVisible(false)
  },
  [popupRef],
)

watch(visible, (v) => {
  if (v) nextTick(updatePopup)
  else clearTimers()
})

onMounted(() => {
  window.addEventListener('resize', onScrollOrResize)
  window.addEventListener('scroll', onScrollOrResize, true)
  if (visible.value) nextTick(updatePopup)
})
onUnmounted(() => {
  clearTimers()
  window.removeEventListener('resize', onScrollOrResize)
  window.removeEventListener('scroll', onScrollOrResize, true)
})
</script>

<template>
  <span
    ref="triggerRef"
    class="trigger-wrap"
    @mouseenter="onMouseEnter"
    @mouseleave="onMouseLeave"
    @click="onClick"
    @focusin="onFocusIn"
    @focusout="onFocusOut"
    @contextmenu.prevent="onContextMenu"
  >
    <slot />
  </span>

  <teleport :to="popupContainer">
    <transition name="trigger-fade" @after-enter="emit('show')" @after-leave="emit('hide')">
      <div
        v-if="visible"
        ref="popupRef"
        class="trigger-popup"
        :data-placement="placement"
        :style="positionStyle"
        @mouseenter="onPopupEnter"
        @mouseleave="onPopupLeave"
      >
        <div class="trigger-content" :class="contentClass">
          <slot name="content" />
        </div>
        <span v-if="showArrow" class="trigger-arrow" :style="arrowStyle" />
      </div>
    </transition>
  </teleport>
</template>

<style scoped src="./Trigger.css"></style>
