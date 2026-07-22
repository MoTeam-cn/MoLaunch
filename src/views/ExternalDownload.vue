<script setup lang="ts">
/**
 * 外部下载工具子组件（由 Tools.vue 作为侧边栏内容区承载）
 *
 * 功能：
 * - 用户输入 URL，自动请求响应头获取文件名并补全（期间文件名输入框禁用）
 * - 通过后端 DownloadManager 下载到自定义目录或默认 .Molaunch/Download/
 * - 复用全局 download_state + versionStore，下载进度在下载管理页可见
 * - 支持暂停/取消/进度展示
 */
import {
  ArrowDownTrayIcon,
  FolderOpenIcon,
  ArrowTopRightOnSquareIcon,
  ArrowPathIcon,
  PauseIcon,
  PlayIcon,
  XMarkIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import { useExternalDownload } from '@/composables/useExternalDownload'
import DownloadedFileList from './external-download/DownloadedFileList.vue'

const {
  url,
  fileName,
  isFetchingFilename,
  onFileNameInput,
  downloadDir,
  isCustomDir,
  selectDownloadDir,
  resetDownloadDir,
  openDownloadDir,
  downloading,
  isPaused,
  percentage,
  speedFormatted,
  downloadedFormatted,
  currentFileName,
  canStartDownload,
  startDownload,
  togglePause,
  cancelDownloadTask,
  files,
  deleteFile,
} = useExternalDownload()
</script>

<template>
  <div class="mx-auto max-w-3xl space-y-6">
    <!-- 下载输入区 -->
    <section class="rounded-lg border border-gray-300 bg-white">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">新建下载</h3>
      <div class="px-5 pb-5 space-y-4">
        <!-- URL -->
        <div>
          <label class="mb-1.5 block text-xs font-medium text-gray-700">下载地址</label>
          <Input
            v-model="url"
            placeholder="https://example.com/file.zip"
            :disabled="downloading"
          />
        </div>

        <!-- 文件名（自动获取中禁用） -->
        <div>
          <label class="mb-1.5 block text-xs font-medium text-gray-700">
            保存文件名
            <span v-if="isFetchingFilename" class="ml-1 text-primary-500">正在获取文件名...</span>
          </label>
          <div class="relative">
            <Input
              v-model="fileName"
              placeholder="file.zip"
              :disabled="downloading || isFetchingFilename"
              @input="onFileNameInput"
            />
            <div
              v-if="isFetchingFilename"
              class="absolute right-3 top-1/2 -translate-y-1/2"
            >
              <ArrowPathIcon class="h-4 w-4 animate-spin text-primary-400" />
            </div>
          </div>
        </div>

        <!-- 下载进度区 -->
        <div v-if="downloading" class="rounded border border-gray-200 bg-gray-50 p-4">
          <!-- 文件名 + 操作按钮 -->
          <div class="mb-3 flex items-center justify-between">
            <span class="truncate text-sm font-medium text-gray-900">{{ currentFileName }}</span>
            <div class="flex flex-none items-center gap-2">
              <Button type="outline" size="mini" @click="togglePause">
                <template #icon>
                  <component :is="isPaused ? PlayIcon : PauseIcon" class="h-3.5 w-3.5" />
                </template>
                {{ isPaused ? '恢复' : '暂停' }}
              </Button>
              <Button type="outline" size="mini" @click="cancelDownloadTask">
                <template #icon>
                  <XMarkIcon class="h-3.5 w-3.5" />
                </template>
                取消
              </Button>
            </div>
          </div>

          <!-- 进度条 -->
          <div class="mb-2 h-2 w-full overflow-hidden rounded-full bg-gray-200">
            <div
              class="h-full rounded-full bg-primary-500 transition-all duration-300"
              :class="{ 'bg-yellow-400': isPaused }"
              :style="{ width: percentage + '%' }"
            />
          </div>

          <!-- 进度信息 -->
          <div class="flex items-center justify-between text-xs text-gray-500">
            <span>{{ downloadedFormatted }}</span>
            <span>{{ percentage.toFixed(1) }}% · {{ speedFormatted }}</span>
          </div>
        </div>

        <!-- 下载按钮 -->
        <Button
          v-else
          type="primary"
          long
          :disabled="!canStartDownload"
          @click="startDownload"
        >
          <template #icon>
            <ArrowDownTrayIcon class="h-4 w-4" />
          </template>
          开始下载
        </Button>
      </div>
    </section>

    <!-- 已下载文件列表 -->
    <DownloadedFileList :files="files" @delete="deleteFile" />

    <!-- 下载目录设置区 -->
    <section class="rounded-lg border border-gray-300 bg-white">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">下载目录</h3>
      <div class="px-5 pb-5 space-y-3">
        <!-- 目录路径展示条 -->
        <div class="flex items-center gap-3 rounded-lg bg-gray-50 px-4 py-3">
          <FolderOpenIcon class="h-5 w-5 flex-none text-primary-500" />
          <Tooltip :text="downloadDir || '.Molaunch/Download/'" position="top">
            <span class="flex-1 truncate text-sm text-gray-700">
              {{ downloadDir || '.Molaunch/Download/' }}
            </span>
          </Tooltip>
          <span
            v-if="isCustomDir"
            class="flex-none rounded-full bg-primary-100 px-2 py-0.5 text-xs font-medium text-primary-700"
          >
            自定义
          </span>
          <span
            v-else
            class="flex-none rounded-full bg-gray-200 px-2 py-0.5 text-xs font-medium text-gray-500"
          >
            默认
          </span>
        </div>

        <!-- 操作按钮 -->
        <div class="flex items-center gap-2">
          <Button type="outline" size="small" @click="selectDownloadDir">
            <template #icon>
              <FolderOpenIcon class="h-3.5 w-3.5" />
            </template>
            选择目录
          </Button>
          <Button type="outline" size="small" @click="openDownloadDir">
            <template #icon>
              <ArrowTopRightOnSquareIcon class="h-3.5 w-3.5" />
            </template>
            打开目录
          </Button>
          <Button v-if="isCustomDir" type="ghost" size="small" @click="resetDownloadDir">
            <template #icon>
              <ArrowPathIcon class="h-3.5 w-3.5" />
            </template>
            恢复默认
          </Button>
        </div>
      </div>
    </section>
  </div>
</template>
