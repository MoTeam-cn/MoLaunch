<script setup lang="ts">
/**
 * 资源卡片（单列行式，参考 PCL2 MyResourceItem）
 * 固定高度，横向布局：Logo + 标题/描述 + 版本/下载量/时间/来源
 */

import { computed } from 'vue'
import type { ResourceProject } from '@/types/community'
import { ModLoaderFlags } from '@/types/community'
import {
  ArrowDownTrayIcon,
  ClockIcon,
  GlobeAltIcon,
  TagIcon,
} from '@heroicons/vue/24/outline'

const props = defineProps<{ project: ResourceProject }>()
const emit = defineEmits<{ click: [project: ResourceProject] }>()

/** 格式化下载量（参考 PCL2） */
function formatDownloads(n: number): string {
  if (n >= 100_000_000) return (n / 100_000_000).toFixed(2) + ' 亿'
  if (n >= 10_000) return (n / 10_000).toFixed(1).replace(/\.0$/, '') + ' 万'
  return String(n)
}

/** 相对时间（参考 PCL2 LabTime） */
function formatRelativeTime(dateStr: string): string {
  if (!dateStr) return '未知'
  const date = new Date(dateStr)
  if (isNaN(date.getTime())) return '未知'
  const diff = Date.now() - date.getTime()
  const days = Math.floor(diff / 86400000)
  if (days <= 0) return '今天'
  if (days === 1) return '昨天'
  if (days < 30) return `${days} 天前`
  if (days < 365) return `${Math.floor(days / 30)} 个月前`
  return `${Math.floor(days / 365)} 年前`
}

/** 提取加载器标签 */
const loaders = computed(() => {
  const flags = props.project.mod_loaders
  const list: string[] = []
  if (flags & ModLoaderFlags.Forge) list.push('Forge')
  if (flags & ModLoaderFlags.NeoForge) list.push('NeoForge')
  if (flags & ModLoaderFlags.Fabric) list.push('Fabric')
  if (flags & ModLoaderFlags.Quilt) list.push('Quilt')
  if (flags & ModLoaderFlags.LiteLoader) list.push('LiteLoader')
  return list
})

/** 主要游戏版本（取前 3 个） */
const topVersions = computed(() => {
  const vs = [...props.project.game_versions].sort().reverse()
  return vs.slice(0, 3)
})

/** 分类标签（取前 4 个） */
const topTags = computed(() => props.project.tags.slice(0, 4))

const platformLabel = computed(() => props.project.platform)
</script>

<template>
  <div
    class="flex items-center gap-3 px-4 py-2.5 bg-white rounded-md border border-transparent hover:border-primary-200 hover:bg-primary-50/30 transition-all cursor-pointer group"
    style="height: 64px;"
    @click="emit('click', project)"
  >
    <!-- Logo -->
    <div class="shrink-0 w-11 h-11 rounded-md overflow-hidden bg-gray-100 flex items-center justify-center">
      <img
        v-if="project.logo_url"
        :src="project.logo_url"
        :alt="project.raw_name"
        class="w-full h-full object-cover"
        loading="lazy"
        @error="($event.target as HTMLImageElement).style.display = 'none'"
      >
      <span v-else class="text-lg text-gray-400">📦</span>
    </div>

    <!-- 中间信息 -->
    <div class="flex-1 min-w-0 flex flex-col justify-center gap-0.5">
      <!-- 标题行 -->
      <div class="flex items-center gap-2">
        <!-- 优先显示中文名，没有则显示英文原名 -->
        <span class="text-sm font-medium text-gray-900 truncate">
          {{ project.translated_name || project.raw_name }}
        </span>
        <!-- 有中文译名时，额外显示英文原名 -->
        <span v-if="project.translated_name" class="text-[10px] text-gray-400 truncate hidden sm:inline">
          {{ project.raw_name }}
        </span>
        <span
          v-for="loader in loaders"
          :key="loader"
          class="shrink-0 px-1 py-0.5 rounded text-[10px] font-medium bg-blue-50 text-blue-600"
        >{{ loader }}</span>
      </div>

      <!-- 描述行 -->
      <div class="flex items-center gap-3 text-[11px] text-gray-400">
        <span class="truncate flex-1">{{ project.description }}</span>
        <!-- 分类标签 -->
        <div v-if="topTags.length" class="hidden md:flex items-center gap-1 shrink-0">
          <TagIcon class="w-3 h-3 text-gray-300" />
          <span
            v-for="tag in topTags"
            :key="tag"
            class="px-1 py-0.5 rounded bg-gray-50 text-gray-400"
          >{{ tag }}</span>
        </div>
      </div>
    </div>

    <!-- 右侧元信息 -->
    <div class="shrink-0 flex items-center gap-4 text-[11px] text-gray-400">
      <!-- 游戏版本 -->
      <div v-if="topVersions.length" class="hidden lg:flex items-center gap-1">
        <span class="font-medium text-gray-500">{{ topVersions.join(', ') }}</span>
      </div>

      <!-- 下载量 -->
      <div class="flex items-center gap-1">
        <ArrowDownTrayIcon class="w-3 h-3" />
        <span>{{ formatDownloads(project.download_count) }}</span>
      </div>

      <!-- 更新时间 -->
      <div class="hidden sm:flex items-center gap-1">
        <ClockIcon class="w-3 h-3" />
        <span>{{ formatRelativeTime(project.last_update) }}</span>
      </div>

      <!-- 来源平台 -->
      <div class="flex items-center gap-1">
        <GlobeAltIcon class="w-3 h-3" />
        <span :class="project.platform === 'CurseForge' ? 'text-orange-500' : 'text-green-500'">
          {{ platformLabel === 'CurseForge' ? 'CF' : 'MR' }}
        </span>
      </div>
    </div>
  </div>
</template>
