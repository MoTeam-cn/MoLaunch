<script setup lang="ts">
/**
 * 下载管理页面
 * 参考 PCL2 设计：左右分栏布局
 * - 左侧：统计面板（总进度、速度、已下载）
 * - 右侧：任务列表（带加权进度的阶段状态）
 */

import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useVersionStore } from '@/stores/version'
import { formatBytes, formatSpeed } from '@/utils/format'
import {
  ArrowPathIcon,
  CheckCircleIcon,
  ClockIcon,
  ArrowDownTrayIcon,
  CubeIcon,
  ArrowRightIcon,
  ChevronDownIcon,
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
  // 统计所有未完成阶段（loading + waiting）的剩余文件数
  let remaining = 0
  for (const s of progress.value.stages) {
    if (s.status === 'loading') {
      remaining += Math.max(0, (s.files_total || 0) - (s.files_downloaded || 0))
    } else if (s.status === 'waiting') {
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

// 按分组聚合 stages：有 group 的聚合成一个任务卡片，无 group 的独立成卡片
interface TaskGroup {
  key: string // 分组键：group 名或 stage.name（无 group 时用 name 作为独立分组）
  title: string // 显示名
  stages: typeof stages.value
  status: 'loading' | 'finished' | 'waiting' | 'failed'
  progress: number // 分组加权进度（0-1，按 stage.weight 加权平均）
  bytesDownloaded: number // 分组已下载字节（Finished + Loading 阶段累加）
  bytesTotal: number // 分组总字节
  isIndependent: boolean // 是否独立阶段（无 group）
}

const taskGroups = computed<TaskGroup[]>(() => {
  const groups: TaskGroup[] = []
  const groupMap = new Map<string, TaskGroup>()

  for (const s of stages.value) {
    const groupKey = s.group || s.name
    let g = groupMap.get(groupKey)
    if (!g) {
      g = {
        key: groupKey,
        title: s.group || s.name,
        stages: [],
        status: 'waiting',
        progress: 0,
        bytesDownloaded: 0,
        bytesTotal: 0,
        isIndependent: !s.group,
      }
      groupMap.set(groupKey, g)
      groups.push(g)
    }
    g.stages.push(s)
  }

  // 计算每个分组的聚合状态、加权进度和字节
  for (const g of groups) {
    const statuses = g.stages.map(s => s.status)
    if (statuses.includes('failed')) {
      g.status = 'failed'
    } else if (statuses.every(s => s === 'finished')) {
      g.status = 'finished'
    } else if (statuses.includes('loading')) {
      g.status = 'loading'
    } else {
      g.status = 'waiting'
    }
    // 加权进度（与整体 percentage 算法一致：按 stage.weight 加权平均）
    let weightedProgress = 0
    let totalWeight = 0
    for (const s of g.stages) {
      totalWeight += s.weight
      weightedProgress += s.progress * s.weight
    }
    g.progress = totalWeight > 0 ? weightedProgress / totalWeight : 0
    // 字节累加（仅 Finished + Loading 阶段，与后端 global_bytes 算法一致）
    for (const s of g.stages) {
      if (s.status === 'finished' || s.status === 'loading') {
        g.bytesDownloaded += s.bytes_downloaded
        g.bytesTotal += s.bytes_total
      }
    }
  }

  return groups
})

// 展开状态：默认全展开，用户点击可折叠
const collapsedGroups = ref<Set<string>>(new Set())

function toggleGroup(key: string) {
  if (collapsedGroups.value.has(key)) {
    collapsedGroups.value.delete(key)
  } else {
    collapsedGroups.value.add(key)
  }
}

function isExpanded(key: string): boolean {
  return !collapsedGroups.value.has(key)
}

function goToVersions() {
  router.push('/apps/versions')
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

              <!-- 任务分组列表（按 group 分组，可折叠展开看子阶段） -->
              <div class="divide-y divide-gray-100">
                <div v-for="g in taskGroups" :key="g.key" class="px-4 py-2">
                  <!-- 分组标题栏（点击折叠/展开） -->
                  <button
                    class="w-full flex items-center gap-2 py-1"
                    :class="g.isIndependent ? 'cursor-default' : 'hover:bg-gray-50 rounded -mx-1 px-1'"
                    @click="g.isIndependent ? undefined : toggleGroup(g.key)"
                  >
                    <!-- 状态图标 -->
                    <div class="w-5 h-5 flex items-center justify-center shrink-0">
                      <CheckCircleIcon v-if="g.status === 'finished'" class="w-5 h-5 text-green-500" />
                      <div v-else-if="g.status === 'loading'" class="w-5 h-5 rounded-full border-2 border-gray-200 border-t-primary-500 animate-spin" />
                      <div v-else-if="g.status === 'failed'" class="w-5 h-5 rounded-full bg-red-500" />
                      <ClockIcon v-else class="w-5 h-5 text-gray-300" />
                    </div>
                    <!-- 分组标题 -->
                    <span
                      class="text-sm flex-1 text-left"
                      :class="{
                        'text-gray-900 font-medium': g.status === 'loading',
                        'text-gray-700': g.status === 'finished',
                        'text-gray-400': g.status === 'waiting',
                        'text-red-600': g.status === 'failed',
                      }"
                    >
                      {{ g.title }}
                    </span>
                    <!-- 子阶段计数 + 分组字节 -->
                    <span v-if="!g.isIndependent" class="text-[10px] text-gray-400">
                      {{ g.stages.length }} 项
                    </span>
                    <span
                      v-if="g.bytesTotal > 0 && (g.status === 'loading' || g.status === 'finished')"
                      class="text-[10px] text-gray-400"
                    >
                      {{ formatBytes(g.bytesDownloaded) }} / {{ formatBytes(g.bytesTotal) }}
                    </span>
                    <!-- 进度 -->
                    <span
                      v-if="g.status === 'loading'"
                      class="text-xs text-primary-600 font-medium"
                    >
                      {{ Math.round(g.progress * 100) }}%
                    </span>
                    <span v-else-if="g.status === 'finished'" class="text-xs text-green-600 font-medium">100%</span>
                    <!-- 展开箭头（只有非独立分组才有） -->
                    <ChevronDownIcon
                      v-if="!g.isIndependent"
                      class="w-4 h-4 text-gray-400 transition-transform duration-200"
                      :class="isExpanded(g.key) ? 'rotate-180' : 'rotate-0'"
                    />
                  </button>

                  <!-- 子阶段列表（独立分组只显示自身，不展开子列表） -->
                  <div
                    v-if="!g.isIndependent && isExpanded(g.key)"
                    class="mt-1 ml-7 space-y-1 transition-all duration-200"
                  >
                    <div
                      v-for="(s, idx) in g.stages"
                      :key="idx"
                      class="flex items-center gap-2 py-1"
                    >
                      <!-- 子状态图标 -->
                      <div class="w-4 h-4 flex items-center justify-center shrink-0">
                        <CheckCircleIcon v-if="s.status === 'finished'" class="w-4 h-4 text-green-400" />
                        <div v-else-if="s.status === 'loading'" class="w-4 h-4 rounded-full border border-gray-200 border-t-primary-400 animate-spin" />
                        <ClockIcon v-else class="w-4 h-4 text-gray-300" />
                      </div>
                      <span
                        class="text-xs flex-1"
                        :class="{
                          'text-gray-900 font-medium': s.status === 'loading',
                          'text-gray-600': s.status === 'finished',
                          'text-gray-400': s.status === 'waiting',
                        }"
                      >
                        {{ s.name }}
                      </span>
                      <!-- 字节/文件进度 -->
                      <span v-if="s.status === 'loading' && s.bytes_total > 0" class="text-[10px] text-gray-400">
                        {{ formatBytes(s.bytes_downloaded) }} / {{ formatBytes(s.bytes_total) }}
                      </span>
                      <span v-else-if="s.status === 'loading' && s.files_total > 0" class="text-[10px] text-gray-400">
                        {{ s.files_downloaded }} / {{ s.files_total }} 文件
                      </span>
                      <span v-if="s.status === 'loading'" class="text-[11px] text-primary-600 font-medium">
                        {{ Math.round(s.progress * 100) }}%
                      </span>
                      <span v-else-if="s.status === 'finished'" class="text-[11px] text-green-600">100%</span>
                    </div>
                  </div>
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
