<script setup lang="ts">
/**
 * 内置插件：快速统计
 *
 * 在主页右侧内容区显示已安装版本数量、账号数量等统计信息。
 * 作为插件系统的示例，展示 homePanel 能力。
 */

import { ref, onMounted } from 'vue'
import { pluginSdk } from '@/plugins/sdk'
import { safeCall } from '@/utils/async'

const versionCount = ref(0)
const loading = ref(true)

onMounted(async () => {
  const versions = await safeCall(() => pluginSdk.listInstalledVersions(), '[QuickStats] load installed versions')
  if (versions) versionCount.value = versions.length
  loading.value = false
})
</script>

<template>
  <div class="flex h-full flex-col p-6">
    <h3 class="text-base font-semibold text-gray-900 mb-4">快速统计</h3>
    <div v-if="loading" class="text-sm text-gray-500">加载中...</div>
    <div v-else class="space-y-3">
      <div class="flex items-center justify-between rounded-md border border-gray-200 px-4 py-3">
        <span class="text-sm text-gray-600">已安装版本</span>
        <span class="text-lg font-semibold text-primary-600">{{ versionCount }}</span>
      </div>
      <p class="text-xs text-gray-400">这是一个内置示例插件，可在「设置 → 插件」中关闭。</p>
    </div>
  </div>
</template>
