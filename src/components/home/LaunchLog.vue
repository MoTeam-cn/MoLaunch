<script setup lang="ts">
/**
 * 右侧内容区（参考 PCL2 PageLaunchRight + PanLaunching）
 * - 启动中：居中显示全局进度条 + 当前阶段名 + Java 下载进度
 * - 默认：空内容占位（后续可由用户/插件自定义内容）
 */

import { computed, ref, watch } from 'vue'
import { useVersionStore } from '@/stores/version'

const versionStore = useVersionStore()

// 控制 UI 显示：启动结束后延迟一会再隐藏，让用户看到 100%
const showLaunching = ref(false)
let hideTimer: number | null = null

watch(
  () => versionStore.launching,
  (launching) => {
    if (launching) {
      // 启动开始：立即显示
      if (hideTimer) { clearTimeout(hideTimer); hideTimer = null }
      showLaunching.value = true
    } else {
      // 启动结束：延迟 600ms 再隐藏，让进度条跑到 100%
      hideTimer = window.setTimeout(() => {
        showLaunching.value = false
        hideTimer = null
      }, 600)
    }
  },
  { immediate: true },
)

// 阶段名映射
const stageNames: Record<string, string> = {
  Init: '初始化',
  GetJava: '获取 Java',
  Login: '登录验证',
  ValidateFiles: '文件检查',
  BuildArgs: '构建参数',
  PreLaunch: '启动前命令',
  ExtractNatives: '解压原生库',
  LaunchProcess: '启动进程',
  WaitWindow: '等待窗口',
  Finished: '完成',
  Failed: '失败',
}

// 进度百分比（0-100，直接显示实际值，不做平滑——后端进度是离散的）
const showPercent = computed(() => {
  if (!versionStore.launchProgress) return 0
  const raw = versionStore.launchProgress.overall_progress
  // overall_progress 是 0.0-1.0，需要乘 100
  return Math.min(100, Math.max(0, raw * 100))
})

// 显示用：启动结束后强制 100%（后端 Finished 状态可能未被前端轮询到）
const displayPercent = computed(() => {
  if (!versionStore.launching) return 100
  return showPercent.value
})

const currentStageName = computed(() => {
  if (!versionStore.launchProgress) return ''
  return stageNames[versionStore.launchProgress.stage] || versionStore.launchProgress.stage
})

// 显示用：启动结束后显示"完成"
const displayStageName = computed(() => {
  if (!versionStore.launching) return '完成'
  return currentStageName.value
})

const launchingVersion = computed(() => versionStore.launchingVersionId || versionStore.selectedVersion || '')

// Java 自动下载进度
const javaDl = computed(() => versionStore.javaDownloadProgress)
const javaDlPercent = computed(() => {
  const p = javaDl.value
  if (!p || p.total === 0) return 0
  return Math.round((p.current / p.total) * 100)
})
const javaDlBytesPercent = computed(() => {
  const p = javaDl.value
  if (!p || p.bytes_total === 0) return 0
  return Math.round((p.bytes_downloaded / p.bytes_total) * 100)
})
const javaDlStageText = computed(() => {
  const p = javaDl.value
  if (!p) return ''
  switch (p.stage) {
    case 'fetching': return '正在获取 Java 索引...'
    case 'matching': return '正在匹配 Java 版本...'
    case 'manifest': return '正在获取文件清单...'
    case 'downloading': return `正在下载 Java (${p.current}/${p.total})`
    case 'verifying': return '正在验证 Java...'
    case 'done': return 'Java 下载完成'
    default: return p.message
  }
})

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const units = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return (bytes / Math.pow(k, i)).toFixed(1) + ' ' + units[i]
}
</script>

<template>
  <div class="flex h-full flex-col">
    <!-- 启动中：居中显示全局进度（参考 PCL2 PanLaunching） -->
    <Transition name="fade">
      <div
        v-if="showLaunching"
        class="flex flex-1 flex-col items-center justify-center px-8"
      >
        <!-- 旋转图标（启动完成后替换为对勾） -->
        <svg v-if="versionStore.launching" class="mb-5 h-10 w-10 animate-spin text-primary-500" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
          <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
        </svg>
        <svg v-else class="mb-5 h-10 w-10 text-green-500" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2" class="opacity-25" fill="currentColor" />
          <path d="M8 12l3 3 5-5" stroke="white" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>

        <!-- 标题 -->
        <h2 class="mb-1 text-xl font-medium text-gray-700">
          {{ versionStore.launching ? '正在启动游戏' : '启动完成' }}
        </h2>
        <!-- 版本名 -->
        <p class="mb-6 text-sm text-gray-400">{{ launchingVersion }}</p>

        <!-- 全局进度条 -->
        <div class="w-full max-w-md">
          <div class="h-1.5 overflow-hidden rounded-full bg-gray-100">
            <div
              class="h-full rounded-full bg-gradient-to-r from-primary-400 to-primary-600 transition-all duration-300 ease-out"
              :style="{ width: displayPercent + '%' }"
            />
          </div>
          <div class="mt-2 flex items-center justify-between text-xs">
            <span class="text-gray-500">{{ displayStageName }}</span>
            <span class="font-medium text-primary-600">{{ displayPercent.toFixed(1) }}%</span>
          </div>
        </div>

        <!-- Java 自动下载进度（启动流程触发时显示） -->
        <div v-if="javaDl" class="mt-4 w-full max-w-md rounded-lg bg-blue-50 px-4 py-3">
          <div class="mb-1.5 flex items-center justify-between text-xs">
            <span class="font-medium text-blue-700">{{ javaDlStageText }}</span>
            <span class="text-blue-600">{{ javaDlPercent }}%</span>
          </div>
          <div class="h-1.5 overflow-hidden rounded-full bg-blue-100">
            <div
              class="h-full rounded-full bg-blue-500 transition-all duration-300"
              :style="{ width: javaDlBytesPercent + '%' }"
            />
          </div>
          <div v-if="javaDl.bytes_total > 0" class="mt-1 text-[10px] text-blue-500">
            {{ formatBytes(javaDl.bytes_downloaded) }} / {{ formatBytes(javaDl.bytes_total) }}
          </div>
        </div>
      </div>
    </Transition>

    <!-- 默认：空内容占位（参考 PCL2 PanCustom 空栈） -->
    <div v-if="!showLaunching" class="flex flex-1 items-center justify-center p-8">
      <div class="text-center text-gray-300">
        <svg class="mx-auto mb-3 h-16 w-16 opacity-40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
          <path d="M4 5a1 1 0 011-1h14a1 1 0 011 1v14a1 1 0 01-1 1H5a1 1 0 01-1-1V5z" stroke-linecap="round" stroke-linejoin="round" />
          <path d="M9 9h6M9 13h6M9 17h3" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <p class="text-sm">自定义内容区</p>
        <p class="mt-1 text-xs text-gray-300">此处可由后续插件或用户配置填充</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
