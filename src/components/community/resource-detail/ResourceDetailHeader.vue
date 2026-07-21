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
import { ref, watch } from 'vue'
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
import Button from '@/components/common/Button.vue'

const props = defineProps<{ project: ResourceProject }>()
const emit = defineEmits<{ close: [] }>()

/** MC 百科详情页直链（查不到则为 null，按钮不显示） */
const mcmodUrl = ref<string | null>(null)

// project 变化时异步查询 MC 百科直链，查不到则按钮不显示
watch(
  () => props.project,
  async (p) => {
    mcmodUrl.value = null
    if (!p || p.resource_type !== 'Mod') return
    try {
      mcmodUrl.value = await getMcmodUrl(p.platform, p.slug)
    } catch (e) {
      console.debug('[ResourceDetailHeader] 查询 MC 百科直链失败:', e)
    }
  },
  { immediate: true },
)

/** 点击"转到 MC百科"：直接打开已查到的直链 */
async function openMcmod() {
  if (!mcmodUrl.value) return
  showSuccess('正在打开 MC 百科详情页')
  await openUrl(mcmodUrl.value)
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
        <span
class="px-1.5 py-0.5 rounded text-[10px] font-medium"
          :class="project.platform === 'CurseForge' ? 'bg-orange-100 text-orange-700' : 'bg-green-100 text-green-700'">
          {{ project.platform }}
        </span>
        <span class="text-[11px] text-gray-400">{{ formatDownloads(project.download_count) }} 下载</span>
      </div>
    </div>
    <Button type="ghost" size="mini" @click="emit('close')">
      <template #icon><XMarkIcon class="w-5 h-5" /></template>
    </Button>
  </div>

  <!-- 操作按钮行 -->
  <div class="flex items-center gap-2 px-4 py-2 border-b border-gray-100">
    <Button
      v-if="project.website"
      type="primary"
      size="small"
      @click="openUrl(project.website)"
    >
      <template #icon><ArrowTopRightOnSquareIcon class="w-3.5 h-3.5" /></template>
      转到 {{ project.platform }}
    </Button>
    <Button
      v-if="mcmodUrl"
      type="secondary"
      size="small"
      @click="openMcmod"
    >
      <template #icon><BookOpenIcon class="w-3.5 h-3.5" /></template>
      转到 MC百科
    </Button>
    <Button
      type="secondary"
      size="small"
      @click="copyName"
    >
      <template #icon><ClipboardDocumentIcon class="w-3.5 h-3.5" /></template>
      复制名称
    </Button>
  </div>
</template>
