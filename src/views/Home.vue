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
import { useVersionSettings } from '@/composables/useVersionSettings'
import * as tauri from '@/utils/tauri'
import LaunchPanel from '@/components/home/LaunchPanel.vue'
import LaunchLog from '@/components/home/LaunchLog.vue'

const authStore = useAuthStore()
const sdkStore = useSdkStore()
const versionStore = useVersionStore()
const javaStore = useJavaStore()
const { refreshInstalledVersionTypes } = useVersionSettings()

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

  // 一次性获取已安装版本列表，复用给后续三个逻辑（避免重复 IPC 调用）
  let installed: Awaited<ReturnType<typeof tauri.listInstalledVersionsWithType>> = []
  try {
    installed = await tauri.listInstalledVersionsWithType()
  } catch {
    // 忽略：用户可去版本选择页手动选
  }

  // 1. 刷新已安装版本类型映射缓存（主页 VersionSelector 依赖此缓存显示版本类型图标）
  await refreshInstalledVersionTypes(installed)

  // 2. 恢复上次选中的版本（复用已获取的列表校验版本是否仍然存在）
  await versionStore.restoreSelectedVersion(installed)

  // 3. 如果仍未选中，自动选中第一个已安装版本
  if (!versionStore.selectedVersion && installed.length > 0) {
    versionStore.selectedVersion = installed[0].id
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
