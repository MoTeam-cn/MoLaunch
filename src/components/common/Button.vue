<!--
  MoLaunch - Minecraft Launcher
  Copyright (C) 2026 MoTeam

  This file is derived from Arco Design Vue (https://arco.design/).
  Original code licensed under the MIT License.

  MIT License full text will be added here
-->
<script setup lang="ts">
/**
 * 按钮组件
 *
 * 类型：
 * - primary：蓝底白字（默认）
 * - secondary：灰底深字
 * - outline：透明底蓝边框
 * - ghost：透明底，hover 浅灰
 * - text：文本按钮
 *
 * 尺寸：mini(24px) / small(28px) / default(32px) / large(36px)
 *
 * 用法：
 * <Button type="primary" @click="...">确定</Button>
 * <Button type="secondary" :loading="true">加载中</Button>
 * <Button type="outline" size="small">
 *   <template #icon><PlusIcon /></template>
 *   添加
 * </Button>
 */
import { computed, useSlots } from 'vue'

interface Props {
  /** 按钮类型 */
  type?: 'primary' | 'secondary' | 'outline' | 'ghost' | 'text'
  /** 按钮尺寸 */
  size?: 'mini' | 'small' | 'default' | 'large'
  /** 是否禁用 */
  disabled?: boolean
  /** 是否加载中 */
  loading?: boolean
  /** 是否撑满父容器宽度 */
  long?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  type: 'primary',
  size: 'default',
  disabled: false,
  loading: false,
  long: false,
})

defineEmits<{ click: [e: MouseEvent] }>()

const slots = useSlots()
/**
 * 检测 default slot 是否有文本内容（用于区分图标-only 按钮）
 *
 * 图标-only 按钮（如 <Button><HomeIcon /></Button>）的 default slot 只有组件 vnode，
 * 没有文本 vnode。此时 svg 的 margin-right 会让图标在 flex 居中时向左偏移，
 * 需要去除 margin 让图标真正居中。
 *
 * 检测逻辑：default slot 的 vnode 中是否存在文本类型（type 为 Symbol 或 children 为非空字符串）。
 */
const hasText = computed(() => {
  const vnodes = slots.default?.() ?? []
  return vnodes.some(v => {
    if (typeof v.children === 'string' && v.children.trim()) return true
    if (Array.isArray(v.children)) {
      return v.children.some(c => typeof c === 'string' && c.trim())
    }
    return false
  })
})

/**
 * 类名使用静态映射而非动态拼接（`btn-${type}` / `btn-size-${size}`）。
 *
 * 关键：Tailwind 的 purge 扫描器只能静态识别源码中出现的完整类名字符串，
 * 无法推断模板字符串 `btn-${type}` 会展开为哪些具体类名。若使用动态拼接，
 * main.css 中 @layer components 定义的自定义类（btn-outline / btn-secondary
 * / btn-ghost / btn-text）会因"未被检测到使用"而被整体 purge，
 * 导致这些类型按钮丢失 border/background/color，只剩 scoped 尺寸样式。
 */
const typeClass = computed(() => {
  switch (props.type) {
    case 'primary': return 'btn-primary'
    case 'secondary': return 'btn-secondary'
    case 'outline': return 'btn-outline'
    case 'ghost': return 'btn-ghost'
    case 'text': return 'btn-text'
    default: return 'btn-primary'
  }
})

const sizeClass = computed(() => {
  switch (props.size) {
    case 'mini': return 'btn-size-mini'
    case 'small': return 'btn-size-small'
    case 'default': return 'btn-size-default'
    case 'large': return 'btn-size-large'
    default: return 'btn-size-default'
  }
})
</script>

<template>
  <button
    :class="[
      'btn',
      typeClass,
      sizeClass,
      { 'btn-long': long, 'btn-loading': loading, 'btn-icon-only': !hasText },
    ]"
    :disabled="disabled || loading"
    @click="$emit('click', $event)"
  >
    <!-- 加载图标 -->
    <svg v-if="loading" class="btn-spinner" viewBox="0 0 1024 1024" fill="currentColor">
      <path d="M512 64a448 448 0 1 0 448 448 32 32 0 0 0-64 0 384 384 0 1 1-384-384 32 32 0 0 0 0-64z" />
    </svg>
    <!-- 前置图标 -->
    <slot v-else name="icon" />
    <!-- 文字内容 -->
    <slot />
  </button>
</template>

<style scoped>
/* 按钮基础样式已在全局 main.css 中定义（.btn / .btn-primary 等）
   这里只补充尺寸和特殊状态 */

/* 尺寸 */
.btn-size-mini {
  height: 24px;
  padding: 0 11px;
  font-size: 12px;
  /* line-height 收紧到 1，让文本行高接近 font-size，
     与 14px icon 在 flex 居中后视觉中心对齐（默认 1.5715 会让文本偏高） */
  line-height: 1;
}
.btn-size-small {
  height: 28px;
  padding: 0 15px;
  font-size: 14px;
}
.btn-size-default {
  height: 32px;
  padding: 0 15px;
  font-size: 14px;
}
.btn-size-large {
  height: 36px;
  padding: 0 19px;
  font-size: 14px;
}

/* 撑满宽度 */
.btn-long {
  display: flex;
  width: 100%;
}

/* 图标与文字间距（有图标时） */
.btn:not(.btn-size-mini) > :deep(svg:not(.btn-spinner)) {
  margin-right: 8px;
}
.btn-size-mini > :deep(svg:not(.btn-spinner)) {
  margin-right: 8px;
}
.btn-size-small > :deep(svg:not(.btn-spinner)) {
  margin-right: 6px;
}

/* 没有 slot 文字内容时（纯图标按钮），去掉 margin
   原 :empty 选择器因 Vue slot 注释节点不匹配，改用 btn-icon-only class
   （由 useSlots 检测 default slot 是否有文本内容动态添加）*/
.btn-icon-only > :deep(svg:not(.btn-spinner)) {
  margin-right: 0;
}

/* 加载动画 */
.btn-spinner {
  width: 14px;
  height: 14px;
  margin-right: 8px;
  animation: btn-spin 0.8s linear infinite;
}
.btn-icon-only .btn-spinner {
  margin-right: 0;
}

@keyframes btn-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* loading 状态下隐藏图标槽位 */
.btn-loading > :deep(slots[name='icon']) {
  display: none;
}
</style>
