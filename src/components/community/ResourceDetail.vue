<script setup lang="ts">
/**
 * 资源详情弹窗
 *
 * - 顶部资源预览 + 操作按钮（ResourceDetailHeader 子组件）
 * - 版本筛选 RadioButton（HorizontalFilter）
 * - 版本按游戏版本分组卡片（VersionGroupCard 子组件，可折叠/展开带动画）
 * - 加载进度条
 * - 下载进度浮层（DownloadProgressOverlay 子组件）
 *
 * 下载/前置检查/整合包安装逻辑分别抽到 useResourceDownload / useResourceModpackInstall composable。
 */
import { ref, watch, nextTick } from 'vue'
import type { ResourceProject, ResourceVersion } from '@/types/community'
import { getProjectVersions } from '@/utils/api/community'
import { toastError } from '@/utils/toast'
import { useVersionGroups, getFilterVersionName } from '@/composables/useVersionGroups'
import { useSearchProgress } from '@/composables/useSearchProgress'
import { useResourceDownload } from '@/composables/useResourceDownload'
import { useResourceModpackInstall } from '@/composables/useResourceModpackInstall'
import HorizontalFilter from '@/components/common/HorizontalFilter.vue'
import ResourceDetailHeader from './resource-detail/ResourceDetailHeader.vue'
import { ArchiveBoxXMarkIcon } from '@heroicons/vue/24/outline'
import VersionGroupCard from './resource-detail/VersionGroupCard.vue'
import DependencyConfirmDialog from './DependencyConfirmDialog.vue'

interface Props {
  visible: boolean
  project: ResourceProject | null
  versionId?: string
  /** 整合包对应的 MC 版本号，设置后自动选中顶部筛选 tag */
  gameVersion?: string
  /** 整合包的 mods 目录路径，设置后下载 Mod 默认保存到该目录 */
  modsDir?: string
  /** 是否禁止更新 Mod（版本独立设置 advance_disable_mod_update），开启后下载已存在文件时拦截 */
  disableModUpdate?: boolean
}

const props = defineProps<Props>()
const emit = defineEmits<{ close: [] }>()

const versions = ref<ResourceVersion[]>([])
const loading = ref(false)

const { groups, filterOptions, versionFilter, toggleGroup, setFilter, expandedOf, mountedOf } = useVersionGroups(() => versions.value)
const { percent, slowMerging, stageText, start, finish, fail } = useSearchProgress()

// 下载 + 前置 Mod 检查逻辑（直接传 props 保持响应式，composable 内部通过 options.xxx 访问最新值）
const {
  downloading, downloadStage, showDependencyDialog, pendingMainVersion,
  depsMap, depsLoadingSet,
  depsChecking, depsInstalling, depsMissing, depsUpToDate,
  handleDownload, handleDependencyConfirm, handleDependencyClose, handleLoadDeps,
} = useResourceDownload(props)

// 整合包安装逻辑（共享 useResourceDownload 的 downloading 状态）
const { handleInstallModpack } = useResourceModpackInstall(props, downloading)

watch(
  [() => props.visible, () => props.project],
  async ([v, p], [oldV, oldP]) => {
    // 仅在 visible 变为 true 或 project 变化时触发（避免 visible/gameVersion 单独变化重复加载）
    if (!v || !p) return
    if (v === oldV && p === oldP) return
    loading.value = true
    versions.value = []
    depsMap.value = new Map()
    depsLoadingSet.value = new Set()
    setFilter('')
    start(p.platform === 'CurseForge' ? 1 : p.platform === 'Modrinth' ? 2 : 0)
    try {
      versions.value = await getProjectVersions(p.platform, p.id, p.resource_type)
      finish()
      // 整合包来自 ModTab 时自动选中对应版本筛选
      if (props.gameVersion) {
        const target = getFilterVersionName(props.gameVersion)
        if (target && filterOptions.value.includes(target)) {
          nextTick(() => setFilter(target))
        }
      }
    } catch (e: any) {
      toastError('加载版本列表失败: ' + (e?.message || String(e)))
      fail()
    } finally {
      loading.value = false
    }
  },
)
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
        v-if="visible && project"
        class="fixed inset-0 z-[10000] flex items-start justify-center px-4 pt-14 pb-4"
        @click.self="emit('close')"
      >
        <div class="absolute inset-0 bg-black/40" />
        <div class="relative w-full max-w-2xl bg-white rounded-lg shadow-xl flex flex-col max-h-[calc(100vh-100px)] mt-2">
          <!-- 头部 + 操作按钮 -->
          <ResourceDetailHeader :project="project" @close="emit('close')" />

          <!-- 版本筛选 -->
          <div v-if="filterOptions.length > 1" class="px-4 py-2 border-b border-gray-100 bg-gray-50/50">
            <HorizontalFilter
              :model-value="versionFilter"
              :options="filterOptions.map(o => ({ label: o, value: o }))"
              @update:model-value="setFilter"
            />
          </div>

          <!-- 版本列表区 -->
          <div class="flex-1 overflow-y-auto p-2">
            <!-- 加载中：进度条 -->
            <div v-if="loading" class="py-12 px-4">
              <div class="flex flex-col items-center">
                <svg class="mb-4 h-8 w-8 animate-spin text-primary-500" viewBox="0 0 24 24" fill="none">
                  <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
                  <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
                </svg>
                <div class="w-full max-w-sm">
                  <div class="h-1.5 overflow-hidden rounded-full bg-gray-100">
                    <div
                      class="h-full rounded-full bg-gradient-to-r from-primary-400 to-primary-600 transition-all duration-300 ease-out"
                      :style="{ width: Math.min(100, percent) + '%' }"
                    />
                  </div>
                  <div class="mt-2 flex items-center justify-between text-xs">
                    <span class="text-gray-500">{{ stageText }}</span>
                    <span class="font-medium text-primary-600">{{ percent.toFixed(1) }}%</span>
                  </div>
                  <p v-if="slowMerging" class="mt-2 text-center text-[11px] text-gray-400">
                    资源有点多，稍安勿躁，静候处理
                  </p>
                </div>
              </div>
            </div>

            <!-- 版本分组卡片 -->
            <div v-else-if="groups.length > 0" class="space-y-1.5">
              <VersionGroupCard
                v-for="g in groups"
                :key="g.title"
                :title="g.title"
                :versions="g.versions"
                :expanded="expandedOf(g.title)"
                :mounted="mountedOf(g.title)"
                :downloading="downloading"
                :download-stage="downloadStage"
                :is-modpack="project.resource_type === 'ModPack'"
                :deps-map="depsMap"
                :deps-loading-set="depsLoadingSet"
                @toggle="toggleGroup(g.title)"
                @download="handleDownload"
                @install="handleInstallModpack"
                @load-deps="handleLoadDeps"
              />
            </div>

            <!-- 空状态 -->
            <div v-else class="py-12 flex flex-col items-center justify-center text-gray-400">
              <ArchiveBoxXMarkIcon class="w-10 h-10 mb-3" />
              <span class="text-sm">暂无版本数据</span>
            </div>

          </div>
        </div>
      </div>
    </transition>

    <!-- 前置 Mod 确认弹窗（独立 teleport，避免嵌套在详情弹窗内影响层级） -->
    <DependencyConfirmDialog
      :visible="showDependencyDialog"
      :missing="depsMissing"
      :up-to-date="depsUpToDate"
      :main-name="pendingMainVersion?.file_name || ''"
      :installing="depsInstalling"
      :checking="depsChecking"
      @close="handleDependencyClose"
      @confirm="handleDependencyConfirm"
    />
  </teleport>
</template>
