<script setup lang="ts">
/**
 * 数据工具分类页
 *
 * 编排层，承载该分类下所有工具子组件：
 * - Java 版本管理
 * - 崩溃日志分析
 * - 版本 JSON 编辑
 * - NBT 数据查看
 * - 截图批量管理
 * - 启动器数据导出
 * - 资源包转换
 */
import JavaManager from '@/views/tools/data/JavaManager.vue'
import CrashAnalyzer from '@/views/tools/data/CrashAnalyzer.vue'
import VersionJsonEditor from '@/views/tools/data/VersionJsonEditor.vue'
import NbtViewer from '@/views/tools/data/NbtViewer.vue'
import ScreenshotManager from '@/views/tools/data/ScreenshotManager.vue'
import DataExporter from '@/views/tools/data/DataExporter.vue'
import ResourcePackConverter from '@/views/tools/data/ResourcePackConverter.vue'
</script>

<template>
  <div class="mx-auto max-w-3xl space-y-6">
    <div id="tool-java-manager" data-toc-card="tool-java-manager" data-toc-title="Java 管理">
      <JavaManager />
    </div>
    <div id="tool-crash-analyzer" data-toc-card="tool-crash-analyzer" data-toc-title="崩溃分析">
      <CrashAnalyzer />
    </div>
    <div id="tool-version-json-editor" data-toc-card="tool-version-json-editor" data-toc-title="版本 JSON">
      <VersionJsonEditor />
    </div>
    <div id="tool-nbt-viewer" data-toc-card="tool-nbt-viewer" data-toc-title="NBT 查看">
      <NbtViewer />
    </div>
    <div id="tool-screenshot-manager" data-toc-card="tool-screenshot-manager" data-toc-title="截图管理">
      <ScreenshotManager />
    </div>
    <div id="tool-data-exporter" data-toc-card="tool-data-exporter" data-toc-title="数据导出">
      <DataExporter />
    </div>
    <div id="tool-resourcepack-converter" data-toc-card="tool-resourcepack-converter" data-toc-title="资源包转换">
      <ResourcePackConverter />
    </div>
  </div>
</template>
