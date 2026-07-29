<script setup lang="ts">
/**
 * 开发者 - 日志子页签
 *
 * 包含两个自包含子组件：
 * - HttpLogViewer：联机 API 调用追踪（HTTP 请求日志，默认收起，展开才加载）
 * - LogViewer：应用运行日志（默认收起，展开才加载）
 *
 * 两者均为 CollapsibleCard 包裹的独立组件，内部自行管理数据加载。
 */
import LogViewer from '@/components/settings/LogViewer.vue'
import HttpLogViewer from '@/components/settings/HttpLogViewer.vue'

defineProps<{
  /** 日志目录路径（LogViewer「打开目录」按钮使用） */
  logsDir?: string
}>()
</script>

<template>
  <div class="space-y-6">
    <!-- HTTP 请求日志（联机 API 调用追踪，默认收起，展开才加载） -->
    <HttpLogViewer />

    <!-- 日志查看（自包含子组件，传入日志目录用于「打开目录」按钮） -->
    <LogViewer :logs-dir="logsDir" />
  </div>
</template>
