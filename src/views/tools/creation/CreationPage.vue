<script setup lang="ts">
/**
 * 创作工具分类页
 *
 * 编排层，承载该分类下所有工具子组件：
 * - 渐变文字生成器
 * - 合成配方生成器
 */
import GradientTextPage from './GradientTextPage.vue'
import RecipeGeneratorPage from './recipe-generator/RecipeGeneratorPage.vue'
</script>

<template>
  <div class="space-y-6">
    <div id="tool-gradient-text" data-toc-card="tool-gradient-text" data-toc-title="渐变文字">
      <GradientTextPage />
    </div>
    <div
      id="tool-recipe-generator"
      data-toc-card="tool-recipe-generator"
      data-toc-title="合成配方"
    >
      <RecipeGeneratorPage />
    </div>
  </div>
</template>
