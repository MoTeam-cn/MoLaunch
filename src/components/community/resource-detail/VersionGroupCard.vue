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
import type { ResourceProject, ResourceVersion } from '@/types/community'
import { ModLoaderFlags } from '@/types/community'
import { formatBytes, formatDownloads } from '@/utils/format'
import { ref } from 'vue'
import {
  ChevronDownIcon,
  CubeIcon,
  ArrowDownTrayIcon,
  RocketLaunchIcon,
  Squares2X2Icon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Tag from '@/components/common/Tag.vue'
import DependencyInlineList from './DependencyInlineList.vue'

defineProps<{
  title: string
  versions: ResourceVersion[]
  expanded: boolean
  mounted: boolean
  downloading: string | null
  /** 下载阶段（按钮文字分阶段显示） */
  downloadStage: 'idle' | 'requesting' | 'waiting' | 'downloading'
  isModpack: boolean
  /** 前置项目详情缓存（key=version_id） */
  depsMap: Map<string, ResourceProject[]>
  /** 正在加载前置的 version_id 集合 */
  depsLoadingSet: Set<string>
}>()

const emit = defineEmits<{
  toggle: []
  download: [version: ResourceVersion]
  install: [version: ResourceVersion]
  /** 懒加载请求查询该版本的前置项目详情 */
  loadDeps: [version: ResourceVersion]
}>()

/** 当前展开前置列表的 version_id 集合 */
const expandedDeps = ref(new Set<string>())

function toggleDeps(v: ResourceVersion) {
  if (expandedDeps.value.has(v.id)) {
    expandedDeps.value.delete(v.id)
  } else {
    expandedDeps.value.add(v.id)
    // 首次展开时通知父组件懒加载前置详情
    emit('loadDeps', v)
  }
}

function loaderNames(flags: number): string[] {
  const list: string[] = []
  if (flags & ModLoaderFlags.Forge) list.push('Forge')
  if (flags & ModLoaderFlags.NeoForge) list.push('NeoForge')
  if (flags & ModLoaderFlags.Fabric) list.push('Fabric')
  if (flags & ModLoaderFlags.Quilt) list.push('Quilt')
  return list
}

function releaseColor(rt: string): string {
  if (rt === 'Release') return 'green'
  if (rt === 'Beta') return 'gold'
  return 'gray'
}
</script>

<template>
  <div
    class="border border-gray-200 rounded-md overflow-hidden transition-colors duration-200"
    :class="expanded ? 'border-primary-200 bg-primary-50/30' : 'bg-white hover:border-gray-300'"
  >
    <!-- 卡片标题栏（点击折叠/展开） -->
    <!-- 保留原生 button：折叠头（w-full justify-between + active 状态 + 图标旋转），
         Button.vue 的 scoped size 类与布局不适合折叠头 -->
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
            class="px-2 py-2 rounded-md hover:bg-gray-50 transition-colors"
          >
            <div class="flex items-center gap-2">
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-1.5">
                  <Tag size="small" :color="releaseColor(v.release_type)">{{ v.release_type }}</Tag>
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
                  <template v-if="isModpack">安装中...</template>
                  <template v-else-if="downloadStage === 'requesting'">请求中...</template>
                  <template v-else-if="downloadStage === 'waiting'">等待中...</template>
                  <template v-else>下载中...</template>
                </template>
                <template v-else>
                  {{ isModpack ? '安装' : '下载' }}
                </template>
              </Button>
            </div>
            <!-- 前置依赖：仅 Mod 且有 dependencies 时展示 -->
            <div v-if="!isModpack && v.dependencies.length > 0" class="mt-1">
              <!-- 保留原生 button：前置依赖切换（inline text-[11px] + icon + toggle 状态），
                   Button.vue 的 scoped size 类固定 padding 会破坏紧凑 inline 布局 -->
              <button
                type="button"
                class="inline-flex items-center gap-1 text-[11px] text-primary-600 hover:text-primary-700 font-medium transition-colors"
                @click="toggleDeps(v)"
              >
                <Squares2X2Icon class="w-3 h-3" />
                <span>{{ expandedDeps.has(v.id) ? '收起前置' : `查看 ${v.dependencies.length} 个前置` }}</span>
              </button>
              <div v-if="expandedDeps.has(v.id)" class="mt-0.5">
                <DependencyInlineList
                  :deps="depsMap.get(v.id) || []"
                  :loading="depsLoadingSet.has(v.id)"
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
