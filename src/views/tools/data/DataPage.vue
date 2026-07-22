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
    <JavaManager />
    <CrashAnalyzer />
    <VersionJsonEditor />
    <NbtViewer />
    <ScreenshotManager />
    <DataExporter />
    <ResourcePackConverter />
  </div>
</template>
