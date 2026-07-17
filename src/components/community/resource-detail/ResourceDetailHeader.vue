<script setup lang="ts">
/**
 * 资源详情弹窗头部
 *
 * - Logo + 标题（译名/原名）+ 描述 + 平台标签 + 下载量
 * - 操作按钮行：转到平台官网 / MC百科 / 复制名称
 * - close 按钮
 *
 * openMcmod 和 copyName 内部处理（只依赖 project prop）
 */
import { open as openUrl } from '@tauri-apps/plugin-shell'
import type { ResourceProject } from '@/types/community'
import { getMcmodUrl } from '@/utils/api/community'
import { showSuccess, showError } from '@/utils/toast'
import { formatDownloads } from '@/utils/format'
import {
  XMarkIcon,
  ArrowTopRightOnSquareIcon,
  ClipboardDocumentIcon,
  BookOpenIcon,
  CubeIcon,
} from '@heroicons/vue/24/outline'

const props = defineProps<{ project: ResourceProject }>()
const emit = defineEmits<{ close: [] }>()

/** 点击"转到 MC百科"：先查 class id 直链，查不到回退搜索 URL */
async function openMcmod() {
  const { platform, slug, translated_name, raw_name } = props.project
  let url = await getMcmodUrl(platform, slug)
  if (!url) {
    const name = translated_name || raw_name
    url = `https://search.mcmod.cn/s?key=${encodeURIComponent(name)}`
    showSuccess('未找到 MC 百科直链，已跳转到搜索页')
  } else {
    showSuccess('正在打开 MC 百科详情页')
  }
  await openUrl(url)
}

/** 复制资源名称到剪贴板 */
async function copyName() {
  const name = props.project.translated_name || props.project.raw_name || ''
  if (!name) return
  try {
    await navigator.clipboard.writeText(name)
    showSuccess('已复制: ' + name)
  } catch {
    showError('复制失败')
  }
}
</script>

<template>
  <!-- 头部：Logo + 标题 + 操作按钮 -->
  <div class="flex items-start gap-3 p-4 border-b border-gray-200">
    <div class="shrink-0 w-12 h-12 rounded-md overflow-hidden bg-gray-100 flex items-center justify-center">
      <img v-if="project.logo_url" :src="project.logo_url" :alt="project.raw_name" class="w-full h-full object-cover">
      <CubeIcon v-else class="w-6 h-6 text-gray-400" />
    </div>
    <div class="flex-1 min-w-0">
      <h2 class="text-base font-semibold text-gray-900 truncate">
        {{ project.translated_name || project.raw_name }}
      </h2>
      <p v-if="project.translated_name" class="text-xs text-gray-400 mt-0.5 truncate">
        {{ project.raw_name }}
      </p>
      <p class="text-xs text-gray-500 mt-0.5 line-clamp-2">{{ project.description }}</p>
      <div class="flex items-center gap-2 mt-1.5">
        <span class="px-1.5 py-0.5 rounded text-[10px] font-medium"
          :class="project.platform === 'CurseForge' ? 'bg-orange-100 text-orange-700' : 'bg-green-100 text-green-700'">
          {{ project.platform }}
        </span>
        <span class="text-[11px] text-gray-400">{{ formatDownloads(project.download_count) }} 下载</span>
      </div>
    </div>
    <button class="p-1 rounded text-gray-400 hover:bg-gray-100" @click="emit('close')">
      <XMarkIcon class="w-5 h-5" />
    </button>
  </div>

  <!-- 操作按钮行 -->
  <div class="flex items-center gap-2 px-4 py-2 border-b border-gray-100">
    <button
      v-if="project.website"
      class="px-3 py-1.5 rounded-md text-xs font-medium text-white bg-primary-600 hover:bg-primary-700 transition-colors flex items-center gap-1"
      @click="openUrl(project.website)"
    >
      <ArrowTopRightOnSquareIcon class="w-3.5 h-3.5" />
      转到 {{ project.platform }}
    </button>
    <button
      v-if="project.resource_type === 'Mod'"
      class="px-3 py-1.5 rounded-md text-xs font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 transition-colors flex items-center gap-1"
      @click="openMcmod"
    >
      <BookOpenIcon class="w-3.5 h-3.5" />
      转到 MC百科
    </button>
    <button
      class="px-3 py-1.5 rounded-md text-xs font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 transition-colors flex items-center gap-1"
      @click="copyName"
    >
      <ClipboardDocumentIcon class="w-3.5 h-3.5" />
      复制名称
    </button>
  </div>
</template>
