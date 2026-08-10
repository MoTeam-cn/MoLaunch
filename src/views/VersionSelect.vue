<script setup lang="ts">
/**
 * 版本选择页（左右分栏布局）
 *
 * 左侧：Minecraft 文件夹列表（FolderSidebar 子组件，可添加/删除/切换）
 * 右侧：当前文件夹下的版本列表（按类型分组卡片）
 *
 * - 点击版本项即选，选完返回主页
 * - 文件夹切换后由 FolderSidebar 触发 @switched 事件，本组件重新加载版本列表
 */
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useVersionStore } from '@/stores/version'
import { useVersionSettings } from '@/composables/useVersionSettings'
import * as tauri from '@/utils/tauri'
import grassIcon from '@/assets/blocks/Grass.png'
import { inferVersionType, typeMetaMap, type VersionTypeMeta } from '@/composables/useVersionMeta'
import Button from '@/components/common/Button.vue'
import Tag from '@/components/common/Tag.vue'
import FolderSidebar from './version-select/FolderSidebar.vue'
import { toastSuccess, toastError, toastInfo } from '@/utils/toast'
import {
  ArrowLeftIcon,
  ArrowPathIcon,
  CheckIcon,
  ArrowDownTrayIcon,
  ArchiveBoxIcon,
  ChevronDownIcon,
} from '@heroicons/vue/24/outline'

const router = useRouter()
const versionStore = useVersionStore()
const { resolveVersionIconWithLogo } = useVersionSettings()

interface InstalledVersion {
  id: string
  version_type: string
  inferredType: string
  logo: string
}

const installed = ref<InstalledVersion[]>([])
const loading = ref(false)

const defaultMeta: VersionTypeMeta = { icon: grassIcon, label: '其他', groupTitle: '其他版本', order: 99 }

function typeMeta(type: string): VersionTypeMeta {
  return typeMetaMap[type] ?? defaultMeta
}

interface VersionGroup {
  key: string
  title: string
  icon: string
  versions: InstalledVersion[]
}
const groups = computed<VersionGroup[]>(() => {
  const map = new Map<string, VersionGroup>()
  for (const v of installed.value) {
    const meta = typeMeta(v.inferredType)
    if (!map.has(v.inferredType)) {
      map.set(v.inferredType, { key: v.inferredType, title: meta.groupTitle, icon: meta.icon, versions: [] })
    }
    map.get(v.inferredType)!.versions.push(v)
  }
  return Array.from(map.values()).sort((a, b) => typeMeta(a.key).order - typeMeta(b.key).order)
})

// 分组折叠状态（默认全部收起，点击标题栏展开）
// 用 expandedKeys 跟踪已展开的分组，空集合 = 全部收起
const expandedKeys = ref<Set<string>>(new Set())
function toggleGroup(key: string) {
  const s = new Set(expandedKeys.value)
  if (s.has(key)) s.delete(key)
  else s.add(key)
  expandedKeys.value = s
}
function isCollapsed(key: string) {
  return !expandedKeys.value.has(key)
}

const selectedId = computed({
  get: () => versionStore.selectedVersion,
  set: (val) => { versionStore.selectedVersion = val },
})

const hasVersions = computed(() => installed.value.length > 0)

/** 加载已安装版本列表 */
async function loadInstalled() {
  loading.value = true
  toastInfo('正在刷新版本列表...')
  try {
    const list = await tauri.listInstalledVersionsWithType()
    installed.value = list.map(v => ({
      id: v.id,
      version_type: v.version_type,
      inferredType: inferVersionType(v.id, undefined, v.version_type),
      logo: v.logo || '',
    }))
    if (installed.value.length > 0) {
      const exists = installed.value.some(v => v.id === selectedId.value)
      if (!exists) selectedId.value = installed.value[0].id
      // 自动展开当前选中版本所在的分组（用户进入页面即可看到选中项）
      const selectedVer = installed.value.find(v => v.id === selectedId.value)
      if (selectedVer) {
        const s = new Set(expandedKeys.value)
        s.add(selectedVer.inferredType)
        expandedKeys.value = s
      }
    }
    toastSuccess('版本列表已刷新')
  } catch (e) {
    console.error('Failed to load installed versions:', e)
    toastError('刷新版本列表失败')
  } finally {
    loading.value = false
  }
}

/** 选中版本并返回主页 */
function selectVersion(id: string) {
  selectedId.value = id
  router.push('/apps')
}

/** 空状态"下载游戏"按钮：进入下载页面（原版/社区资源安装），而非下载管理 */
function goToDownloads() {
  router.push('/apps/versions')
}

function goBack() {
  router.push('/apps')
}

onMounted(() => loadInstalled())
</script>

<template>
  <div class="flex h-full rounded-xl overflow-hidden bg-white shadow-sm">
    <!-- 左侧：文件夹列表（FolderSidebar 子组件） -->
    <FolderSidebar @switched="loadInstalled" />

    <!-- 右侧：版本列表 -->
    <div class="flex flex-1 flex-col overflow-hidden">
      <!-- 顶部栏（对齐 Settings 标题栏规范：px-6 py-4 + text-lg + text-gray-900） -->
      <header class="flex flex-none items-center justify-between border-b border-gray-200 bg-white px-6 py-4 shrink-0">
        <div class="flex items-center gap-3">
          <Button
            type="ghost"
            size="small"
            @click="goBack"
          >
            <template #icon>
              <ArrowLeftIcon class="h-4 w-4" />
            </template>
            返回
          </Button>
          <h2 class="text-lg font-semibold text-gray-900">选择版本</h2>
        </div>
        <Button
          type="ghost"
          size="small"
          :disabled="loading"
          @click="loadInstalled"
        >
          <template #icon>
            <ArrowPathIcon class="h-4 w-4" :class="{ 'animate-spin': loading }" />
          </template>
          刷新
        </Button>
      </header>

      <!-- 主体（对齐 Settings 内容区规范：p-6） -->
      <main class="flex-1 overflow-y-auto p-6">
        <!-- 加载中 -->
        <div v-if="loading && !hasVersions" class="flex h-full items-center justify-center">
          <div class="flex flex-col items-center gap-3 text-gray-400">
            <ArrowPathIcon class="h-8 w-8 animate-spin" />
            <span class="text-sm">正在获取版本列表...</span>
          </div>
        </div>

        <!-- 空状态 -->
        <div v-else-if="!hasVersions" class="flex h-full items-center justify-center">
          <div class="flex flex-col items-center gap-4 rounded-lg border border-gray-200 bg-white px-12 py-10 text-center shadow-sm">
            <ArchiveBoxIcon class="h-12 w-12 text-gray-300" />
            <div>
              <p class="text-base font-medium text-gray-700">无可用版本</p>
              <p class="mt-1 text-sm text-gray-400">当前文件夹下未找到任何已安装的版本</p>
            </div>
            <Button
              type="primary"
              @click="goToDownloads"
            >
              <template #icon>
                <ArrowDownTrayIcon class="h-4 w-4" />
              </template>
              下载游戏
            </Button>
          </div>
        </div>

        <!-- 版本分组卡片（可折叠，默认收起，grid-rows 0fr→1fr 平滑动画） -->
        <div v-else class="mx-auto max-w-3xl space-y-6">
          <section
            v-for="group in groups"
            :key="group.key"
            class="overflow-hidden rounded-lg border border-gray-300 bg-white"
          >
            <!-- 标题栏（点击展开/折叠，对齐 MoLaunchIntro.vue 样式规范） -->
            <!-- 保留原生 button：分组折叠头为 w-full justify-between 布局 + 右侧 chevron 图标，
                 Button.vue 的 svg margin 与居中布局不适合折叠头/列表项布局 -->
            <button
              type="button"
              class="flex w-full items-center justify-between px-4 py-3 text-left transition-colors hover:bg-gray-50"
              :aria-expanded="!isCollapsed(group.key)"
              @click="toggleGroup(group.key)"
            >
              <div class="flex items-center gap-2.5">
                <img :src="group.icon" class="h-5 w-5 rounded-sm" alt="">
                <span class="text-sm font-semibold text-gray-900">{{ group.title }}</span>
                <Tag size="small" color="gray">{{ group.versions.length }}</Tag>
              </div>
              <ChevronDownIcon
                class="h-4 w-4 flex-none text-gray-400 transition-transform duration-300 ease-in-out"
                :class="isCollapsed(group.key) ? '' : 'rotate-180'"
              />
            </button>
            <!-- 内容区（grid-template-rows 0fr→1fr 平滑高度过渡，与 MoLaunchIntro.vue 一致） -->
            <div
              class="grid transition-all duration-300 ease-in-out"
              :class="isCollapsed(group.key) ? 'grid-rows-[0fr]' : 'grid-rows-[1fr]'"
            >
              <div class="overflow-hidden">
                <ul class="divide-y divide-gray-100 border-t border-gray-100">
                  <li v-for="ver in group.versions" :key="ver.id">
                    <!-- 保留原生 button：版本列表项（w-full + active 状态 + 左侧色条），
                         Button.vue 的 scoped size 类无法承载列表项布局 -->
                    <button
                      class="flex w-full items-center gap-3 px-4 py-3 text-left transition-colors hover:bg-gray-50"
                      :class="ver.id === selectedId
                        ? 'border-l-2 border-primary-500 bg-primary-50/50'
                        : 'border-l-2 border-transparent'"
                      @click="selectVersion(ver.id)"
                    >
                      <img :src="resolveVersionIconWithLogo(ver.logo, ver.id, ver.version_type)" class="h-8 w-8 flex-none rounded" alt="">
                      <div class="min-w-0 flex-1">
                        <div class="truncate text-sm font-medium text-gray-900">{{ ver.id }}</div>
                        <div class="mt-0.5 text-xs text-gray-400">{{ typeMeta(ver.inferredType).label }}</div>
                      </div>
                      <CheckIcon v-if="ver.id === selectedId" class="h-5 w-5 flex-none text-primary-500" />
                    </button>
                  </li>
                </ul>
              </div>
            </div>
          </section>
        </div>
      </main>
    </div>
  </div>
</template>
