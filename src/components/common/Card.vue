<!--
  MoLaunch - Minecraft Launcher
  Copyright (C) 2026 MoTeam

  This file is derived from Arco Design Vue (https://arco.design/).
  Original code licensed under the MIT License.

  MIT License full text will be added here
-->
<script setup lang="ts">
/**
 * 卡片组件
 *
 *
 * Props：
 * - bordered：是否有边框（默认 true）
 * - hoverable：是否悬浮效果（默认 false）
 * - size：卡片尺寸 'medium' | 'small'（默认 'medium'）
 * - title：卡片标题
 * - extra：右上角操作区文本
 * - headerStyle / bodyStyle：自定义样式
 *
 * Slots：
 * - default：卡片内容
 * - title：自定义标题
 * - extra：自定义右上角操作区
 * - cover：卡片封面
 * - actions：底部操作组
 */
import type { CSSProperties } from 'vue'

interface Props {
  bordered?: boolean
  hoverable?: boolean
  size?: 'medium' | 'small'
  title?: string
  extra?: string
  headerStyle?: CSSProperties
  bodyStyle?: CSSProperties
}

const props = withDefaults(defineProps<Props>(), {
  bordered: true,
  hoverable: false,
  size: 'medium',
  title: '',
  extra: '',
  headerStyle: () => ({}),
  bodyStyle: () => ({}),
})
</script>

<template>
  <div
    class="rounded-lg bg-white transition-shadow"
    :class="[
      props.bordered ? 'border border-gray-200' : 'border-none',
      props.hoverable ? 'hover:shadow-md' : '',
      props.size === 'small' ? 'text-sm' : '',
    ]"
  >
    <!-- 头部（标题 + 操作区） -->
    <div
      v-if="$slots.title || props.title || $slots.extra || props.extra"
      class="flex items-center justify-between border-b border-gray-100 px-4 py-3"
      :style="props.headerStyle"
    >
      <div v-if="$slots.title || props.title" class="text-sm font-semibold text-gray-800">
        <slot name="title">{{ props.title }}</slot>
      </div>
      <div v-if="$slots.extra || props.extra" class="text-xs text-gray-500">
        <slot name="extra">{{ props.extra }}</slot>
      </div>
    </div>

    <!-- 封面 -->
    <div v-if="$slots.cover">
      <slot name="cover" />
    </div>

    <!-- 内容区 -->
    <div
      class="px-4 py-3"
      :class="props.size === 'small' ? 'px-3 py-2' : ''"
      :style="props.bodyStyle"
    >
      <slot />
    </div>

    <!-- 底部操作组 -->
    <div v-if="$slots.actions" class="border-t border-gray-100 px-4 py-2">
      <div class="flex items-center justify-end gap-3">
        <slot name="actions" />
      </div>
    </div>
  </div>
</template>
