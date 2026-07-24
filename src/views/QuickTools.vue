<script setup lang="ts">
/**
 * 便捷工具子组件（由 Tools.vue 作为侧边栏内容区承载）
 *
 * 本文件为编排层，各工具子组件已拆分到 ./quick-tools/ 目录：
 * - 清理游戏垃圾 → CleanupTool.vue
 * - 内存优化 → MemoryOptimizer.vue
 * - 启动器数据导出 → DataExporter.vue（从原数据工具迁入，属实用工具）
 *
 * 其余工具已迁移至工具页侧边栏的其他分类：
 * 存档管理 / Mod 工具 / 网络工具 / 计算工具 / Java 管理 / 诊断工具 / 游戏资源
 */
import CleanupTool from './quick-tools/CleanupTool.vue'
import MemoryOptimizer from './quick-tools/MemoryOptimizer.vue'
import DataExporter from '@/views/tools/data/DataExporter.vue'
</script>

<template>
  <div class="mx-auto max-w-3xl space-y-6">
    <!-- 清理游戏垃圾 -->
    <CleanupTool />

    <!-- 内存优化 -->
    <MemoryOptimizer />

    <!-- 启动器数据导出 -->
    <div id="tool-data-exporter" data-toc-card="tool-data-exporter" data-toc-title="数据导出">
      <DataExporter />
    </div>
  </div>
</template>
