<script setup lang="ts">
/**
 * 网络工具分类页
 *
 * 编排层，承载该分类下所有工具子组件：
 * - 服务器状态检测
 * - 网络延迟测试
 */
import ServerPinger from '@/views/tools/network/ServerPinger.vue'
import NetworkLatencyTester from '@/views/tools/network/NetworkLatencyTester.vue'
</script>

<template>
  <div class="mx-auto max-w-3xl space-y-6">
    <div id="tool-server-pinger" data-toc-card="tool-server-pinger" data-toc-title="服务器检测">
      <ServerPinger />
    </div>
    <div id="tool-network-latency-tester" data-toc-card="tool-network-latency-tester" data-toc-title="延迟测试">
      <NetworkLatencyTester />
    </div>
  </div>
</template>
