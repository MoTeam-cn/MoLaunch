<script setup lang="ts">
/** 下载页面 */
import { ref, computed, onMounted } from 'vue'
import { useVersionStore } from '@/stores/version'
import { useVersionSettings } from '@/composables/useVersionSettings'
import { showError } from '@/utils/modal'
import Tooltip from '@/components/common/Tooltip.vue'
import LoaderSelect from './LoaderSelect.vue'
import VersionSection from '@/components/version/VersionSection.vue'
import Community from './Community.vue'
import DownloadSidebar from './downloads/DownloadSidebar.vue'
import {
  CubeIcon, WrenchIcon, ArchiveBoxIcon,
  StarIcon, BeakerIcon, ClockIcon, SparklesIcon,
  ArrowPathIcon, FaceSmileIcon,
  PuzzlePieceIcon, SwatchIcon, BoltIcon, CircleStackIcon,
} from '@heroicons/vue/24/outline'
import { resolveVersionIcon as resolveIconByType } from '@/composables/useVersionMeta'
import { useVersionInstallActions, type InstallOptions } from '@/composables/useVersionInstallActions'
import type { ResourceType } from '@/types/community'

const versionStore = useVersionStore()
const { resolveVersionIconWithLogo: resolveVersionIcon } = useVersionSettings()

const {
  installedVersions, installedVersionTypes, installedVersionLogos,
  loadInstalledVersions, handleRefresh, onInstallRequest,
  handleUninstall, handleOpenGameDir,
} = useVersionInstallActions()

const loading = ref(false)
const activeCategory = ref('vanilla')
const selectedVersion = ref<string | null>(null)

const topCategories = [
  { id: 'vanilla', label: '原版游戏', icon: CubeIcon },
  { id: 'modloaders', label: '模组加载器', icon: WrenchIcon },
  { id: 'modpacks', label: '整合包', icon: ArchiveBoxIcon },
]

const communityCategories: { id: string; type: ResourceType; label: string; icon: any }[] = [
  { id: 'community:Mod', type: 'Mod', label: 'Mod', icon: PuzzlePieceIcon },
  { id: 'community:ModPack', type: 'ModPack', label: '整合包', icon: ArchiveBoxIcon },
  { id: 'community:ResourcePack', type: 'ResourcePack', label: '资源包', icon: SwatchIcon },
  { id: 'community:Shader', type: 'Shader', label: '光影', icon: BoltIcon },
  { id: 'community:DataPack', type: 'DataPack', label: '数据包', icon: CircleStackIcon },
]

const communityResourceType = computed<ResourceType | null>(() =>
  activeCategory.value.startsWith('community:')
    ? activeCategory.value.slice('community:'.length) as ResourceType : null
)

const headerTitle = computed(() =>
  communityResourceType.value
    ? communityCategories.find(c => c.id === activeCategory.value)?.label || '社区资源'
    : '原版游戏'
)
const headerSubtitle = computed(() =>
  communityResourceType.value ? '从 CurseForge 和 Modrinth 搜索并安装' : 'Minecraft Java Edition 官方版本'
)

function getVersionIcon(id: string, type: string): string {
  const logo = installedVersionLogos.value[id]
  if (logo) return resolveVersionIcon(logo, id)
  const actualType = installedVersionTypes.value[id] || type
  const normalized = (actualType === 'old_beta' || actualType === 'old_alpha') ? 'old' : actualType
  return resolveIconByType(normalized)
}

function formatDate(ts: number): string {
  if (!ts) return '未知'
  return new Date(ts * 1000).toLocaleString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
}

const latestVersions = computed(() => {
  const list = []
  if (versionStore.latestRelease) {
    const v = versionStore.getVersionById(versionStore.latestRelease)
    if (v) list.push({ ...v, tag: '正式版' })
  }
  if (versionStore.latestSnapshot) {
    const v = versionStore.getVersionById(versionStore.latestSnapshot)
    if (v) list.push({ ...v, tag: '快照版' })
  }
  return list
})

const sections = computed(() => [
  { id: 'latest', label: '最新版本', icon: SparklesIcon, versions: latestVersions.value },
  { id: 'release', label: '正式版', icon: StarIcon, versions: versionStore.getReleaseVersions() },
  { id: 'snapshot', label: '快照版', icon: BeakerIcon, versions: versionStore.getSnapshotVersions() },
  { id: 'fool', label: '愚人节版', icon: FaceSmileIcon, versions: versionStore.versions.filter(v => v.version_type === 'fool') },
  { id: 'old', label: '远古版', icon: ClockIcon, versions: versionStore.versions.filter(v => v.version_type === 'old_beta' || v.version_type === 'old_alpha') },
])

/** 安装请求：清空 LoaderSelect 展开后委托 composable 执行后台安装流程 */
function handleInstallRequest(options: InstallOptions) {
  // 立刻返回版本列表（仅清空本页面的 LoaderSelect 展开，不影响首页启动用选中版本）
  selectedVersion.value = null
  onInstallRequest(options)
}

/** 选择分类：切换分类并清空 LoaderSelect 展开 */
function handleSelectCategory(category: string) {
  activeCategory.value = category
  selectedVersion.value = null
}

onMounted(async () => {
  // 有缓存就不显示 loading
  if (versionStore.versions.length === 0) {
    loading.value = true
  }
  try {
    await Promise.all([versionStore.fetchVersions(), loadInstalledVersions()])
  } catch (e) {
    showError('获取版本列表失败', String(e))
  }
  loading.value = false
})
</script>

<template>
  <div class="flex h-full rounded-xl overflow-hidden bg-white shadow-sm">
    <!-- 左侧菜单 -->
    <DownloadSidebar
      :top-categories="topCategories"
      :community-categories="communityCategories"
      :active-category="activeCategory"
      @select="handleSelectCategory"
      @open-game-dir="handleOpenGameDir"
    />

    <!-- 右侧内容 -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <div class="px-6 py-4 bg-white border-b border-gray-200 flex items-center justify-between shrink-0">
        <div>
          <h2 class="text-lg font-semibold text-gray-900">{{ headerTitle }}</h2>
          <p class="text-xs text-gray-500 mt-1">{{ headerSubtitle }}</p>
        </div>
        <Tooltip text="刷新版本列表" position="bottom">
          <button
            class="p-2 rounded-lg transition-colors"
            :class="(selectedVersion || activeCategory !== 'vanilla') ? 'text-gray-300 cursor-not-allowed' : 'text-gray-500 hover:text-gray-700 hover:bg-gray-100'"
            :disabled="!!selectedVersion || activeCategory !== 'vanilla'"
            @click="!selectedVersion && activeCategory === 'vanilla' && handleRefresh()"
          >
            <ArrowPathIcon class="w-4 h-4" />
          </button>
        </Tooltip>
      </div>

      <div class="flex-1 overflow-hidden">
        <transition name="slide-right" mode="out-in">
          <LoaderSelect
            v-if="selectedVersion"
            :key="'loader-' + selectedVersion"
            :mc-version="selectedVersion"
            @back="selectedVersion = null"
            @install="handleInstallRequest"
          />

          <!-- 社区资源：全高度，无 padding -->
          <Community
            v-else-if="communityResourceType"
            :key="activeCategory"
            :resource-type="communityResourceType"
          />

          <div v-else key="version-list" class="p-6 h-full overflow-y-auto">
            <!-- 原版游戏 - 加载中 -->
            <div v-if="activeCategory === 'vanilla' && loading" class="flex items-center justify-center h-full">
              <div class="text-center">
                <div class="animate-spin rounded-full h-10 w-10 border-2 border-gray-200 border-t-primary-600 mx-auto"></div>
                <p class="text-sm text-gray-500 mt-3">加载中...</p>
              </div>
            </div>

            <!-- 原版游戏 - 列表 -->
            <div v-else-if="activeCategory === 'vanilla'" class="space-y-4">
              <VersionSection
                v-for="(section, idx) in sections" :id="section.id" :key="section.id"
                :label="section.label" :icon="section.icon" :versions="section.versions"
                :installed-ids="installedVersions" :downloading="versionStore.downloading"
                :default-expanded="idx === 0" :format-date="formatDate"
                :get-version-icon="getVersionIcon"
                @download="selectedVersion = $event" @uninstall="handleUninstall"
              />
            </div>

            <!-- 模组加载器/整合包 - 待实现 -->
            <div v-else class="flex items-center justify-center h-full">
              <div class="text-center">
                <CubeIcon class="w-16 h-16 text-gray-400 mx-auto" />
                <p class="text-gray-600 mt-4">即将开放</p>
              </div>
            </div>
          </div>
        </transition>
      </div>
    </div>
  </div>
</template>

<style scoped>
.slide-right-enter-active { transition: transform 0.3s ease-out, opacity 0.2s ease-out; }
.slide-right-leave-active { transition: transform 0.2s ease-in, opacity 0.15s ease-in; }
.slide-right-enter-from { transform: translateX(100%); opacity: 0; }
.slide-right-leave-to { transform: translateX(-30%); opacity: 0; }
</style>
