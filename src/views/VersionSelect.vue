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
import FolderSidebar from './version-select/FolderSidebar.vue'
import { safeCall } from '@/utils/async'
import {
  ArrowLeftIcon,
  ArrowPathIcon,
  CheckIcon,
  ArrowDownTrayIcon,
  ArchiveBoxIcon,
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
  versions: InstalledVersion[]
}
const groups = computed<VersionGroup[]>(() => {
  const map = new Map<string, VersionGroup>()
  for (const v of installed.value) {
    const meta = typeMeta(v.inferredType)
    if (!map.has(v.inferredType)) {
      map.set(v.inferredType, { key: v.inferredType, title: meta.groupTitle, versions: [] })
    }
    map.get(v.inferredType)!.versions.push(v)
  }
  return Array.from(map.values()).sort((a, b) => typeMeta(a.key).order - typeMeta(b.key).order)
})

const selectedId = computed({
  get: () => versionStore.selectedVersion,
  set: (val) => { versionStore.selectedVersion = val },
})

const hasVersions = computed(() => installed.value.length > 0)

/** 加载已安装版本列表 */
async function loadInstalled() {
  loading.value = true
  await safeCall(async () => {
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
    }
  }, 'load installed versions')
  loading.value = false
}

/** 选中版本并返回主页 */
function selectVersion(id: string) {
  selectedId.value = id
  router.push('/apps')
}

function goToDownloads() {
  router.push('/apps/downloads')
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

        <!-- 版本分组卡片（对齐 Settings 卡片规范：rounded-lg + border-gray-300 + 无灰底头 + space-y-6） -->
        <div v-else class="mx-auto max-w-3xl space-y-6">
          <section
            v-for="group in groups"
            :key="group.key"
            class="overflow-hidden rounded-lg border border-gray-300 bg-white"
          >
            <div class="flex items-center justify-between px-5 pt-5 pb-3">
              <h3 class="text-sm font-semibold text-gray-900">{{ group.title }}</h3>
              <span class="rounded-full bg-gray-100 px-2 py-0.5 text-xs font-medium text-gray-500">
                {{ group.versions.length }}
              </span>
            </div>
            <ul class="divide-y divide-gray-100">
              <li v-for="ver in group.versions" :key="ver.id">
                <button
                  class="flex w-full items-center gap-3 px-5 py-4 text-left transition-colors hover:bg-gray-50"
                  :class="{ 'bg-primary-50': ver.id === selectedId }"
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
          </section>
        </div>
      </main>
    </div>
  </div>
</template>
