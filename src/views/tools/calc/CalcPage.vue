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
    <div id="tool-coord-calculator" data-toc-card="tool-coord-calculator" data-toc-title="坐标计算">
      <CoordCalculator />
    </div>
    <div id="tool-color-palette" data-toc-card="tool-color-palette" data-toc-title="调色板">
      <ColorPalette />
    </div>
  </div>
</template>
