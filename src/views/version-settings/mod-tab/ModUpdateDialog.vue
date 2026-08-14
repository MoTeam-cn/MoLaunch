<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
/**
 * Mod 版本更新/更改对话框（薄编排层）
 *
 * 逻辑已抽离到 composables/useModUpdate.ts，版本列表表格抽离到 VersionTable.vue。
 * 本文件仅负责弹窗外壳（teleport + transition + 标题栏 + 内容区 + 底部操作栏）。
 *
 * 采用 teleport + transition 自承载弹窗（与 ResourceDetail 一致），
 * 不使用 singleton Modal（Modal 仅适合简单确认/提示，不支持自定义宽度和表格内容）。
 */
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
import { useModUpdate } from '@/composables/useModUpdate'
import { formatDownloads } from '@/utils/format'
import type { ModInfo } from '@/utils/api/personalization'
import {
  XMarkIcon,
  ArrowPathIcon,
  CheckCircleIcon,
  ArrowUpIcon,
  ArrowDownIcon,
} from '@heroicons/vue/24/outline'
const VersionTable = defineAsyncComponent(() => import('./VersionTable.vue'))
import { defaultAsset } from '@/utils/assets'

interface Props {
  visible: boolean
  /** 要更新/更改的 mod */
  mod: ModInfo | null
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
  versionChange,
  installSelected,
} = useModUpdate(props, emit)
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
        v-if="visible && mod"
        class="modal-shell"
        @click.self="$emit('update:visible', false)"
      >
        <div class="absolute inset-0 bg-black/40" />
        <div class="modal-body max-w-2xl mt-2">
          <!-- 标题栏 -->
          <div class="flex items-center justify-between px-5 py-3 border-b border-gray-200">
            <h3 class="text-sm font-semibold text-gray-900 flex items-center gap-2">
              <ArrowPathIcon class="w-4 h-4 text-blue-500" />
              更新 / 更改 Mod 版本
            </h3>
            <Button type="ghost" size="small" @click="$emit('update:visible', false)">
              <template #icon><XMarkIcon class="w-5 h-5" /></template>
            </Button>
          </div>

          <!-- 内容区 -->
          <div class="modal-scroll p-5">
            <div class="flex flex-col gap-3">
              <!-- 当前 mod 信息 -->
              <div class="flex items-center gap-3 p-3 bg-gray-50 rounded-lg">
                <img
                  :src="mod.cached_logo_url || defaultAsset(true)"
                  class="w-10 h-10 rounded-lg object-cover"
                  alt=""
                  @error="(e) => { (e.target as HTMLImageElement).src = defaultAsset(true) }"
                >
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium text-gray-800 truncate">{{ mod.project?.raw_name || mod.file_name }}</div>
                  <div class="text-xs text-gray-500">
                    当前版本：{{ mod.version || '未知' }}
                    <span v-if="mod.project" class="ml-2 text-gray-400">·</span>
                    <span v-if="mod.project" class="ml-2">{{ mod.project.platform }}</span>
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
            <!-- 左侧：版本变化徽章 + 下载量 -->
            <div v-if="selectedVersion" class="flex items-center gap-2 min-w-0">
              <!-- 胶囊式版本变化徽章：图标 + 旧版本(删除线) + 箭头 + 新版本(高亮) -->
              <div
                class="flex items-center gap-1 pl-2 pr-2.5 py-1 rounded-full border transition-colors"
                :class="{
                  'bg-green-50 border-green-200': versionChange === 'upgrade',
                  'bg-amber-50 border-amber-200': versionChange === 'downgrade',
                  'bg-gray-100 border-gray-200': versionChange === 'same',
                  'bg-blue-50 border-blue-200': versionChange === 'unknown',
                }"
              >
                <ArrowUpIcon v-if="versionChange === 'upgrade'" class="w-3.5 h-3.5 text-green-600 shrink-0" />
                <ArrowDownIcon v-else-if="versionChange === 'downgrade'" class="w-3.5 h-3.5 text-amber-600 shrink-0" />
                <CheckCircleIcon v-else-if="versionChange === 'same'" class="w-3.5 h-3.5 text-gray-500 shrink-0" />
                <span v-else class="w-1.5 h-1.5 rounded-full bg-blue-400 shrink-0"></span>

                <span
                  v-if="mod.version && versionChange !== 'same'"
                  class="text-xs font-mono text-gray-400 line-through decoration-gray-300"
                >{{ mod.version }}</span>
                <span v-if="versionChange !== 'same'" class="text-xs text-gray-400">→</span>
                <span
                  class="text-xs font-mono font-semibold"
                  :class="{
                    'text-green-700': versionChange === 'upgrade',
                    'text-amber-700': versionChange === 'downgrade',
                    'text-gray-700': versionChange === 'same',
                    'text-blue-700': versionChange === 'unknown',
                  }"
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
