<script setup lang="ts">
/** 下载页面 */
import { ref, computed, watch, onMounted, onUnmounted, defineAsyncComponent } from 'vue'
import type { Component } from 'vue'
import { useVersionStore } from '@/stores/version'
import { useVersionSettings } from '@/composables/useVersionSettings'
import { backToTopEnabled } from '@/composables/useFloatingButtonState'
import { showError, showModal } from '@/utils/modal'
import { toastError, toastSuccess } from '@/utils/toast'
import { formatTimestamp } from '@/utils/format'
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const NavSidebar = defineAsyncComponent(() => import('@/components/common/NavSidebar.vue'))
const LoaderSelect = defineAsyncComponent(() => import('./LoaderSelect.vue'))
const RegionRestrictedPanel = defineAsyncComponent(() => import('@/components/downloads/RegionRestrictedPanel.vue'))
const VersionSection = defineAsyncComponent(() => import('@/components/version/VersionSection.vue'))
const Community = defineAsyncComponent(() => import('./Community.vue'))
import {
  CubeIcon, ArchiveBoxIcon,
  StarIcon, BeakerIcon, ClockIcon, SparklesIcon,
  ArrowPathIcon, FaceSmileIcon, FolderOpenIcon,
  PuzzlePieceIcon, SwatchIcon, BoltIcon, CircleStackIcon,
} from '@heroicons/vue/24/outline'
import { resolveVersionIcon as resolveIconByType } from '@/composables/useVersionMeta'
import { useVersionInstallActions, type InstallOptions } from '@/composables/useVersionInstallActions'
import { useRegion } from '@/composables/useRegion'
import type { ResourceType } from '@/types/community'

const versionStore = useVersionStore()
const { resolveVersionIconWithLogo: resolveVersionIcon } = useVersionSettings()

// 地区判定：受限地区（官网下发）+ 网络出口命中时，整页下载入口替换为限制说明
const {
  isRestricted, loadRegionInfo, notice, noticeShown, markNoticeShown, bypassed, setBypass,
} = useRegion()

const {
  installedVersions, installedVersionTypes, installedVersionLogos,
  handleRefresh, onInstallRequest,
  handleUninstall, handleOpenGameDir,
} = useVersionInstallActions()

const loading = ref(false)
const activeCategory = ref('vanilla')
const selectedVersion = ref<string | null>(null)

// 官方下载分类（仅保留原版游戏；模组加载器/整合包已由社区资源替代）
const communityCategories: { id: string; type: ResourceType; label: string; icon: Component }[] = [
  { id: 'community:Mod', type: 'Mod', label: 'Mod', icon: PuzzlePieceIcon },
  { id: 'community:ModPack', type: 'ModPack', label: '整合包', icon: ArchiveBoxIcon },
  { id: 'community:ResourcePack', type: 'ResourcePack', label: '资源包', icon: SwatchIcon },
  { id: 'community:Shader', type: 'Shader', label: '光影', icon: BoltIcon },
  { id: 'community:DataPack', type: 'DataPack', label: '数据包', icon: CircleStackIcon },
]

// 侧边栏分类（原版游戏 + 社区资源分组，供 NavSidebar 渲染）
const categories = computed(() => [
  { id: 'vanilla', label: '原版游戏', icon: CubeIcon },
  ...communityCategories.map((c) => ({ id: c.id, label: c.label, icon: c.icon, group: '社区资源' })),
])

const communityResourceType = computed<ResourceType | null>(() =>
  activeCategory.value.startsWith('community:')
    ? activeCategory.value.slice('community:'.length) as ResourceType : null
)

const headerTitle = computed(() => {
  if (isRestricted.value) return '游戏下载'
  return communityResourceType.value
    ? communityCategories.find(c => c.id === activeCategory.value)?.label || '社区资源'
    : '原版游戏'
})
const headerSubtitle = computed(() => {
  if (isRestricted.value) return notice.value?.title.trim() || '暂不提供游戏下载'
  return communityResourceType.value ? '从 CurseForge 和 Modrinth 搜索并安装' : 'Minecraft Java Edition 官方版本'
})

/**
 * 展示官网下发的下载限制提示（每次运行只弹一次）
 *
 * 按官网约定：点击确认只关闭提示，不解除下载限制。
 */
function showRestrictionNotice() {
  if (noticeShown.value) return
  const n = notice.value
  markNoticeShown()
  showModal({
    type: 'info',
    title: n?.title.trim() || '下载功能说明',
    message: n?.text.trim() || '当前地区暂不提供 Minecraft 下载功能。',
    confirmText: n?.confirmText.trim() || '我知道了',
  })
}

/** 恢复下载限制（开发者模式临时关闭后的一键回退） */
async function handleRestoreRestriction() {
  try {
    await setBypass(false)
    toastSuccess('已恢复下载限制')
  } catch (e) {
    toastError('恢复失败：' + String(e))
  }
}

function getVersionIcon(id: string, type: string): string {
  const logo = installedVersionLogos.value[id]
  if (logo) return resolveVersionIcon(logo, id)
  const actualType = installedVersionTypes.value[id] || type
  const normalized = (actualType === 'old_beta' || actualType === 'old_alpha') ? 'old' : actualType
  return resolveIconByType(normalized)
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

// 展开选择加载器页时禁用全局返回顶部按钮（避免残留遮挡右下角「开始安装」）
watch(selectedVersion, (v) => {
  backToTopEnabled.value = !v
})

onUnmounted(() => {
  backToTopEnabled.value = true
})

onMounted(async () => {
  // 地区判定期间先显示加载态，避免空白列表闪烁
  loading.value = true
  await loadRegionInfo()
  // 命中限制时不再拉取版本清单（避免无意义的元数据请求）
  if (isRestricted.value) {
    loading.value = false
    showRestrictionNotice()
    return
  }
  // 有缓存就不显示 loading
  if (versionStore.versions.length > 0) {
    loading.value = false
  }
  try {
    // 仅拉取版本清单，不扫描本地已安装版本（原版下载列表不依赖 installedVersions）
    await versionStore.fetchVersions()
  } catch (e) {
    showError('获取版本列表失败', String(e))
  }
  loading.value = false
})
</script>

<template>
  <div class="flex h-full rounded-xl overflow-hidden bg-white shadow-xs">
    <!-- 左侧菜单（公共组件，tab 同步到 URL query；底部插槽渲染「打开游戏目录」按钮） -->
    <NavSidebar :model-value="activeCategory" :categories="categories" @update:model-value="handleSelectCategory">
      <div class="p-3 border-t border-gray-200">
        <Button type="ghost" long @click="handleOpenGameDir">
          <template #icon><FolderOpenIcon class="w-4 h-4" /></template>
          打开游戏目录
        </Button>
      </div>
    </NavSidebar>

    <!-- 右侧内容 -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <div class="px-6 py-4 bg-white border-b border-gray-200 flex items-center justify-between shrink-0">
        <div>
          <h2 class="text-lg font-semibold text-gray-900">{{ headerTitle }}</h2>
          <p class="text-xs text-gray-500 mt-1">{{ headerSubtitle }}</p>
        </div>
        <Tooltip text="刷新版本列表" position="bottom">
          <Button
            type="ghost"
            size="small"
            :disabled="isRestricted || !!selectedVersion || activeCategory !== 'vanilla'"
            @click="!isRestricted && !selectedVersion && activeCategory === 'vanilla' && handleRefresh()"
          >
            <template #icon><ArrowPathIcon class="w-4 h-4" /></template>
          </Button>
        </Tooltip>
      </div>

      <!-- 开发者模式临时关闭限制：提示当前状态并提供一键恢复 -->
      <div
        v-if="bypassed"
        class="px-6 py-2 bg-amber-50 border-b border-amber-200 flex items-center justify-between shrink-0"
      >
        <span class="text-xs text-amber-700">
          开发者模式已临时关闭下载限制，重启启动器后按官网策略重新生效。
        </span>
        <Button type="ghost" size="small" @click="handleRestoreRestriction">恢复限制</Button>
      </div>

      <div class="flex-1 overflow-hidden">
        <!-- 地区限制：受限地区（官网下发）+ 网络出口命中时，替换整页下载内容 -->
        <RegionRestrictedPanel v-if="isRestricted" />

        <transition v-else name="slide-right" mode="out-in">
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
                :default-expanded="idx === 0" :format-date="(ts: number) => (ts ? formatTimestamp(ts, { withSeconds: false }) : '未知')"
                :get-version-icon="getVersionIcon"
                @download="selectedVersion = $event" @uninstall="handleUninstall"
              />
            </div>
          </div>
        </transition>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.slide-right-enter-active { transition: transform 0.3s ease-out, opacity 0.2s ease-out; }
.slide-right-leave-active { transition: transform 0.2s ease-in, opacity 0.15s ease-in; }
.slide-right-enter-from { transform: translateX(100%); opacity: 0; }
.slide-right-leave-to { transform: translateX(-30%); opacity: 0; }
</style>
