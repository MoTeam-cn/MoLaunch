<script setup lang="ts">
/**
 * Java 管理分类页
 *
 * Java 运行时是启动游戏的核心依赖，作为重要工具单独一栏。
 * 三个区块：Java 运行时列表 / 已安装版本 Java 环境检测 / Java 下载器。
 */
import JavaManager from '@/views/tools/data/JavaManager.vue'
import JavaEnvCheck from '@/views/tools/java/JavaEnvCheck.vue'
import JavaDownloader from '@/views/tools/java/JavaDownloader.vue'
</script>

<template>
  <div class="mx-auto max-w-3xl space-y-6">
    <div id="tool-java-downloader" data-toc-card="tool-java-downloader" data-toc-title="Java 下载器">
      <JavaDownloader />
    </div>
    <div
      id="tool-java-env-check"
      data-toc-card="tool-java-env-check"
      data-toc-title="已安装版本 Java 环境检测"
    >
      <JavaEnvCheck />
    </div>
    <div id="tool-java-manager" data-toc-card="tool-java-manager" data-toc-title="Java 运行时列表">
      <JavaManager />
    </div>
  </div>
</template>