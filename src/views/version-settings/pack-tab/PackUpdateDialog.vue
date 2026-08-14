<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
/**
 * 资源包/光影版本更新/更改对话框（薄编排层）
 *
 * 逻辑在 composables/usePackUpdate.ts，版本列表表格复用 mod-tab/VersionTable.vue。
 * 本文件仅负责弹窗外壳（teleport + transition + 标题栏 + 内容区 + 底部操作栏）。
 */
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
import { usePackUpdate } from '@/composables/usePackUpdate'
import { formatDownloads } from '@/utils/format'
import { defaultAsset } from '@/utils/assets'
import type { PackInfo, PackKind } from '@/utils/api/personalization'
import { XMarkIcon, ArrowPathIcon } from '@heroicons/vue/24/outline'
const VersionTable = defineAsyncComponent(() => import('../mod-tab/VersionTable.vue'))

interface Props {
  visible: boolean
  /** 要更新/更改的包 */
  pack: PackInfo | null
  /** 内容类型：resourcepack / shader */
  kind: PackKind
  /** 当前版本的游戏版本号（如 "1.20.1"） */
  mcVersion: string
  /** 当前版本的 ID */
  versionId: string
}

const props = defineProps<Props>()
const emit = defineEmits<{
  'update:visible': [val: boolean]
  /** 安装完成后触发，父组件刷新列表 */
  installed: []
}>()

const {
  loading,
  versions,
  error,
  installing,
  selectedVersionId,
  filteredVersions,
  selectedVersion,
  installSelected,
} = usePackUpdate(props, emit)
</script>

<template>
  <teleport to="body">
    <transition
      enter-active-class="transition ease-out duration-150"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition ease-in duration-100"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="visible && pack"
        class="modal-shell"
        @click.self="$emit('update:visible', false)"
      >
        <div class="absolute inset-0 bg-black/40" />
        <div class="modal-body max-w-2xl mt-2">
          <!-- 标题栏 -->
          <div class="flex items-center justify-between px-5 py-3 border-b border-gray-200">
            <h3 class="text-sm font-semibold text-gray-900 flex items-center gap-2">
              <ArrowPathIcon class="w-4 h-4 text-blue-500" />
              更新 / 更改{{ kind === 'resourcepack' ? '资源包' : '光影' }}版本
            </h3>
            <Button type="ghost" size="small" @click="$emit('update:visible', false)">
              <template #icon><XMarkIcon class="w-5 h-5" /></template>
            </Button>
          </div>

          <!-- 内容区 -->
          <div class="modal-scroll p-5">
            <div class="flex flex-col gap-3">
              <!-- 当前包信息 -->
              <div class="flex items-center gap-3 p-3 bg-gray-50 rounded-lg">
                <img
                  :src="pack.cached_logo_url || defaultAsset(true)"
                  class="w-10 h-10 rounded-lg object-cover"
                  alt=""
                  @error="(e) => { (e.target as HTMLImageElement).src = defaultAsset(true) }"
                >
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium text-gray-800 truncate">{{ pack.project?.raw_name || pack.file_name }}</div>
                  <div class="text-xs text-gray-500">
                    当前版本：未知
                    <span v-if="pack.project" class="ml-2 text-gray-400">·</span>
                    <span v-if="pack.project" class="ml-2">{{ pack.project.platform }}</span>
                  </div>
                </div>
              </div>

              <!-- 加载中 -->
              <div v-if="loading" class="flex items-center justify-center py-8">
                <svg class="animate-spin w-6 h-6 text-blue-500" viewBox="0 0 24 24" fill="none">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                </svg>
                <span class="ml-2 text-sm text-gray-500">正在查询版本列表...</span>
              </div>

              <!-- 错误 -->
              <div v-else-if="error" class="p-4 bg-red-50 rounded-lg">
                <p class="text-sm text-red-600">{{ error }}</p>
              </div>

              <!-- 无版本数据 -->
              <div v-else-if="versions.length === 0" class="py-8 text-center">
                <p class="text-sm text-gray-500">未找到任何版本信息</p>
              </div>

              <!-- 版本列表表格 -->
              <VersionTable
                v-else
                :versions="filteredVersions"
                :selected-id="selectedVersionId"
                @update:selected-id="selectedVersionId = $event"
              />
            </div>
          </div>

          <!-- 底部操作栏 -->
          <div class="flex items-center justify-between gap-3 px-5 py-3 border-t border-gray-200 bg-gray-50 rounded-b-lg">
            <!-- 左侧：版本变化徽章（当前版本未知恒为 unknown）+ 下载量 -->
            <div v-if="selectedVersion" class="flex items-center gap-2 min-w-0">
              <div
                class="flex items-center gap-1 pl-2 pr-2.5 py-1 rounded-full border transition-colors bg-blue-50 border-blue-200"
              >
                <span class="w-1.5 h-1.5 rounded-full bg-blue-400 shrink-0"></span>
                <span
                  class="text-xs font-mono font-semibold text-blue-700"
                >{{ selectedVersion.version || '?' }}</span>
              </div>

              <span
                v-if="selectedVersion.download_count > 0"
                class="text-xs text-gray-400 whitespace-nowrap"
              >
                · {{ formatDownloads(selectedVersion.download_count) }} 次下载
              </span>
            </div>
            <div v-else class="flex-1"></div>
            <!-- 右侧：操作按钮 -->
            <div class="flex gap-2 shrink-0">
              <Button type="ghost" @click="$emit('update:visible', false)">取消</Button>
              <Button
                type="primary"
                :loading="installing"
                :disabled="!selectedVersion || installing"
                @click="installSelected"
              >
                {{ installing ? '安装中...' : '安装' }}
              </Button>
            </div>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>
