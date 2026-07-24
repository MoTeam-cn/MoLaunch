<script setup lang="ts">
/**
 * 游戏资源分类页
 *
 * 编排层，承载该分类下所有工具子组件：
 * - 截图批量管理
 * - 资源包转换
 */
import ScreenshotManager from '@/views/tools/data/ScreenshotManager.vue'
import ResourcePackConverter from '@/views/tools/data/ResourcePackConverter.vue'
</script>

<template>
  <div class="mx-auto max-w-3xl space-y-6">
    <div id="tool-screenshot-manager" data-toc-card="tool-screenshot-manager" data-toc-title="截图管理">
      <ScreenshotManager />
    </div>
    <div id="tool-resourcepack-converter" data-toc-card="tool-resourcepack-converter" data-toc-title="资源包转换">
      <ResourcePackConverter />
    </div>
  </div>
</template>
