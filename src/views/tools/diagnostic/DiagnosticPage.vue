<script setup lang="ts">
/**
 * 诊断工具分类页
 *
 * 编排层，承载该分类下所有工具子组件：
 * - 崩溃日志分析
 * - 版本 JSON 编辑
 * - NBT 数据查看
 */
import CrashAnalyzer from '@/views/tools/data/CrashAnalyzer.vue'
import VersionJsonEditor from '@/views/tools/data/VersionJsonEditor.vue'
import NbtViewer from '@/views/tools/data/NbtViewer.vue'
</script>

<template>
  <div class="mx-auto max-w-3xl space-y-6">
    <div id="tool-crash-analyzer" data-toc-card="tool-crash-analyzer" data-toc-title="崩溃分析">
      <CrashAnalyzer />
    </div>
    <div id="tool-version-json-editor" data-toc-card="tool-version-json-editor" data-toc-title="版本 JSON">
      <VersionJsonEditor />
    </div>
    <div id="tool-nbt-viewer" data-toc-card="tool-nbt-viewer" data-toc-title="NBT 查看">
      <NbtViewer />
    </div>
  </div>
</template>
