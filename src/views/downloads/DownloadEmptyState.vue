<script setup lang="ts">
/**
 * 下载管理页空状态（无下载任务时）
 * 参考 PCL2 PageDownloads 空状态
 */
import { useRouter } from 'vue-router'
import { ArrowDownTrayIcon, CubeIcon, ArrowRightIcon } from '@heroicons/vue/24/outline'

const router = useRouter()

function goToVersions() {
  router.push('/apps/versions')
}
</script>

<template>
  <div class="flex-1 overflow-y-auto p-6">
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
</template>
