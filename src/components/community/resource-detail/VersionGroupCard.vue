<script setup lang="ts">
/**
 * 版本分组卡片（可折叠/展开）
 *
 * - 卡片标题栏（点击折叠/展开，带 icon 旋转动画）
 * - 懒挂载：首次展开才渲染版本条目
 * - grid-template-rows 0fr→1fr 过渡 + 内容 opacity/translate 渐入
 * - 版本条目含：release_type 徽章 / 游戏版本 / 加载器 / 大小 / 下载量 / 下载或安装按钮
 *
 * 内部 helper：releaseColor、loaderNames（仅本组件使用）
 */
import type { ResourceVersion } from '@/types/community'
import { ModLoaderFlags } from '@/types/community'
import { formatBytes, formatDownloads } from '@/utils/format'
import {
  ChevronDownIcon,
  CubeIcon,
  ArrowDownTrayIcon,
  RocketLaunchIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'

defineProps<{
  title: string
  versions: ResourceVersion[]
  expanded: boolean
  mounted: boolean
  downloading: string | null
  isModpack: boolean
}>()

const emit = defineEmits<{
  toggle: []
  download: [version: ResourceVersion]
  install: [version: ResourceVersion]
}>()

function loaderNames(flags: number): string[] {
  const list: string[] = []
  if (flags & ModLoaderFlags.Forge) list.push('Forge')
  if (flags & ModLoaderFlags.NeoForge) list.push('NeoForge')
  if (flags & ModLoaderFlags.Fabric) list.push('Fabric')
  if (flags & ModLoaderFlags.Quilt) list.push('Quilt')
  return list
}

function releaseColor(rt: string): string {
  if (rt === 'Release') return 'bg-green-100 text-green-700'
  if (rt === 'Beta') return 'bg-yellow-100 text-yellow-700'
  return 'bg-gray-100 text-gray-600'
}
</script>

<template>
  <div
    class="border border-gray-200 rounded-md overflow-hidden transition-colors duration-200"
    :class="expanded ? 'border-primary-200 bg-primary-50/30' : 'bg-white hover:border-gray-300'"
  >
    <!-- 卡片标题栏（点击折叠/展开） -->
    <button
      class="w-full flex items-center justify-between px-3 py-2.5 transition-colors duration-200"
      :class="expanded ? 'bg-primary-50/50 hover:bg-primary-100/50' : 'bg-gray-50 hover:bg-gray-100'"
      @click="emit('toggle')"
    >
      <div class="flex items-center gap-1.5">
        <CubeIcon
          class="w-3.5 h-3.5 transition-colors duration-300"
          :class="expanded ? 'text-primary-500' : 'text-gray-400'"
        />
        <span
          class="text-sm font-medium transition-colors duration-200"
          :class="expanded ? 'text-primary-700' : 'text-gray-700'"
        >{{ title }}</span>
      </div>
      <div class="flex items-center gap-2">
        <span
          class="text-xs transition-colors duration-200"
          :class="expanded ? 'text-primary-400' : 'text-gray-400'"
        >{{ versions.length }} 个版本</span>
        <span
          class="inline-flex items-center justify-center w-5 h-5 rounded-full transition-all duration-300 ease-[cubic-bezier(0.4,0,0.2,1)]"
          :class="expanded
            ? 'bg-primary-100 text-primary-600 rotate-180'
            : 'bg-gray-100 text-gray-500 rotate-0'"
        >
          <ChevronDownIcon class="w-3.5 h-3.5" />
        </span>
      </div>
    </button>

    <!-- 卡片内容（懒挂载 + grid-template-rows 0fr→1fr 过渡） -->
    <div
      v-if="mounted"
      class="grid transition-[grid-template-rows] duration-[400ms] ease-[cubic-bezier(0.4,0,0.2,1)]"
      :class="expanded ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
    >
      <div class="overflow-hidden">
        <div
          class="p-1.5 space-y-0.5 transition-all duration-[350ms] ease-[cubic-bezier(0.4,0,0.2,1)]"
          :class="expanded
            ? 'opacity-100 translate-y-0 transition-delay-[50ms]'
            : 'opacity-0 -translate-y-2 transition-delay-0'"
        >
          <div
            v-for="v in versions"
            :key="v.id"
            class="flex items-center gap-2 px-2 py-2 rounded-md hover:bg-gray-50 transition-colors"
          >
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-1.5">
                <span class="px-1 py-0.5 rounded text-[9px] font-medium" :class="releaseColor(v.release_type)">{{ v.release_type }}</span>
                <span class="text-sm text-gray-900 truncate">{{ v.display || v.file_name }}</span>
              </div>
              <div class="flex items-center gap-2 mt-0.5 text-[11px] text-gray-400">
                <span>{{ v.game_versions.slice(0, 3).join(', ') }}</span>
                <span v-for="l in loaderNames(v.mod_loaders)" :key="l" class="text-blue-500">{{ l }}</span>
                <span>{{ formatBytes(v.size) }}</span>
                <span>{{ formatDownloads(v.download_count) }} 下载</span>
              </div>
            </div>
            <Button
              type="primary"
              size="mini"
              class="shrink-0"
              :loading="downloading === v.id"
              @click="isModpack ? emit('install', v) : emit('download', v)"
            >
              <template #icon>
                <RocketLaunchIcon v-if="isModpack" class="w-3.5 h-3.5" />
                <ArrowDownTrayIcon v-else class="w-3.5 h-3.5" />
              </template>
              <template v-if="downloading === v.id">
                {{ isModpack ? '安装中...' : '下载中...' }}
              </template>
              <template v-else>
                {{ isModpack ? '安装' : '下载' }}
              </template>
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
