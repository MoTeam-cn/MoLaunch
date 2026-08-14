<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
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
  InformationCircleIcon,
  Cog6ToothIcon,
} from '@heroicons/vue/24/outline'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const CollapsibleCard = defineAsyncComponent(() => import('@/components/common/CollapsibleCard.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
import { useExternalDownload } from '@/composables/useExternalDownload'
const DownloadedFileList = defineAsyncComponent(() => import('./external-download/DownloadedFileList.vue'))

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
  userAgent,
  maxThreads,
  chunkCount,
  maxSpeedMB,
  resetSettings,
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
            placeholder="请输入下载链接"
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
              placeholder="自动获取文件名"
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

    <!-- 工具原理说明 -->
    <section class="rounded-lg border border-gray-300 bg-white">
      <h3 class="flex items-center gap-1.5 px-5 pt-5 pb-3 text-sm font-semibold text-gray-900">
        <InformationCircleIcon class="h-4 w-4 text-primary-500" />
        工具原理
      </h3>
      <div class="px-5 pb-5 space-y-2 text-xs leading-relaxed text-gray-600">
        <p>
          本工具将下载地址发送给启动器内置下载引擎：后端通过
          <code class="rounded bg-gray-100 px-1 py-0.5 text-gray-800">DownloadManager</code>
          发起请求，把文件以多线程分片方式下载到下载目录，并接入全局下载进度（可在下载管理页查看）。
        </p>
        <p>
          支持<b class="text-gray-700">断点续传</b>（暂停/恢复）与<b class="text-gray-700">多线程分片</b>：
          大文件会按分片数拆成多段并行下载，完成后自动合并，显著提升速度。
        </p>
        <p>
          部分资源站会对请求方做校验（限速 / 防盗链），此时可展开<b class="text-gray-700">高级设置</b>
          自定义 User-Agent、调整线程数与分片数，或设置全局限速，从而兼容更多站点。
        </p>
      </div>
    </section>

    <!-- 高级设置 -->
    <CollapsibleCard>
      <template #title>
        <span class="flex items-center gap-1.5">
          <Cog6ToothIcon class="h-4 w-4 text-gray-500" />
          高级设置
        </span>
      </template>
      <div class="space-y-4">
        <div>
          <label class="mb-1.5 block text-xs font-medium text-gray-700">自定义 User-Agent</label>
          <Input
            v-model="userAgent"
            placeholder="留空使用默认 UA（如 Mozilla/5.0 ...）"
            :disabled="downloading"
          />
          <p class="mt-1 text-xs text-gray-400">部分资源站按 UA 识别下载客户端，留空自动使用启动器默认 UA。</p>
        </div>

        <div class="grid grid-cols-3 gap-3">
          <div>
            <label class="mb-1.5 block text-xs font-medium text-gray-700">并发线程数</label>
            <Input v-model.number="maxThreads" type="number" min="0" max="64" placeholder="跟随全局" :disabled="downloading" />
            <p class="mt-1 text-xs text-gray-400">0 = 使用全局配置</p>
          </div>
          <div>
            <label class="mb-1.5 block text-xs font-medium text-gray-700">单文件分片数</label>
            <Input v-model.number="chunkCount" type="number" min="0" max="32" placeholder="跟随全局" :disabled="downloading" />
            <p class="mt-1 text-xs text-gray-400">0/1 = 单流下载</p>
          </div>
          <div>
            <label class="mb-1.5 block text-xs font-medium text-gray-700">限速（MB/s）</label>
            <Input v-model.number="maxSpeedMB" type="number" min="0" max="1024" placeholder="不限速" :disabled="downloading" />
            <p class="mt-1 text-xs text-gray-400">0 = 不限速</p>
          </div>
        </div>

        <div class="flex justify-end">
          <Button type="ghost" size="small" @click="resetSettings">
            <template #icon>
              <ArrowPathIcon class="h-3.5 w-3.5" />
            </template>
            恢复默认
          </Button>
        </div>
      </div>
    </CollapsibleCard>

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
          <Tag
            v-if="isCustomDir"
            size="small"
            color="arcoblue"
            class="flex-none"
          >
            自定义
          </Tag>
          <Tag
            v-else
            size="small"
            color="gray"
            class="flex-none"
          >
            默认
          </Tag>
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
