<script setup lang="ts">
/**
 * 版本设置 - Mod 管理子页
 *
 * 设计参考 PCL2 PageInstanceMod + MyLocalModItem：
 * - 左侧 4px 状态色条（启用=primary/禁用=gray）
 * - 34×34 圆角真实 Logo 图标（从 jar 内提取，无 logo fallback 到加载器首字母色块）
 * - 标题 14px + 副标题 12px 灰色（译名/文件名按 modLocalNameStyle 切换）
 * - 详情行：文件大小 · 加载器类型 · 文件名（hover Tooltip 显示完整路径）
 * - 四个操作按钮（参考 PCL2 MyLocalModItem）：详情、打开文件位置、启用/禁用、删除
 * - 按钮默认 opacity-0 隐藏，hover 列表项时才显示（与 PCL2 ButtonStack 行为一致）
 */
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import * as tauri from '@/utils/tauri'
import { showSuccess, showError } from '@/utils/toast'
import { showInfo, showConfirm } from '@/utils/modal'
import { useVersionSettings } from '@/composables/useVersionSettings'
import { formatBytes } from '@/utils/format'
import Tooltip from '@/components/common/Tooltip.vue'
import {
  ArrowDownTrayIcon,
  FolderOpenIcon,
  ArrowPathIcon,
  PlayIcon,
  PauseIcon,
  TrashIcon,
  MagnifyingGlassIcon,
  PuzzlePieceIcon,
  InformationCircleIcon,
} from '@heroicons/vue/24/outline'

const router = useRouter()
const { selectedId, isModable } = useVersionSettings()

const mods = ref<tauri.ModInfo[]>([])
const modsLoading = ref(false)
const modFilter = ref<'all' | 'enabled' | 'disabled'>('all')
const modSearch = ref('')
const isModableVersion = ref(false)
const checkingModable = ref(false)
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
    showError('加载 Mod 列表失败', String(e))
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

const enabledCount = computed(() => mods.value.filter(m => m.is_enabled).length)
const disabledCount = computed(() => mods.value.filter(m => !m.is_enabled).length)

/** 根据 modLocalNameStyle 返回 Mod 标题（主显示名） */
function modTitle(mod: tauri.ModInfo): string {
  if (modLocalNameStyle.value === 0) {
    return mod.translated_name || mod.enabled_name
  }
  return mod.enabled_name
}

/** 根据 modLocalNameStyle 返回 Mod 副标题（详情名） */
function modSubtitle(mod: tauri.ModInfo): string {
  if (modLocalNameStyle.value === 0) {
    return mod.enabled_name
  }
  return mod.translated_name
}

/** 加载器类型对应的图标背景色与首字母（无 logo 时 fallback） */
function loaderVisual(type: string): { bg: string; text: string; label: string; letter: string } {
  const t = type.toLowerCase()
  if (t === 'forge') return { bg: 'bg-orange-100', text: 'text-orange-600', label: 'Forge', letter: 'F' }
  if (t === 'neoforge') return { bg: 'bg-amber-100', text: 'text-amber-600', label: 'NeoForge', letter: 'N' }
  if (t === 'fabric') return { bg: 'bg-cyan-100', text: 'text-cyan-600', label: 'Fabric', letter: 'F' }
  if (t === 'quilt') return { bg: 'bg-pink-100', text: 'text-pink-600', label: 'Quilt', letter: 'Q' }
  if (t === 'liteloader') return { bg: 'bg-rose-100', text: 'text-rose-600', label: 'LiteLoader', letter: 'L' }
  return { bg: 'bg-gray-100', text: 'text-gray-500', label: '未知', letter: 'M' }
}

async function handleToggleMod(mod: tauri.ModInfo) {
  if (!selectedId.value) return
  try {
    await tauri.toggleMod(selectedId.value, mod.file_name, !mod.is_enabled)
    showSuccess(mod.is_enabled ? '已禁用' : '已启用', mod.enabled_name)
    await loadMods()
  } catch (e) {
    showError('操作失败', String(e))
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
        showSuccess('Mod 已删除', mod.enabled_name)
        await loadMods()
      } catch (e) {
        showError('删除失败', String(e))
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
    showError('安装失败', String(e))
  }
}

async function handleOpenModsDir() {
  if (!selectedId.value) return
  try {
    await tauri.openModsDir(selectedId.value)
  } catch (e) {
    showError('打开文件夹失败', String(e))
  }
}

/** 打开单个 Mod 的文件位置（参考 PCL2 Open_Click） */
async function handleOpenFile(mod: tauri.ModInfo) {
  if (!selectedId.value) return
  try {
    await tauri.revealModFile(selectedId.value, mod.file_name)
  } catch (e) {
    showError('打开文件位置失败', String(e))
  }
}

/** 详情按钮：显示 Mod 完整信息（参考 PCL2 Info_Click） */
function handleShowInfo(mod: tauri.ModInfo) {
  const lines: string[] = []
  if (mod.description) {
    lines.push(mod.description)
    lines.push('')
  }
  lines.push(`文件：${mod.file_name}（${formatBytes(mod.size)}）`)
  if (mod.version) lines.push(`版本：${mod.version}`)
  if (mod.translated_name) lines.push(`译名：${mod.translated_name}`)
  if (mod.loader_type !== 'unknown') lines.push(`加载器：${loaderVisual(mod.loader_type).label}`)
  showInfo(modTitle(mod), lines.join('\n'))
}

const filterOptions = [
  { v: 'all' as const, l: '全部', count: () => mods.value.length },
  { v: 'enabled' as const, l: '已启用', count: () => enabledCount.value },
  { v: 'disabled' as const, l: '已禁用', count: () => disabledCount.value },
]

onMounted(async () => {
  try {
    const cfg = await tauri.getConfigMap()
    modLocalNameStyle.value = cfg.communityModLocalNameStyle
  } catch { /* 默认 0 */ }
  await checkModable()
  if (isModableVersion.value) await loadMods()
})
</script>

<template>
  <div class="flex flex-col">
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

    <!-- Mod 管理主体：顶部工具栏 sticky 固定，列表可滚动 -->
    <div v-else class="flex flex-col">
      <!-- 顶部工具栏（sticky 固定） -->
      <section class="sticky top-0 z-10 rounded-xl border border-gray-200 bg-white p-3 shadow-sm">
        <div class="flex flex-wrap items-center gap-2">
          <Tooltip text="从本地 jar 文件安装 Mod" position="bottom">
            <button
              class="flex items-center gap-1.5 rounded-lg bg-primary-600 px-3 py-1.5 text-xs font-medium text-white transition-colors hover:bg-primary-700"
              @click="handleInstallMod"
            >
              <ArrowDownTrayIcon class="h-3.5 w-3.5" />
              从文件安装
            </button>
          </Tooltip>
          <Tooltip text="在系统资源管理器中打开 mods 目录" position="bottom">
            <button
              class="flex items-center gap-1.5 rounded-lg border border-gray-300 bg-white px-3 py-1.5 text-xs font-medium text-gray-700 transition-colors hover:bg-gray-50"
              @click="handleOpenModsDir"
            >
              <FolderOpenIcon class="h-3.5 w-3.5" />
              打开文件夹
            </button>
          </Tooltip>
          <Tooltip text="重新扫描 mods 目录" position="bottom">
            <button
              class="flex items-center gap-1.5 rounded-lg border border-gray-300 bg-white px-3 py-1.5 text-xs font-medium text-gray-700 transition-colors hover:bg-gray-50"
              @click="loadMods"
            >
              <ArrowPathIcon class="h-3.5 w-3.5" :class="{ 'animate-spin': modsLoading }" />
              刷新
            </button>
          </Tooltip>

          <div class="ml-auto flex items-center gap-1 rounded-lg bg-gray-100 p-0.5">
            <button
              v-for="opt in filterOptions"
              :key="opt.v"
              class="flex items-center gap-1 rounded-md px-2.5 py-1 text-xs font-medium transition-colors"
              :class="modFilter === opt.v
                ? 'bg-white text-primary-700 shadow-sm'
                : 'text-gray-500 hover:text-gray-700'"
              @click="modFilter = opt.v"
            >
              {{ opt.l }}
              <span
                class="rounded-full px-1.5 py-0.5 text-[10px] leading-none"
                :class="modFilter === opt.v
                  ? 'bg-primary-100 text-primary-700'
                  : 'bg-gray-200 text-gray-500'"
              >{{ opt.count() }}</span>
            </button>
          </div>

          <div class="relative">
            <MagnifyingGlassIcon class="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-gray-400" />
            <input
              v-model="modSearch"
              type="text"
              placeholder="搜索 Mod 名称"
              class="w-56 rounded-lg border border-gray-300 bg-white py-1.5 pl-8 pr-3 text-xs text-gray-700 transition-colors placeholder:text-gray-400 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
            >
          </div>
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
          <PuzzlePieceIcon class="mx-auto mb-3 h-10 w-10 text-gray-300" />
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

      <!-- Mod 列表（与顶部工具栏无间距，紧贴下方） -->
      <div v-else class="mt-0 overflow-hidden rounded-xl border border-t-0 border-gray-200 bg-white shadow-sm">
        <ul class="divide-y divide-gray-100">
          <li
            v-for="mod in filteredMods"
            :key="mod.file_name"
            class="group relative flex items-center gap-3 px-3 py-2.5 transition-colors hover:bg-gray-50"
            :class="{ 'bg-gray-50/40': !mod.is_enabled }"
          >
            <!-- 左侧状态色条 -->
            <div
              class="absolute left-0 top-0 h-full w-1 transition-colors"
              :class="mod.is_enabled ? 'bg-primary-500' : 'bg-gray-300'"
            ></div>

            <!-- 图标：有 logo 用真实 logo，否则 fallback 到加载器色块 -->
            <div class="relative flex-none">
              <img
                v-if="mod.logo_data"
                :src="mod.logo_data"
                class="h-9 w-9 rounded-lg object-cover"
                :class="{ 'opacity-50 grayscale': !mod.is_enabled }"
                alt=""
                @error="(e) => (e.target as HTMLImageElement).style.display = 'none'"
              >
              <div
                v-else
                class="flex h-9 w-9 items-center justify-center rounded-lg text-sm font-semibold"
                :class="mod.is_enabled
                  ? loaderVisual(mod.loader_type).bg + ' ' + loaderVisual(mod.loader_type).text
                  : 'bg-gray-100 text-gray-400'"
              >
                {{ loaderVisual(mod.loader_type).letter }}
              </div>
              <!-- 禁用角标 -->
              <div
                v-if="!mod.is_enabled"
                class="absolute -bottom-1 -right-1 flex h-4 w-4 items-center justify-center rounded-full bg-gray-400 text-white shadow"
              >
                <PauseIcon class="h-2.5 w-2.5" />
              </div>
            </div>

            <!-- 信息区 -->
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <span
                  class="truncate text-sm font-medium"
                  :class="mod.is_enabled ? 'text-gray-800' : 'text-gray-500 line-through decoration-gray-300'"
                >
                  {{ modTitle(mod) }}
                </span>
                <span
                  v-if="mod.version"
                  class="flex-none rounded bg-gray-100 px-1.5 py-0.5 text-[10px] font-medium text-gray-500"
                >{{ mod.version }}</span>
                <span
                  v-if="modSubtitle(mod) && modSubtitle(mod) !== modTitle(mod)"
                  class="truncate text-xs text-gray-400"
                >{{ modSubtitle(mod) }}</span>
              </div>
              <div class="mt-0.5 flex items-center gap-1.5 text-xs text-gray-400">
                <span>{{ formatBytes(mod.size) }}</span>
                <span v-if="mod.loader_type !== 'unknown'">·</span>
                <span v-if="mod.loader_type !== 'unknown'">{{ loaderVisual(mod.loader_type).label }}</span>
                <span>·</span>
                <Tooltip :text="mod.file_name" position="top" :delay="200">
                  <span class="cursor-help underline decoration-dotted underline-offset-2 hover:text-gray-600">
                    {{ mod.file_name.length > 28 ? mod.file_name.slice(0, 25) + '...' : mod.file_name }}
                  </span>
                </Tooltip>
              </div>
            </div>

            <!-- 操作区：四个按钮，默认隐藏，hover 时显示（参考 PCL2 ButtonStack opacity 动画） -->
            <div class="flex flex-none items-center gap-1 opacity-0 transition-opacity duration-200 group-hover:opacity-100">
              <Tooltip text="查看详情" position="top">
                <button
                  class="rounded-md p-1.5 text-gray-400 transition-colors hover:bg-blue-50 hover:text-blue-600"
                  @click="handleShowInfo(mod)"
                >
                  <InformationCircleIcon class="h-4 w-4" />
                </button>
              </Tooltip>
              <Tooltip text="打开文件位置" position="top">
                <button
                  class="rounded-md p-1.5 text-gray-400 transition-colors hover:bg-gray-100 hover:text-gray-700"
                  @click="handleOpenFile(mod)"
                >
                  <FolderOpenIcon class="h-4 w-4" />
                </button>
              </Tooltip>
              <Tooltip :text="mod.is_enabled ? '禁用' : '启用'" position="top">
                <button
                  class="rounded-md p-1.5 transition-colors"
                  :class="mod.is_enabled
                    ? 'text-gray-400 hover:bg-amber-50 hover:text-amber-600'
                    : 'text-gray-400 hover:bg-green-50 hover:text-green-600'"
                  @click="handleToggleMod(mod)"
                >
                  <PauseIcon v-if="mod.is_enabled" class="h-4 w-4" />
                  <PlayIcon v-else class="h-4 w-4" />
                </button>
              </Tooltip>
              <Tooltip text="删除" position="top">
                <button
                  class="rounded-md p-1.5 text-gray-400 transition-colors hover:bg-red-50 hover:text-red-600"
                  @click="handleDeleteMod(mod)"
                >
                  <TrashIcon class="h-4 w-4" />
                </button>
              </Tooltip>
            </div>
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>
