<!--
  MoLaunch - Minecraft Launcher
  Copyright (C) 2026 MoTeam

  This file is derived from Arco Design Vue (https://arco.design/).
  Original code licensed under the MIT License.

  MIT License full text will be added here
-->
<script setup lang="ts">
/**
 * 抽屉组件（参考 Arco Design Drawer 的滑入/遮罩设计思路，API 为项目自定义）
 *
 * 能力：
 * - 4 个滑出方向：left / right / top / bottom
 * - 受控（v-model:visible）/ 非受控（defaultVisible）双模式
 * - 遮罩（mask）、点击遮罩关闭（maskClosable）、ESC 关闭（escToClose）
 * - header / title / footer 插槽，默认自带标题 + 关闭按钮
 * - 关闭动画结束后卸载节点，支持 unmountOnClose
 *
 * 用法：
 * <Drawer v-model:visible="visible" title="设置" placement="right" width="380">
 *   内容
 *   <template #footer><Button @click="visible = false">关闭</Button></template>
 * </Drawer>
 */
import { ref, computed, watch, onBeforeUnmount } from 'vue'
import { ArrowUturnLeftIcon, XMarkIcon } from '@heroicons/vue/24/outline'
import { onEscape } from '@/utils/click-outside'

interface Props {
  /** 受控显示状态（v-model:visible） */
  visible?: boolean
  /** 非受控初始显示状态 */
  defaultVisible?: boolean
  /** 标题（也可用 #title 插槽） */
  title?: string
  /** 滑出方向 */
  placement?: 'left' | 'right' | 'top' | 'bottom'
  /** 宽度（仅 left/right 生效） */
  width?: number | string
  /** 高度（仅 top/bottom 生效） */
  height?: number | string
  /** 是否显示关闭按钮 */
  closable?: boolean
  /** 是否显示遮罩 */
  mask?: boolean
  /** 点击遮罩是否关闭 */
  maskClosable?: boolean
  /** 按 ESC 是否关闭 */
  escToClose?: boolean
  /** 是否显示默认底部栏（占位，内容由 #footer 插槽提供） */
  footer?: boolean
  /** 关闭后是否卸载节点 */
  unmountOnClose?: boolean
  /** 关闭反悔期（毫秒）：默认关闭；由调用方按需传入（如 5000）后，面板滑出会保留节点、边缘显示可恢复 tab，期间点击可重新打开，超时后真正卸载；配合 unmountOnClose 则关闭即卸载 */
  undoMs?: number
  /** 就地渲染：不 teleport 到 body，absolute 铺满最近定位祖先（nav 下方内容区） */
  renderInPlace?: boolean
  /** 挂载容器（选择器或元素），默认 body；render-in-place 时建议传内容区容器（如 #app-content） */
  popupContainer?: string | HTMLElement
}

const props = withDefaults(defineProps<Props>(), {
  defaultVisible: false,
  title: '',
  placement: 'right',
  width: 280,
  height: 280,
  closable: true,
  mask: true,
  maskClosable: true,
  escToClose: true,
  footer: false,
  unmountOnClose: false,
  undoMs: 0,
  popupContainer: 'body',
})

const emit = defineEmits<{
  'update:visible': [visible: boolean]
  visibleChange: [visible: boolean]
  open: []
  close: []
}>()

const localVisible = ref(props.defaultVisible)
const visible = computed(() => props.visible ?? localVisible.value)
/** 保持节点存活以播放关闭动画（关闭 / 反悔期结束才卸载） */
const mounted = ref(visible.value)
/** 是否处于关闭反悔期（面板已滑出、边缘显示可恢复 tab） */
const undoVisible = ref(false)
/** 反悔期剩余秒数（用于气泡倒计时展示） */
const undoSeconds = ref(0)
let undoTimer: ReturnType<typeof setTimeout> | null = null
let undoCountdown: ReturnType<typeof setInterval> | null = null

function clearUndoTimer() {
  if (undoTimer) {
    clearTimeout(undoTimer)
    undoTimer = null
  }
  if (undoCountdown) {
    clearInterval(undoCountdown)
    undoCountdown = null
  }
}
onBeforeUnmount(clearUndoTimer)

/** 关闭后是否进入反悔期（unmountOnClose 或关闭恢复期设为 0 时直接卸载） */
function canUndo() {
  return !props.unmountOnClose && props.undoMs > 0
}

/** 面板滑出动画结束：进入反悔期，倒计时结束后真正卸载 */
function startUndo() {
  const totalMs = props.undoMs
  const startedAt = Date.now()
  const remaining = () => Math.max(0, Math.ceil((totalMs - (Date.now() - startedAt)) / 1000))
  undoSeconds.value = remaining()
  undoVisible.value = true
  clearUndoTimer()
  undoTimer = setTimeout(() => {
    undoTimer = null
    finishClose()
  }, totalMs)
  undoCountdown = setInterval(() => {
    undoSeconds.value = remaining()
  }, 1000)
}

/** 真正关闭：若恢复 tab 正在显示，先播完其消失动画再卸载，否则直接卸载 */
function finishClose() {
  if (undoVisible.value) {
    undoVisible.value = false
    return
  }
  finalizeClose()
}

/** 恢复 tab 消失动画结束：未重新打开时才真正卸载 */
function onUndoTabLeave() {
  if (!visible.value) finalizeClose()
}

/** 卸载节点并广播 close */
function finalizeClose() {
  mounted.value = false
  emit('close')
}

/** 点击反悔 tab：立即重新打开，取消卸载倒计时 */
function reopen() {
  clearUndoTimer()
  undoVisible.value = false
  localVisible.value = true
  emit('update:visible', true)
  emit('visibleChange', true)
}

function close() {
  if (!visible.value) return
  localVisible.value = false
  emit('update:visible', false)
  emit('visibleChange', false)
}

/** 面板滑入动画结束（此时必然可见） */
function onEnter() {
  if (visible.value) emit('open')
}

/** 面板滑出动画结束（此时必然不可见）：进入反悔期或直接卸载 */
function onLeave() {
  if (visible.value) return
  if (canUndo()) {
    startUndo()
  } else {
    finishClose()
  }
}

watch(visible, (v) => {
  if (v) {
    clearUndoTimer()
    undoVisible.value = false
    mounted.value = true
  }
})

onEscape(() => {
  if (props.escToClose && visible.value) close()
})

/** render-in-place 严格归一化为布尔（防御无值 attribute 传空字符串的情况） */
const isInPlace = computed(() => props.renderInPlace === true)
/**
 * 挂载目标：
 * - 普通模式：teleport 到 popupContainer（默认 body）
 * - 就地模式：若调用方指定了内容区容器（如 #app-content）则 teleport 到该容器内 absolute 铺满，
 *   与 DragOverlay 同一挂载点、从布局上避开 nav；未指定则原地渲染（teleport 目标为 null）
 */
const teleportTarget = computed(() => {
  if (!isInPlace.value) return props.popupContainer
  return props.popupContainer === 'body' ? null : props.popupContainer
})

const panelStyle = computed(() => {
  const isVertical = props.placement === 'left' || props.placement === 'right'
  const size = isVertical ? props.width : props.height
  const value = typeof size === 'number' ? `${size}px` : size
  return isVertical ? { width: value } : { height: value }
})
</script>

<template>
  <teleport :to="teleportTarget">
    <div
      v-if="visible || mounted"
      v-show="visible || mounted"
      class="drawer-root"
      :class="[
        `drawer-placement-${placement}`,
        { 'drawer-root--maskless': !mask },
        { 'drawer-root--absolute': isInPlace },
        { 'drawer-root--undo': undoVisible },
      ]"
    >
      <transition name="drawer-fade" appear>
        <div v-if="mask" v-show="visible" class="drawer-mask" @click="maskClosable && close()" />
      </transition>

      <transition
        :name="`drawer-slide-${placement}`"
        appear
        @after-enter="onEnter"
        @after-leave="onLeave"
      >
        <div v-show="visible" class="drawer-panel" :style="panelStyle">
          <!-- 头部：有 title / 插槽 / 关闭按钮时渲染 -->
          <div v-if="$slots.header || $slots.title || title || closable" class="drawer-header">
            <slot name="header">
              <div v-if="$slots.title || title" class="drawer-title">
                <slot name="title">{{ title }}</slot>
              </div>
              <button v-if="closable" class="drawer-close" aria-label="关闭" @click="close">
                <XMarkIcon class="drawer-close-icon" />
              </button>
            </slot>
          </div>

          <!-- 内容 -->
          <div class="drawer-body">
            <slot />
          </div>

          <!-- 底部 -->
          <div v-if="$slots.footer || footer" class="drawer-footer">
            <slot name="footer" />
          </div>
        </div>
      </transition>

      <!-- 关闭反悔期：面板滑出后边缘保留的小 tab（带剩余秒数气泡），点击可重新打开 -->
      <transition name="drawer-undo" @after-leave="onUndoTabLeave">
        <div v-if="undoVisible" class="drawer-undo" :class="`drawer-undo--${placement}`">
          <span class="drawer-undo-bubble">还有 {{ undoSeconds }} 秒后关闭</span>
          <button
            class="drawer-undo-tab"
            aria-label="重新打开"
            @click="reopen"
          >
            <ArrowUturnLeftIcon class="drawer-undo-tab-icon" />
          </button>
        </div>
      </transition>
    </div>
  </teleport>
</template>

<style scoped src="./Drawer.css"></style>
