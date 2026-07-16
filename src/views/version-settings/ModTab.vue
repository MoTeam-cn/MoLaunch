<script setup lang="ts">
/**
 * 版本设置 - Mod 管理子页
 * 列表、筛选、启用/禁用、安装、删除
 * 原版不支持 Mod 时显示提示
 */
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import * as tauri from '@/utils/tauri'
import { showSuccess, showError } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { useVersionSettings } from '@/composables/useVersionSettings'
import { formatBytes } from '@/utils/format'

const router = useRouter()
const { selectedId, isModable } = useVersionSettings()

const mods = ref<tauri.ModInfo[]>([])
const modsLoading = ref(false)
const modFilter = ref<'all' | 'enabled' | 'disabled'>('all')
const modSearch = ref('')
const isModableVersion = ref(false)
const checkingModable = ref(false)
// Mod 管理样式：0=标题显示译名，详情显示文件名；1=标题显示文件名，详情显示译名
const modLocalNameStyle = ref(0)

async function checkModable() {
  if (!selectedId.value) {
    isModableVersion.value = false
    return
  }
  checkingModable.value = true
  try {
    isModableVersion.value = await tauri.isVersionModable(selectedId.value)
  } catch {
    isModableVersion.value = isModable.value
  } finally {
    checkingModable.value = false
  }
}

async function loadMods() {
  if (!selectedId.value) return
  modsLoading.value = true
  try {
    mods.value = await tauri.listMods(selectedId.value)
  } catch (e) {
    showError('加载 Mod 列表失败：' + String(e))
    mods.value = []
  } finally {
    modsLoading.value = false
  }
}

const filteredMods = computed(() => {
  let list = mods.value
  if (modFilter.value === 'enabled') list = list.filter(m => m.is_enabled)
  else if (modFilter.value === 'disabled') list = list.filter(m => !m.is_enabled)
  if (modSearch.value.trim()) {
    const q = modSearch.value.toLowerCase()
    list = list.filter(m =>
      m.enabled_name.toLowerCase().includes(q) ||
      m.translated_name.toLowerCase().includes(q),
    )
  }
  return list
})

/** 根据 modLocalNameStyle 返回 Mod 标题（主显示名） */
function modTitle(mod: tauri.ModInfo): string {
  // 0 = 标题显示译名，1 = 标题显示文件名
  if (modLocalNameStyle.value === 0) {
    return mod.translated_name || mod.enabled_name
  }
  return mod.enabled_name
}

/** 根据 modLocalNameStyle 返回 Mod 副标题（详情名） */
function modSubtitle(mod: tauri.ModInfo): string {
  // 0 = 详情显示文件名，1 = 详情显示译名
  if (modLocalNameStyle.value === 0) {
    return mod.enabled_name
  }
  return mod.translated_name
}

async function handleToggleMod(mod: tauri.ModInfo) {
  if (!selectedId.value) return
  try {
    await tauri.toggleMod(selectedId.value, mod.file_name, !mod.is_enabled)
    showSuccess(mod.is_enabled ? '已禁用' : '已启用')
    await loadMods()
  } catch (e) {
    showError('操作失败：' + String(e))
  }
}

function handleDeleteMod(mod: tauri.ModInfo) {
  if (!selectedId.value) return
  showConfirm(
    '删除 Mod',
    `确定要删除 "${modTitle(mod)}" 吗？此操作不可恢复。`,
    async () => {
      try {
        await tauri.deleteMod(selectedId.value!, mod.file_name)
        showSuccess('Mod 已删除')
        await loadMods()
      } catch (e) {
        showError('删除失败：' + String(e))
      }
    },
  )
}

async function handleInstallMod() {
  if (!selectedId.value) return
  try {
    const files = await tauri.selectFile('选择要安装的 Mod', [
      { name: 'Mod 文件', extensions: ['jar', 'litemod', 'disabled', 'old'] },
    ])
    if (!files) return
    await tauri.installMod(selectedId.value, files)
    showSuccess('Mod 安装成功')
    await loadMods()
  } catch (e) {
    showError('安装失败：' + String(e))
  }
}

async function handleOpenModsDir() {
  if (!selectedId.value) return
  try {
    await tauri.openModsDir(selectedId.value)
  } catch (e) {
    showError('打开文件夹失败：' + String(e))
  }
}

onMounted(async () => {
  // 读取 Mod 管理样式配置
  try {
    const cfg = await tauri.getConfigMap()
    modLocalNameStyle.value = cfg.communityModLocalNameStyle
  } catch { /* 默认 0 */ }
  await checkModable()
  if (isModableVersion.value) await loadMods()
})
</script>

<template>
  <div class="space-y-4">
    <!-- 不可安装 Mod 的提示 -->
    <div v-if="!isModableVersion && !checkingModable" class="flex items-center justify-center py-12">
      <div class="rounded-xl border border-gray-200 bg-white p-8 text-center shadow-sm">
        <div class="mb-3 text-lg font-semibold text-gray-700">该版本不可使用 Mod</div>
        <div class="mx-auto mb-5 h-0.5 w-12 bg-gray-300"></div>
        <p class="mb-5 text-sm text-gray-500">
          你需要先安装 Forge、Fabric 等 Mod 加载器才能使用 Mod，请在下载页面安装这些版本。<br>
          如果你已经安装过 Mod 加载器，可能是版本选择有误，请切换版本。
        </p>
        <div class="flex justify-center gap-3">
          <button
            class="rounded-lg bg-primary-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-700"
            @click="router.push('/apps/downloads')"
          >
            转到下载页面
          </button>
          <button
            class="rounded-lg border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 transition-colors hover:bg-gray-50"
            @click="router.push('/apps/versions/select')"
          >
            版本选择
          </button>
        </div>
      </div>
    </div>

    <!-- Mod 管理主体 -->
    <div v-else>
      <!-- 顶部工具栏 -->
      <section class="rounded-xl border border-gray-200 bg-white p-3 shadow-sm">
        <div class="flex flex-wrap items-center gap-2">
          <button
            class="flex items-center gap-1.5 rounded-lg bg-primary-600 px-3 py-1.5 text-xs text-white transition-colors hover:bg-primary-700"
            @click="handleInstallMod"
          >
            <svg class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor"><path d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" /></svg>
            从文件安装
          </button>
          <button
            class="flex items-center gap-1.5 rounded-lg border border-gray-300 bg-white px-3 py-1.5 text-xs text-gray-700 transition-colors hover:bg-gray-50"
            @click="handleOpenModsDir"
          >
            <svg class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor"><path d="M2 6a2 2 0 012-2h4l2 2h6a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" /></svg>
            打开文件夹
          </button>
          <button
            class="flex items-center gap-1.5 rounded-lg border border-gray-300 bg-white px-3 py-1.5 text-xs text-gray-700 transition-colors hover:bg-gray-50"
            @click="loadMods"
          >
            <svg class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z" clip-rule="evenodd" /></svg>
            刷新
          </button>

          <div class="ml-auto flex items-center gap-1">
            <button
              v-for="opt in [
                { v: 'all', l: '全部' },
                { v: 'enabled', l: '已启用' },
                { v: 'disabled', l: '已禁用' },
              ]"
              :key="opt.v"
              class="rounded-md px-2.5 py-1 text-xs font-medium transition-colors"
              :class="modFilter === opt.v ? 'bg-primary-100 text-primary-700' : 'text-gray-500 hover:bg-gray-100'"
              @click="modFilter = opt.v as 'all' | 'enabled' | 'disabled'"
            >
              {{ opt.l }}
            </button>
          </div>

          <input
            v-model="modSearch"
            type="text"
            placeholder="搜索 Mod 名称"
            class="w-48 rounded-md border border-gray-300 px-3 py-1 text-xs focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          >
        </div>
      </section>

      <!-- 加载中 -->
      <div v-if="modsLoading" class="flex items-center justify-center py-12 text-sm text-gray-400">
        <svg class="mr-2 h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
          <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
        </svg>
        正在加载 Mod 列表...
      </div>

      <!-- 空列表 -->
      <div v-else-if="filteredMods.length === 0" class="flex items-center justify-center py-12">
        <div class="text-center">
          <div class="mb-2 text-base font-medium text-gray-500">
            {{ mods.length === 0 ? '尚未安装 Mod' : '没有符合条件的 Mod' }}
          </div>
          <p v-if="mods.length === 0" class="mb-4 text-xs text-gray-400">
            你可以从文件安装 Mod，或下载新 Mod
          </p>
          <button
            v-if="mods.length === 0"
            class="rounded-lg bg-primary-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-700"
            @click="handleInstallMod"
          >
            从文件安装 Mod
          </button>
        </div>
      </div>

      <!-- Mod 列表 -->
      <div v-else class="space-y-2">
        <div
          v-for="mod in filteredMods"
          :key="mod.file_name"
          class="flex items-center gap-3 rounded-lg border border-gray-200 bg-white p-3 shadow-sm transition-colors hover:border-gray-300"
          :class="{ 'opacity-60': !mod.is_enabled }"
        >
          <div
            class="flex h-10 w-10 flex-none items-center justify-center rounded-md text-xs font-medium"
            :class="mod.is_enabled ? 'bg-primary-50 text-primary-600' : 'bg-gray-100 text-gray-400'"
          >
            {{ mod.loader_type === 'unknown' ? 'M' : mod.loader_type.charAt(0).toUpperCase() }}
          </div>
          <div class="min-w-0 flex-1">
            <div class="truncate text-sm font-medium text-gray-800">{{ modTitle(mod) }}</div>
            <div class="mt-0.5 flex items-center gap-2 text-xs text-gray-400">
              <span v-if="modSubtitle(mod) && modSubtitle(mod) !== modTitle(mod)" class="truncate">{{ modSubtitle(mod) }}</span>
              <span v-if="modSubtitle(mod) && modSubtitle(mod) !== modTitle(mod)">·</span>
              <span>{{ formatBytes(mod.size) }}</span>
              <span v-if="mod.loader_type !== 'unknown'">·</span>
              <span v-if="mod.loader_type !== 'unknown'">{{ mod.loader_type }}</span>
              <span>·</span>
              <span :class="mod.is_enabled ? 'text-green-600' : 'text-gray-400'">
                {{ mod.is_enabled ? '已启用' : '已禁用' }}
              </span>
            </div>
          </div>
          <div class="flex flex-none items-center gap-1">
            <button
              class="rounded-md p-1.5 text-gray-400 transition-colors hover:bg-gray-100 hover:text-gray-600"
              :title="mod.is_enabled ? '禁用' : '启用'"
              @click="handleToggleMod(mod)"
            >
              <svg v-if="mod.is_enabled" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor"><path d="M5 4a2 2 0 012-2h6a2 2 0 012 2v14l-5-2.5L5 18V4z" /></svg>
              <svg v-else class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor"><path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" /></svg>
            </button>
            <button
              class="rounded-md p-1.5 text-gray-400 transition-colors hover:bg-red-50 hover:text-red-600"
              title="删除"
              @click="handleDeleteMod(mod)"
            >
              <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" /></svg>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
