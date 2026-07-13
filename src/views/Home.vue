<script setup lang="ts">
/**
 * 首页（左右双栏布局，参考 PCL2）
 * 左侧：账号选择 + 版本选择 + 启动按钮
 * 右侧：启动进度 + 状态总览 + 快速操作
 */

import { onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useSdkStore } from '@/stores/sdk'
import { useVersionStore } from '@/stores/version'
import { useJavaStore } from '@/stores/java'
import * as tauri from '@/utils/tauri'
import LaunchPanel from '@/components/home/LaunchPanel.vue'
import LaunchLog from '@/components/home/LaunchLog.vue'

const authStore = useAuthStore()
const sdkStore = useSdkStore()
const versionStore = useVersionStore()
const javaStore = useJavaStore()

onMounted(async () => {
  // 并行恢复会话和初始化
  await Promise.all([
    authStore.restoreSession(),
    sdkStore.fetchPlatformInfo(),
    javaStore.detectJava(),
  ])

  if (sdkStore.isReady) {
    await versionStore.fetchVersions()
    await versionStore.checkRunningGame()
  }

  // 先尝试恢复上次选中的版本（会校验版本是否仍然存在）
  await versionStore.restoreSelectedVersion()

  // 如果仍未选中，自动选中第一个已安装版本
  if (!versionStore.selectedVersion) {
    try {
      const installed = await tauri.listInstalledVersionsWithType()
      if (installed.length > 0) {
        versionStore.selectedVersion = installed[0].id
      }
    } catch {
      // 忽略：用户可去版本选择页手动选
    }
  }
})
</script>

<template>
  <div class="flex h-full gap-2 p-2">
    <!-- 左侧启动面板（固定宽度 320px） -->
    <aside class="flex w-80 flex-none flex-col overflow-hidden rounded-lg border border-gray-200 bg-white shadow-sm">
      <LaunchPanel />
    </aside>

    <!-- 右侧内容区（弹性宽度） -->
    <main class="flex-1 overflow-hidden rounded-lg border border-gray-200 bg-white shadow-sm">
      <LaunchLog />
    </main>
  </div>
</template>
