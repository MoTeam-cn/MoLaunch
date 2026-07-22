<script setup lang="ts">
/**
 * Mod 工具分类页
 *
 * 编排层，承载该分类下所有工具子组件：
 * - Mod 依赖检测
 * - Mod 文件去重
 */
import ModDependencyChecker from '@/views/tools/mod-tools/ModDependencyChecker.vue'
import ModDedupScanner from '@/views/tools/mod-tools/ModDedupScanner.vue'
</script>

<template>
  <div class="mx-auto max-w-3xl space-y-6">
    <div id="tool-mod-dependency-checker" data-toc-card="tool-mod-dependency-checker" data-toc-title="依赖检测">
      <ModDependencyChecker />
    </div>
    <div id="tool-mod-dedup-scanner" data-toc-card="tool-mod-dedup-scanner" data-toc-title="Mod 去重">
      <ModDedupScanner />
    </div>
  </div>
</template>
