<script setup lang="ts">
/**
 * 计算工具分类页
 *
 * 编排层，承载该分类下所有工具子组件：
 * - 坐标距离计算
 * - 游戏内调色板
 */
import CoordCalculator from '@/views/tools/calc/CoordCalculator.vue'
import ColorPalette from '@/views/tools/calc/ColorPalette.vue'
</script>

<template>
  <div class="mx-auto max-w-3xl space-y-6">
    <CoordCalculator />
    <ColorPalette />
  </div>
</template>
