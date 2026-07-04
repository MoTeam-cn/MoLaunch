<script setup lang="ts">
/**
 * 下载管理页面
 * 参考 PCL2 设计：左右分栏布局
 * - 左侧：统计面板（总进度、速度、已下载）
 * - 右侧：任务列表（带加权进度的阶段状态）
 */

import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useVersionStore } from '@/stores/version'
import { formatBytes, formatSpeed } from '@/utils/format'
import { 
  ArrowPathIcon, 
  CheckCircleIcon, 
  ClockIcon, 
  ArrowDownTrayIcon,
  CubeIcon,
  ArrowRightIcon
} from '@heroicons/vue/24/outline'

const router = useRouter()
const versionStore = useVersionStore()

const hasActiveDownload = computed(() => versionStore.downloading)
const progress = computed(() => versionStore.downloadProgress)

const percentage = computed(() => progress.value?.percentage || 0)
const speed = computed(() => progress.value?.global_speed || 0)
const bytesDownloaded = computed(() => progress.value?.global_bytes_downloaded || 0)
const bytesTotal = computed(() => progress.value?.global_bytes_total || 0)
const filesRemaining = computed(() => {
  if (!progress.value?.stages) return 0
  // 计算所有阶段中剩余的文件数
  let remaining = 0
  for (const s of progress.value.stages) {
    if (s.status === 'loading') {
      // 进行中的阶段：总文件数 - 已下载文件数
      remaining += Math.max(0, (s.files_total || 0) - (s.files_downloaded || 0))
    } else if (s.status === 'waiting') {
      // 等待中的阶段：使用总文件数
      remaining += s.files_total || 0
    }
  }
  return remaining
})
const stages = computed(() => progress.value?.stages || [])
const currentStageIndex = computed(() => progress.value?.current_stage_index ?? 0)

const currentStageName = computed(() => {
  const s = stages.value
  const idx = currentStageIndex.value
  return idx < s.length ? s[idx].name : '准备中...'
})

function goToVersions() {
  router.push('/versions')
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
      <div v-if="!hasActiveDownload" class="flex-1 overflow-y-auto p-6">
        <div class="max-w-md mx-auto">
          <!-- 主提示卡片 -->
          <div class="bg-white rounded-xl border border-gray-200 p-8 text-center">
            <div class="w-20 h-20 mx-auto mb-5 rounded-2xl bg-primary-50 flex items-center justify-center">
              <ArrowDownTrayIcon class="w-10 h-10 text-primary-500" />
            </div>
            <h3 class="text-lg font-semibold text-gray-900 mb-2">暂无下载任务</h3>
            <p class="text-sm text-gray-500 mb-6">当前没有正在下载的任务，前往版本页面开始下载吧</p>
            <button
              class="inline-flex items-center px-5 py-2.5 bg-primary-600 text-white text-sm font-medium rounded-lg hover:bg-primary-700 active:scale-[0.98] transition-all"
              @click="goToVersions"
            >
              <CubeIcon class="w-4 h-4 mr-2" />
              浏览版本
              <ArrowRightIcon class="w-4 h-4 ml-1.5" />
            </button>
          </div>

          <!-- 提示信息卡片 -->
          <div class="mt-4 bg-white rounded-xl border border-gray-200 p-5">
            <h4 class="text-sm font-medium text-gray-900 mb-3">下载管理功能</h4>
            <div class="space-y-3">
              <div class="flex items-start gap-3">
                <div class="w-8 h-8 rounded-lg bg-blue-50 flex items-center justify-center shrink-0 mt-0.5">
                  <svg class="w-4 h-4 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                </div>
                <div>
                  <p class="text-sm font-medium text-gray-800">实时进度监控</p>
                  <p class="text-xs text-gray-500 mt-0.5">查看下载速度、已下载大小、剩余文件数</p>
                </div>
              </div>
              <div class="flex items-start gap-3">
                <div class="w-8 h-8 rounded-lg bg-green-50 flex items-center justify-center shrink-0 mt-0.5">
                  <svg class="w-4 h-4 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                  </svg>
                </div>
                <div>
                  <p class="text-sm font-medium text-gray-800">多阶段状态显示</p>
                  <p class="text-xs text-gray-500 mt-0.5">版本清单、客户端、库文件、资源等各阶段进度</p>
                </div>
              </div>
              <div class="flex items-start gap-3">
                <div class="w-8 h-8 rounded-lg bg-purple-50 flex items-center justify-center shrink-0 mt-0.5">
                  <svg class="w-4 h-4 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                </div>
                <div>
                  <p class="text-sm font-medium text-gray-800">快速访问</p>
                  <p class="text-xs text-gray-500 mt-0.5">双击顶部导航栏的"下载"按钮可快速进入此页面</p>
                </div>
              </div>
            </div>
          </div>
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
              <div class="text-base font-semibold text-gray-900">{{ currentStageName }}</div>
            </div>

            <!-- 分割线 -->
            <div class="h-px bg-gradient-to-r from-gray-100 to-gray-200"></div>

            <!-- 总进度（加权平均） -->
            <div>
              <div class="text-xs text-gray-500 mb-1">总进度</div>
              <div class="text-2xl font-bold text-primary-600">
                {{ percentage.toFixed(1) }}%
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

            <!-- 已下载 / 总大小（累计） -->
            <div>
              <div class="text-xs text-gray-500 mb-1">已下载 / 总大小（累计）</div>
              <div class="text-sm font-medium text-gray-900">
                {{ formatBytes(bytesDownloaded) }}
              </div>
              <div class="text-xs text-gray-400">
                {{ bytesTotal > 0 ? formatBytes(bytesTotal) : '计算中...' }}
              </div>
            </div>

            <!-- 分割线 -->
            <div class="h-px bg-gradient-to-r from-gray-100 to-gray-200"></div>

            <!-- 剩余文件（当前阶段） -->
            <div>
              <div class="text-xs text-gray-500 mb-1">剩余文件（当前阶段）</div>
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
                  v-for="(s, idx) in stages"
                  :key="idx"
                  class="flex items-center gap-3 px-4 py-2.5"
                >
                  <!-- 状态图标 -->
                  <div class="w-5 h-5 flex items-center justify-center shrink-0">
                    <!-- 完成 -->
                    <CheckCircleIcon
                      v-if="s.status === 'finished'"
                      class="w-5 h-5 text-green-500"
                    />
                    <!-- 进行中 -->
                    <div
                      v-else-if="s.status === 'loading'"
                      class="w-5 h-5 rounded-full border-2 border-gray-200 border-t-primary-500 animate-spin"
                    />
                    <!-- 失败 -->
                    <div
                      v-else-if="s.status === 'failed'"
                      class="w-5 h-5 rounded-full bg-red-500"
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
                      'text-gray-900 font-medium': s.status === 'loading',
                      'text-gray-700': s.status === 'finished',
                      'text-gray-400': s.status === 'waiting',
                      'text-red-600': s.status === 'failed',
                    }"
                  >
                    {{ s.name }}
                  </span>

                  <!-- 权重标签 -->
                  <span class="text-[10px] text-gray-400 bg-gray-100 px-1.5 py-0.5 rounded">
                    {{ Math.round(s.weight) }}s
                  </span>

                  <!-- 进度百分比 -->
                  <span
                    v-if="s.status === 'loading'"
                    class="ml-auto text-xs text-primary-600 font-medium"
                  >
                    {{ Math.round(s.progress * 100) }}%
                  </span>
                  <span
                    v-else-if="s.status === 'finished'"
                    class="ml-auto text-xs text-green-600 font-medium"
                  >
                    100%
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
