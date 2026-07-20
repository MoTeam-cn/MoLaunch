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
    }
  } catch (e) {
    console.error('Failed to load installed versions:', e)
  } finally {
    loading.value = false
  }
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
  <div class="flex h-full">
    <!-- 左侧：文件夹列表（FolderSidebar 子组件） -->
    <FolderSidebar @switched="loadInstalled" />

    <!-- 右侧：版本列表 -->
    <div class="flex flex-1 flex-col overflow-hidden">
      <!-- 顶部栏 -->
      <header class="flex flex-none items-center justify-between border-b border-gray-200 bg-white px-4 py-3">
        <div class="flex items-center gap-3">
          <Button
            type="ghost"
            size="small"
            @click="goBack"
          >
            <template #icon>
              <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M12.7 4.3a1 1 0 010 1.4L8.4 10l4.3 4.3a1 1 0 01-1.4 1.4l-5-5a1 1 0 010-1.4l5-5a1 1 0 011.4 0z" clip-rule="evenodd" />
              </svg>
            </template>
            返回
          </Button>
          <h1 class="text-base font-semibold text-gray-800">选择版本</h1>
        </div>
        <Button
          type="ghost"
          size="small"
          :disabled="loading"
          @click="loadInstalled"
        >
          <template #icon>
            <svg class="h-4 w-4" :class="{ 'animate-spin': loading }" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M4 2a1 1 0 011 1v2.1a7 7 0 0111.6 2.5 1 1 0 11-1.88.7A5 5 0 005.9 6.4H8a1 1 0 010 2H3a1 1 0 01-1-1V3a1 1 0 011-1zm5.3 14.3a1 1 0 011.4 0l5-5a1 1 0 00-1.4-1.4L10 14.6l-2.3-2.3a1 1 0 00-1.4 1.4l3 3z" clip-rule="evenodd" />
            </svg>
          </template>
          刷新
        </Button>
      </header>

      <!-- 主体 -->
      <main class="flex-1 overflow-y-auto p-4">
        <!-- 加载中 -->
        <div v-if="loading && !hasVersions" class="flex h-full items-center justify-center">
          <div class="flex flex-col items-center gap-3 text-gray-400">
            <svg class="h-8 w-8 animate-spin" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
              <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
            </svg>
            <span class="text-sm">正在获取版本列表...</span>
          </div>
        </div>

        <!-- 空状态 -->
        <div v-else-if="!hasVersions" class="flex h-full items-center justify-center">
          <div class="flex flex-col items-center gap-4 rounded-2xl border border-gray-200 bg-white px-12 py-10 text-center shadow-sm">
            <svg class="h-12 w-12 text-gray-300" viewBox="0 0 24 24" fill="currentColor">
              <path d="M4 4a2 2 0 012-2h12a2 2 0 012 2v16a2 2 0 01-2 2H6a2 2 0 01-2-2V4zm2 0v4h12V4H6zm0 6v4h12v-4H6zm0 6v4h12v-4H6z" />
            </svg>
            <div>
              <p class="text-base font-medium text-gray-700">无可用版本</p>
              <p class="mt-1 text-sm text-gray-400">当前文件夹下未找到任何已安装的版本</p>
            </div>
            <Button
              type="primary"
              @click="goToDownloads"
            >
              <template #icon>
                <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                  <path d="M10 3a1 1 0 011 1v6.6l2.3-2.3a1 1 0 111.4 1.4l-4 4a1 1 0 01-1.4 0l-4-4a1 1 0 111.4-1.4L9 10.6V4a1 1 0 011-1z" />
                  <path d="M4 14a1 1 0 011 1v1h10v-1a1 1 0 112 0v2a1 1 0 01-1 1H4a1 1 0 01-1-1v-2a1 1 0 011-1z" />
                </svg>
              </template>
              下载游戏
            </Button>
          </div>
        </div>

        <!-- 版本分组卡片 -->
        <div v-else class="mx-auto max-w-3xl space-y-4">
          <section
            v-for="group in groups"
            :key="group.key"
            class="overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm"
          >
            <div class="flex items-center justify-between border-b border-gray-100 bg-gray-50/60 px-4 py-2.5">
              <h2 class="text-sm font-semibold text-gray-700">{{ group.title }}</h2>
              <span class="rounded-full bg-gray-200/70 px-2 py-0.5 text-xs font-medium text-gray-500">
                {{ group.versions.length }}
              </span>
            </div>
            <ul class="divide-y divide-gray-50">
              <li v-for="ver in group.versions" :key="ver.id">
                <button
                  class="flex w-full items-center gap-3 px-4 py-3 text-left transition-colors hover:bg-primary-50/40"
                  :class="{ 'bg-primary-50': ver.id === selectedId }"
                  @click="selectVersion(ver.id)"
                >
                  <img :src="resolveVersionIconWithLogo(ver.logo, ver.id, ver.version_type)" class="h-8 w-8 flex-none rounded" alt="">
                  <div class="min-w-0 flex-1">
                    <div class="truncate text-sm font-medium text-gray-900">{{ ver.id }}</div>
                    <div class="mt-0.5 text-xs text-gray-400">{{ typeMeta(ver.inferredType).label }}</div>
                  </div>
                  <svg v-if="ver.id === selectedId" class="h-5 w-5 flex-none text-primary-600" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M16.7 5.3a1 1 0 010 1.4l-7.5 7.5a1 1 0 01-1.4 0L3.3 9.7a1 1 0 011.4-1.4l3.8 3.8 6.8-6.8a1 1 0 011.4 0z" clip-rule="evenodd" />
                  </svg>
                </button>
              </li>
            </ul>
          </section>
        </div>
      </main>
    </div>
  </div>
</template>
