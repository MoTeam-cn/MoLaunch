<script setup lang="ts">
/**
 * 诊断工具分类页
 *
 * 编排层，承载该分类下所有工具子组件：
 * - 版本 JSON 编辑
 * - NBT 数据查看
 *
 * 注：崩溃日志分析已迁移至「实验性」页面的日志分析分类（仅实验性页面可用）。
 */
import VersionJsonEditor from '@/views/tools/data/VersionJsonEditor.vue'
import NbtViewer from '@/views/tools/data/NbtViewer.vue'
</script>

<template>
  <div class="mx-auto max-w-3xl space-y-6">
    <div id="tool-version-json-editor" data-toc-card="tool-version-json-editor" data-toc-title="版本 JSON">
      <VersionJsonEditor />
    </div>
    <div id="tool-nbt-viewer" data-toc-card="tool-nbt-viewer" data-toc-title="NBT 查看">
      <NbtViewer />
    </div>
  </div>
</template>
