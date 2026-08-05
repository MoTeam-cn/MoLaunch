<!--
  MoLaunch - Minecraft Launcher
  Copyright (C) 2026 MoTeam

  This file is derived from Arco Design Vue (https://arco.design/).
  Original code licensed under the MIT License.

  MIT License full text will be added here
-->
<script setup lang="ts">
/**
 * 标签组件（复刻 Arco Design Vue Tag）
 *
 * 颜色：内置 13 种预设（red/orangered/orange/gold/lime/green/cyan/blue/
 *       arcoblue/purple/pinkpurple/magenta/gray），浅色底 + 深色字；
 *       也可传自定义 hex/rgb 字符串（背景即色值，白字，与 Arco 一致）
 * 尺寸：small(20px) / medium(24px) / large(28px)
 *
 * 用法：
 * <Tag color="red">修复</Tag>
 * <Tag size="small" :closable="true" @close="...">
 *   <template #icon><.../></template>
 *   文档
 * </Tag>
 */
import { computed, useSlots } from 'vue'
import { XMarkIcon } from '@heroicons/vue/24/outline'

/** 预设颜色（复刻 Arco Design Tag 的 13 色 + primary 主题色） */
type TagColor =
  | 'red'
  | 'orangered'
  | 'orange'
  | 'gold'
  | 'lime'
  | 'green'
  | 'cyan'
  | 'blue'
  | 'arcoblue'
  | 'purple'
  | 'pinkpurple'
  | 'magenta'
  | 'gray'
  | 'primary'

const PRESET_COLORS: TagColor[] = [
  'red', 'orangered', 'orange', 'gold', 'lime', 'green',
  'cyan', 'blue', 'arcoblue', 'purple', 'pinkpurple', 'magenta', 'gray', 'primary',
]

interface Props {
  /** 标签颜色：13 种预设色之一，或自定义 hex/rgb 字符串 */
  color?: TagColor | string
  /** 标签尺寸 */
  size?: 'small' | 'medium' | 'large'
  /** 是否显示关闭按钮 */
  closable?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  color: 'gray',
  size: 'medium',
  closable: false,
})

defineEmits<{ close: [e: MouseEvent] }>()

const slots = useSlots()
/** 是否存在 icon 槽位内容（决定是否渲染图标容器） */
const hasIcon = computed(() => !!slots.icon)

/**
 * 颜色类名使用静态映射而非动态拼接（`tag-${color}`）。
 *
 * 关键：Tailwind 的 purge 扫描器只能静态识别源码中出现的完整类名字符串，
 * 无法推断模板字符串 `tag-${color}` 会展开为哪些具体类名。若使用动态拼接，
 * main.css 中 @layer components 定义的自定义类（tag-red / tag-green 等）
 * 会因"未被检测到使用"而被整体 purge，导致标签丢失预设背景/文字色。
 */
const colorClass = computed(() => {
  switch (props.color) {
    case 'red': return 'tag-red'
    case 'orangered': return 'tag-orangered'
    case 'orange': return 'tag-orange'
    case 'gold': return 'tag-gold'
    case 'lime': return 'tag-lime'
    case 'green': return 'tag-green'
    case 'cyan': return 'tag-cyan'
    case 'blue': return 'tag-blue'
    case 'arcoblue': return 'tag-arcoblue'
    case 'purple': return 'tag-purple'
    case 'pinkpurple': return 'tag-pinkpurple'
    case 'magenta': return 'tag-magenta'
    case 'gray': return 'tag-gray'
    case 'primary': return 'tag-primary'
    default: return ''
  }
})

/** 自定义颜色（非预设）：背景即色值，白字（与 Arco 行为一致） */
const customStyle = computed(() => {
  if (PRESET_COLORS.includes(props.color as TagColor)) return undefined
  return { backgroundColor: props.color, borderColor: props.color, color: '#ffffff' }
})

const sizeClass = computed(() => {
  switch (props.size) {
    case 'small': return 'tag-size-small'
    case 'medium': return 'tag-size-medium'
    case 'large': return 'tag-size-large'
    default: return 'tag-size-medium'
  }
})
</script>

<template>
  <span
    :class="['tag', sizeClass, colorClass]"
    :style="customStyle"
  >
    <span v-if="hasIcon" class="tag-icon">
      <slot name="icon" />
    </span>
    <slot />
    <span
      v-if="closable"
      class="tag-close-btn"
      role="button"
      tabindex="0"
      @click.stop="$emit('close', $event)"
    >
      <XMarkIcon class="tag-close-icon" />
    </span>
  </span>
</template>
