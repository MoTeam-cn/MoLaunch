<script setup lang="ts">
/**
 * 带动画的折叠内容容器组件
 *
 * 只负责"内容区"的展开/收起动画，标题栏由调用方自行渲染（保持布局灵活）。
 * 使用 `grid-template-rows: 0fr → 1fr` 过渡实现平滑高度动画——
 * 无需测量内容高度、纯 CSS 完成，与项目内 MoLaunchIntro / CollapsibleCard /
 * CleanupGroupList 等折叠方案一致。
 *
 * 注意：折叠时内容仍在 DOM 中（不做懒挂载），适用于内容量可控的场景。
 */
defineProps<{
  /** 是否展开 */
  open: boolean
}>()
</script>

<template>
  <div
    class="grid transition-all duration-300 ease-in-out"
    :class="open ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
  >
    <div class="overflow-hidden">
      <slot />
    </div>
  </div>
</template>
