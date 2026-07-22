<script setup lang="ts">
/**
 * 存档管理分类页
 *
 * 编排层，承载该分类下所有工具子组件：
 * - 存档管理（备份/恢复/导出）
 */
import ArchiveManager from '@/views/tools/archive/ArchiveManager.vue'
</script>

<template>
  <div class="mx-auto max-w-3xl space-y-6">
    <div id="tool-archive-manager" data-toc-card="tool-archive-manager" data-toc-title="存档管理">
      <ArchiveManager />
    </div>
  </div>
</template>
