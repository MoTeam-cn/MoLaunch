<script setup lang="ts">
/**
 * 前置 Mod 内联列表（版本条目下方展开）
 *
 * - loading 时展示加载动画
 * - 加载完成展示前置 mod 的 logo + 名称 + 平台标签
 * - 空列表展示"无前置依赖"
 *
 * 调用方：VersionGroupCard.vue 在版本条目下方展开时引用
 */
import type { ResourceProject } from '@/types/community'
import { CubeIcon } from '@heroicons/vue/24/outline'
import CachedImage from '@/components/common/CachedImage.vue'

defineProps<{
  /** 前置项目详情列表 */
  deps: ResourceProject[]
  /** 是否正在加载 */
  loading: boolean
}>()
</script>

<template>
  <div class="mt-1 ml-4 pl-3 border-l-2 border-gray-100 space-y-1">
    <!-- 加载中 -->
    <div v-if="loading" class="flex items-center gap-2 py-1.5 text-xs text-gray-400">
      <svg class="h-3.5 w-3.5 animate-spin text-primary-400" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
        <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
      </svg>
      <span>正在加载前置依赖...</span>
    </div>

    <!-- 空列表 -->
    <div v-else-if="deps.length === 0" class="flex items-center gap-1.5 py-1 text-xs text-gray-400">
      <CubeIcon class="w-3.5 h-3.5" />
      <span>无前置依赖</span>
    </div>

    <!-- 前置列表 -->
    <div
      v-for="dep in deps"
      :key="dep.id"
      class="flex items-center gap-2 py-1 px-1.5 rounded hover:bg-gray-50 transition-colors"
    >
      <CachedImage
        :src="dep.logo_url"
        :alt="dep.raw_name"
        class="w-5 h-5 rounded shrink-0 object-cover"
      >
        <template #fallback>
          <div class="w-5 h-5 rounded shrink-0 bg-gray-100 flex items-center justify-center">
            <CubeIcon class="w-3 h-3 text-gray-400" />
          </div>
        </template>
      </CachedImage>
      <span class="text-xs text-gray-700 truncate flex-1">
        {{ dep.translated_name || dep.raw_name }}
      </span>
      <span
        class="text-[9px] px-1 py-0.5 rounded shrink-0"
        :class="dep.platform === 'CurseForge' ? 'bg-orange-100 text-orange-600' : 'bg-green-100 text-green-600'"
      >
        {{ dep.platform === 'CurseForge' ? 'CF' : 'MR' }}
      </span>
    </div>
  </div>
</template>
