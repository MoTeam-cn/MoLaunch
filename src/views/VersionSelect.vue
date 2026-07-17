<script setup lang="ts">
/**
 * 版本选择页（参考 PCL2 PageSelectLeft + PageSelectRight）
 *
 * 左侧：Minecraft 文件夹列表（可添加/删除/切换）
 * 右侧：当前文件夹下的版本列表（按类型分组卡片）
 *
 * - 点击版本项即选，选完返回主页
 * - 文件夹数据存储在程序配置 ini 的 [Folders] section
 */

import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useVersionStore } from '@/stores/version'
import { useVersionSettings } from '@/composables/useVersionSettings'
import * as tauri from '@/utils/tauri'
import { showSuccess, showWarning, showError } from '@/utils/toast'
import { showConfirm, showPrompt } from '@/utils/modal'
import grassIcon from '@/assets/blocks/Grass.png'
import { inferVersionType, typeMetaMap, type VersionTypeMeta } from '@/composables/useVersionMeta'

const router = useRouter()
const versionStore = useVersionStore()
const { resolveVersionIconWithLogo } = useVersionSettings()

interface InstalledVersion {
  id: string
  version_type: string
  inferredType: string
  logo: string
}

interface McFolder {
  name: string
  path: string
}

const installed = ref<InstalledVersion[]>([])
const folders = ref<McFolder[]>([])
const currentPath = ref<string>('')
const loading = ref(false)
const switchingFolder = ref(false)

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

/** 加载文件夹列表 */
async function loadFolders() {
  try {
    folders.value = await tauri.listMcFolders()
    // 读取当前选中的文件夹路径（复用 get_game_dir 命令，返回绝对路径）
    currentPath.value = await invoke<string>('get_game_dir')
  } catch (e) {
    console.error('Failed to load folders:', e)
  }
}

/** 切换文件夹 */
async function switchFolder(folder: McFolder) {
  if (switchingFolder.value) return
  if (folder.path === currentPath.value) return
  switchingFolder.value = true
  try {
    await tauri.switchMcFolder(folder.path)
    currentPath.value = folder.path
    await loadInstalled()
    showSuccess(`已切换到：${folder.name}`)
  } catch (e) {
    showError(String(e))
  } finally {
    switchingFolder.value = false
  }
}

/** 添加文件夹 */
async function addFolder() {
  try {
    // 复用已有的 select_folder 命令（调用系统文件夹选择对话框）
    const selected = await invoke<string | null>('select_folder')
    if (!selected) return

    // 推导默认名称：取最后一段，若为 .minecraft 则取父级
    const normalized = selected.replace(/[\\/]+$/, '')
    const parts = normalized.split(/[\\/]/)
    let defaultName = parts[parts.length - 1] || '文件夹'
    if (defaultName.toLowerCase() === '.minecraft' && parts.length >= 2) {
      defaultName = parts[parts.length - 2]
    }

    // 用自定义 prompt 弹窗输入名称（替代 window.prompt）
    showPrompt(
      '添加文件夹',
      '请输入文件夹显示名称：',
      async (name) => {
        if (!name.trim()) return
        try {
          folders.value = await tauri.addMcFolder(name.trim(), selected)
          showSuccess('文件夹已添加')
        } catch (e) {
          showError(String(e))
        }
      },
      { defaultValue: defaultName, placeholder: '文件夹名称' },
    )
  } catch (e) {
    showError(String(e))
  }
}

/** 移除文件夹 */
async function removeFolder(folder: McFolder, event: Event) {
  event.stopPropagation()
  if (folders.value.length <= 1) {
    showWarning('至少需要保留一个文件夹')
    return
  }
  showConfirm(
    '移除文件夹',
    `确定要移除文件夹"${folder.name}"吗？（不会删除实际文件）`,
    async () => {
      try {
        folders.value = await tauri.removeMcFolder(folder.path)
        // 如果移除的是当前文件夹，currentPath 已由后端自动切换
        currentPath.value = await invoke<string>('get_game_dir')
        await loadInstalled()
        showSuccess('文件夹已移除')
      } catch (e) {
        showError(String(e))
      }
    },
  )
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

onMounted(async () => {
  await loadFolders()
  await loadInstalled()
})
</script>

<template>
  <div class="flex h-full">
    <!-- 左侧：文件夹列表（参考 PCL2 PageSelectLeft） -->
    <aside class="flex w-64 flex-none flex-col border-r border-gray-200 bg-white">
      <!-- 滚动区 -->
      <div class="flex-1 overflow-y-auto px-3 pt-5">
        <!-- 分组标题 -->
        <div class="mb-1 px-2 text-xs font-medium text-gray-400">文件夹列表</div>
        <!-- 文件夹项 -->
        <ul class="space-y-0.5">
          <li v-for="folder in folders" :key="folder.path">
            <button
              class="group relative flex w-full items-center pl-3 pr-2 py-2.5 text-left transition-colors"
              :class="folder.path === currentPath
                ? 'bg-primary-50/70 text-primary-700'
                : 'text-gray-700 hover:bg-gray-50'"
              :disabled="switchingFolder"
              @click="switchFolder(folder)"
            >
              <!-- 选中时左侧高亮条（参考 PCL2 MyListItem RadioBox 样式） -->
              <span
                v-if="folder.path === currentPath"
                class="absolute left-0 top-1/2 h-5 w-0.5 -translate-y-1/2 rounded-full bg-primary-500"
              />
              <!-- 文件夹图标 -->
              <svg
                class="mr-2.5 h-4 w-4 flex-none"
                :class="folder.path === currentPath ? 'text-primary-500' : 'text-gray-400'"
                viewBox="0 0 20 20" fill="currentColor"
              >
                <path d="M2 6a2 2 0 012-2h4l2 2h6a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
              </svg>
              <!-- 名称 + 路径 -->
              <div class="min-w-0 flex-1">
                <div class="truncate text-sm font-medium">{{ folder.name }}</div>
                <div class="truncate text-xs text-gray-400">{{ folder.path }}</div>
              </div>
              <!-- hover 时显示的设置按钮（参考 PCL2 MyIconButton 齿轮） -->
              <svg
                v-if="folders.length > 1"
                class="ml-1 h-4 w-4 flex-none text-gray-400 opacity-0 transition-opacity hover:text-gray-600 group-hover:opacity-100"
                viewBox="0 0 20 20" fill="currentColor"
                @click="removeFolder(folder, $event)"
              >
                <path fill-rule="evenodd" d="M4.3 4.3a1 1 0 011.4 0L10 8.6l4.3-4.3a1 1 0 111.4 1.4L11.4 10l4.3 4.3a1 1 0 01-1.4 1.4L10 11.4l-4.3 4.3a1 1 0 01-1.4-1.4L8.6 10 4.3 5.7a1 1 0 010-1.4z" clip-rule="evenodd" />
              </svg>
            </button>
          </li>
        </ul>

        <!-- 分组标题：添加或导入（参考 PCL2） -->
        <div class="mb-1 mt-5 px-2 text-xs font-medium text-gray-400">添加或导入</div>
        <ul class="space-y-0.5">
          <li>
            <button
              class="flex w-full items-center rounded-md px-3 py-2 text-left text-sm text-gray-600 transition-colors hover:bg-gray-50 hover:text-primary-600"
              @click="addFolder"
            >
              <svg class="mr-2.5 h-4 w-4 flex-none text-gray-400" viewBox="0 0 20 20" fill="currentColor">
                <path d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" />
              </svg>
              添加已有文件夹
            </button>
          </li>
        </ul>
      </div>
    </aside>

    <!-- 右侧：版本列表 -->
    <div class="flex flex-1 flex-col overflow-hidden">
      <!-- 顶部栏 -->
      <header class="flex flex-none items-center justify-between border-b border-gray-200 bg-white px-4 py-3">
        <div class="flex items-center gap-3">
          <button
            class="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-sm text-gray-500 transition-colors hover:bg-gray-100 hover:text-gray-700"
            @click="goBack"
          >
            <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M12.7 4.3a1 1 0 010 1.4L8.4 10l4.3 4.3a1 1 0 01-1.4 1.4l-5-5a1 1 0 010-1.4l5-5a1 1 0 011.4 0z" clip-rule="evenodd" />
            </svg>
            返回
          </button>
          <h1 class="text-base font-semibold text-gray-800">选择版本</h1>
        </div>
        <button
          class="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-sm text-gray-500 transition-colors hover:bg-gray-100 hover:text-primary-600"
          :disabled="loading"
          @click="loadInstalled"
        >
          <svg class="h-4 w-4" :class="{ 'animate-spin': loading }" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M4 2a1 1 0 011 1v2.1a7 7 0 0111.6 2.5 1 1 0 11-1.88.7A5 5 0 005.9 6.4H8a1 1 0 010 2H3a1 1 0 01-1-1V3a1 1 0 011-1zm5.3 14.3a1 1 0 011.4 0l5-5a1 1 0 00-1.4-1.4L10 14.6l-2.3-2.3a1 1 0 00-1.4 1.4l3 3z" clip-rule="evenodd" />
          </svg>
          刷新
        </button>
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
            <button
              class="flex items-center gap-1.5 rounded-lg bg-primary-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-700"
              @click="goToDownloads"
            >
              <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                <path d="M10 3a1 1 0 011 1v6.6l2.3-2.3a1 1 0 111.4 1.4l-4 4a1 1 0 01-1.4 0l-4-4a1 1 0 111.4-1.4L9 10.6V4a1 1 0 011-1z" />
                <path d="M4 14a1 1 0 011 1v1h10v-1a1 1 0 112 0v2a1 1 0 01-1 1H4a1 1 0 01-1-1v-2a1 1 0 011-1z" />
              </svg>
              下载游戏
            </button>
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
