<script setup lang="ts">
/**
 * 下载管理页面
 * 参考 PCL2 设计：左右分栏布局
 * - 左侧：统计面板（总进度、速度、已下载、剩余文件）
 * - 右侧：任务列表（带子任务状态）
 */

import { computed } from 'vue'
import { useVersionStore } from '@/stores/version'
import { formatBytes, formatSpeed } from '@/utils/format'
import { 
  ArrowPathIcon, 
  CheckCircleIcon, 
  ClockIcon, 
  ArrowDownTrayIcon 
} from '@heroicons/vue/24/outline'

const versionStore = useVersionStore()

const hasActiveDownload = computed(() => versionStore.downloading)
const progress = computed(() => versionStore.downloadProgress)

const percentage = computed(() => progress.value?.percentage || 0)
const speed = computed(() => progress.value?.speed || 0)
const bytesDownloaded = computed(() => progress.value?.bytesDownloaded || 0)
const bytesTotal = computed(() => progress.value?.bytesTotal || 0)
const filesRemaining = computed(() => progress.value?.filesRemaining || 0)
const stage = computed(() => progress.value?.stage || '准备中...')
const current = computed(() => progress.value?.current || 0)
const total = computed(() => progress.value?.total || 0)

// 阶段列表（用于显示子任务状态）
const stages = [
  { id: 0, name: '版本清单', key: 'VersionManifest' },
  { id: 1, name: '版本信息', key: 'VersionJson' },
  { id: 2, name: '客户端', key: 'ClientJar' },
  { id: 3, name: '库文件', key: 'Libraries' },
  { id: 4, name: '资源文件', key: 'Assets' },
  { id: 5, name: '本地库', key: 'Natives' },
  { id: 6, name: '解压', key: 'ExtractNatives' },
  { id: 7, name: '模组', key: 'Mods' },
]

// 获取阶段状态
function getStageStatus(stageId: number): 'finished' | 'loading' | 'waiting' {
  if (!progress.value) return 'waiting'
  
  const currentStage = progress.value.stage
  
  if (stageId < currentStage) return 'finished'
  if (stageId === currentStage) return 'loading'
  return 'waiting'
}
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- 页面标题 -->
    <div class="px-6 py-4 bg-white border-b border-gray-200 shrink-0">
      <h1 class="text-lg font-semibold text-gray-900">下载管理</h1>
      <p class="text-xs text-gray-500 mt-0.5">查看和管理下载任务</p>
    </div>

    <!-- 内容区域 -->
    <div class="flex-1 overflow-hidden flex">
      <!-- 无下载任务时的空状态 -->
      <div v-if="!hasActiveDownload" class="flex-1 flex items-center justify-center">
        <div class="text-center">
          <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-gray-100 flex items-center justify-center">
            <ArrowDownTrayIcon class="w-8 h-8 text-gray-400" />
          </div>
          <h3 class="text-sm font-medium text-gray-900 mb-1">暂无下载任务</h3>
          <p class="text-xs text-gray-500">前往版本页面下载 Minecraft 或 Mod 加载器</p>
        </div>
      </div>

      <!-- 有下载任务时 -->
      <template v-else>
        <!-- 左侧：统计面板 -->
        <div class="w-56 border-r border-gray-200 bg-white flex flex-col shrink-0">
          <div class="flex-1 px-5 py-6 space-y-5">
            <!-- 当前阶段 -->
            <div>
              <div class="text-xs text-gray-500 mb-1">当前阶段</div>
              <div class="text-base font-semibold text-gray-900">{{ stage }}</div>
            </div>

            <!-- 分割线 -->
            <div class="h-px bg-gradient-to-r from-gray-100 to-gray-200"></div>

            <!-- 总进度 -->
            <div>
              <div class="text-xs text-gray-500 mb-1">总进度</div>
              <div class="text-2xl font-bold text-primary-600">
                {{ percentage.toFixed(1) }}%
              </div>
              <div v-if="total > 0" class="text-xs text-gray-400 mt-0.5">
                {{ current }} / {{ total }} 文件
              </div>
            </div>

            <!-- 分割线 -->
            <div class="h-px bg-gradient-to-r from-gray-100 to-gray-200"></div>

            <!-- 下载速度 -->
            <div>
              <div class="text-xs text-gray-500 mb-1">下载速度</div>
              <div class="text-lg font-semibold text-gray-900">
                {{ speed > 0 ? formatSpeed(speed) : '计算中...' }}
              </div>
            </div>

            <!-- 分割线 -->
            <div class="h-px bg-gradient-to-r from-gray-100 to-gray-200"></div>

            <!-- 已下载 / 总大小 -->
            <div>
              <div class="text-xs text-gray-500 mb-1">已下载 / 总大小</div>
              <div class="text-sm font-medium text-gray-900">
                {{ formatBytes(bytesDownloaded) }}
              </div>
              <div class="text-xs text-gray-400">
                {{ bytesTotal > 0 ? formatBytes(bytesTotal) : '计算中...' }}
              </div>
            </div>

            <!-- 分割线 -->
            <div class="h-px bg-gradient-to-r from-gray-100 to-gray-200"></div>

            <!-- 剩余文件 -->
            <div>
              <div class="text-xs text-gray-500 mb-1">剩余文件</div>
              <div class="text-lg font-semibold text-gray-900">{{ filesRemaining }}</div>
            </div>
          </div>
        </div>

        <!-- 右侧：任务列表 -->
        <div class="flex-1 overflow-y-auto p-6 bg-gray-50">
          <div class="max-w-2xl mx-auto">
            <!-- 任务卡片 -->
            <div class="bg-white rounded-xl border border-gray-200 overflow-hidden">
              <!-- 卡片头部 -->
              <div class="px-4 py-3 border-b border-gray-100 flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <ArrowPathIcon class="w-4 h-4 text-primary-500 animate-spin" />
                  <span class="text-sm font-medium text-gray-900">
                    {{ versionStore.downloadingVersion }}
                  </span>
                </div>
                <span class="text-xs text-gray-500">
                  {{ percentage.toFixed(1) }}%
                </span>
              </div>

              <!-- 子任务列表 -->
              <div class="divide-y divide-gray-50">
                <div
                  v-for="s in stages"
                  :key="s.id"
                  class="flex items-center gap-3 px-4 py-2.5"
                >
                  <!-- 状态图标 -->
                  <div class="w-5 h-5 flex items-center justify-center shrink-0">
                    <!-- 完成 -->
                    <CheckCircleIcon
                      v-if="getStageStatus(s.id) === 'finished'"
                      class="w-5 h-5 text-green-500"
                    />
                    <!-- 进行中 -->
                    <div
                      v-else-if="getStageStatus(s.id) === 'loading'"
                      class="w-5 h-5 rounded-full border-2 border-gray-200 border-t-primary-500 animate-spin"
                    />
                    <!-- 等待 -->
                    <ClockIcon
                      v-else
                      class="w-5 h-5 text-gray-300"
                    />
                  </div>

                  <!-- 阶段名称 -->
                  <span
                    class="text-sm"
                    :class="{
                      'text-gray-900 font-medium': getStageStatus(s.id) === 'loading',
                      'text-gray-700': getStageStatus(s.id) === 'finished',
                      'text-gray-400': getStageStatus(s.id) === 'waiting',
                    }"
                  >
                    {{ s.name }}
                  </span>

                  <!-- 进度百分比（仅进行中显示） -->
                  <span
                    v-if="getStageStatus(s.id) === 'loading' && total > 0"
                    class="ml-auto text-xs text-primary-600 font-medium"
                  >
                    {{ Math.round((current / total) * 100) }}%
                  </span>
                </div>
              </div>
            </div>

            <!-- 提示信息 -->
            <div class="mt-4 text-center text-xs text-gray-400">
              下载完成后将自动返回
            </div>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>
