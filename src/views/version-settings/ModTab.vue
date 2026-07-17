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
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import * as tauri from '@/utils/tauri'
import { showSuccess, showError } from '@/utils/toast'
import { showInfo, showConfirm } from '@/utils/modal'
import { useVersionSettings } from '@/composables/useVersionSettings'
import { useModsPreload } from '@/composables/useModsPreload'
import { formatBytes } from '@/utils/format'
import { getProjectDetail, getMcmodUrl } from '@/utils/api/community'
import { open as openUrl } from '@tauri-apps/plugin-shell'
import type { ResourceProject } from '@/types/community'
import Tooltip from '@/components/common/Tooltip.vue'
import ResourceDetail from '@/components/community/ResourceDetail.vue'
// Mod 默认 logo（无 jar 内 logo 时使用，参考 PCL2 Icons/NoIcon.png）
import defaultModLogo from '@/assets/Mods/default-min.png'
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
  BookOpenIcon,
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

// Mod 详情弹窗（关联到 CF/MR 平台工程时使用）
const detailVisible = ref(false)
const detailProject = ref<ResourceProject | null>(null)
/** 当前正在加载详情的 mod file_name（用于按钮 spinner + 防止重复点击同一 mod） */
const detailLoadingFor = ref<string | null>(null)

/**
 * 预加载事件监听：后端 `preload_mods_detail_cmd` 批量查询 CF/MR 后，
 * 通过 `mods-preload-update` 事件推送每个 mod 的 project，本 composable 自动更新 mods 数组。
 */
const { startListener: startPreloadListener, stopListener: stopPreloadListener, isPreloadDone } = useModsPreload(mods)

/**
 * 当前整合包对应的 MC 版本号和 mods 目录路径
 *
 * 在 onMounted 时预取，避免用户点击「详情」按钮后才请求导致卡顿。
 * - gameVersion：传给 ResourceDetail，自动选中顶部筛选 tag
 * - modsDir：传给 ResourceDetail，下载按钮默认保存到此目录
 */
const versionGameVersion = ref<string | null>(null)
const versionModsDir = ref<string | null>(null)

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

/** 预取整合包的 MC 版本号和 mods 目录（不阻塞 UI） */
async function prefetchVersionContext() {
  if (!selectedId.value) return
  try {
    versionGameVersion.value = await tauri.getVersionGameVersion(selectedId.value)
  } catch (e) {
    console.debug('[ModTab] 获取版本号失败:', e)
    versionGameVersion.value = null
  }
  try {
    versionModsDir.value = await tauri.getVersionModsDir(selectedId.value)
  } catch (e) {
    console.debug('[ModTab] 获取 mods 目录失败:', e)
    versionModsDir.value = null
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

/** 加载器类型对应的显示标签（用于详情行和本地信息弹窗） */
function loaderVisual(type: string): { label: string } {
  const t = type.toLowerCase()
  if (t === 'forge') return { label: 'Forge' }
  if (t === 'neoforge') return { label: 'NeoForge' }
  if (t === 'fabric') return { label: 'Fabric' }
  if (t === 'quilt') return { label: 'Quilt' }
  if (t === 'liteloader') return { label: 'LiteLoader' }
  return { label: '未知' }
}

/**
 * 启用/禁用 Mod（参考 PCL2 MyLocalModItem.Enable_Click）
 *
 * 核心设计：**原地更新 mod 字段，不重新加载列表**。
 *
 * 原设计（`await loadMods()`）的问题：
 * 1. 列表视觉闪烁刷新
 * 2. 后端排序规则「启用的排前面 + 文件名升序」会导致禁用的 mod 从启用区跳到禁用区末尾，
 *    用户看到的 mod 突然窜到列表最后，体验差
 * 3. 预加载的 `project` 字段全部丢失（list_mods 返回时 project 为空），用户点详情按钮又要等预加载
 *
 * 现设计：后端 toggle_mod 返回新文件名，前端按 file_name 找到对应 mod 原地更新三个字段：
 * - `file_name`：禁用后变 `xxx.jar.disabled`，启用后变回 `xxx.jar`
 * - `is_enabled`：取反
 * - `enabled_name`：保持不变（永远是去后缀的名称）
 *
 * 这样 mod 在列表中的位置完全不动，project 字段也保留。
 */
async function handleToggleMod(mod: tauri.ModInfo) {
  if (!selectedId.value) return
  const enable = !mod.is_enabled
  try {
    const newFileName = await tauri.toggleMod(selectedId.value, mod.file_name, enable)
    // 原地更新：按 file_name 找到对应 mod，更新字段（用整对象替换确保 Vue 响应式触发）
    const idx = mods.value.findIndex(m => m.file_name === mod.file_name)
    if (idx !== -1) {
      mods.value[idx] = {
        ...mods.value[idx],
        file_name: newFileName,
        is_enabled: enable,
      }
    }
    showSuccess(enable ? '已启用' : '已禁用', mod.enabled_name)
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

/**
 * 详情按钮（参考 PCL2 `MyLocalModItem.Info_Click` 第 751-792 行）：
 *
 * 核心设计：**详情按钮本身不发任何网络请求**，只判断 `mod.project` 是否已被预加载填充。
 *
 * 三级 fallback：
 * 1. **零延迟路径（最优）**：`mod.project` 已被 `preload_mods_detail_cmd` 后台预加载填充
 *    → 直接弹 ResourceDetail（与 PCL2 `Entry.Project IsNot Nothing` 分支一致）
 * 2. **并发 fallback**：预加载尚未完成（用户点太快）或预加载失败
 *    → 并发请求 CF + MR（`Promise.any`），谁先成功用谁
 * 3. **本地信息**：无 slug 或两个平台都查不到
 *    → 弹本地信息弹窗 + "百科搜索"按钮（与 PCL2 `Else` 分支一致）
 *
 * 防呆：detailLoadingFor 记录当前加载中的 mod file_name，
 * 按钮显示 spinner 并禁用同 mod 的重复点击。
 */
async function handleShowInfo(mod: tauri.ModInfo) {
  // 防呆：同一 mod 正在加载中，忽略重复点击
  if (detailLoadingFor.value === mod.file_name) return

  // 1. 零延迟路径：预加载已就绪，直接弹窗（参考 PCL2 Entry.Project IsNot Nothing）
  if (mod.project) {
    detailProject.value = mod.project
    detailVisible.value = true
    return
  }

  // 2. 无 slug：预加载可能还没读到 jar 元数据，等一小段时间再判断
  //    如果预加载已完成且仍无 slug，说明 jar 内没有 metadata，立即走本地信息弹窗
  if (!mod.slug) {
    // 预加载已完成 → slug 不会再来了，立即走本地信息弹窗
    if (isPreloadDone.value) {
      showLocalModInfo(mod)
      return
    }
    // 预加载未完成 → 等待最多 3 秒（每 100ms 检查一次 slug 或 project 是否就绪）
    detailLoadingFor.value = mod.file_name
    try {
      for (let i = 0; i < 30; i++) {
        await new Promise(r => setTimeout(r, 100))
        const current = mods.value.find(m => m.file_name === mod.file_name)
        if (current?.slug) {
          mod = current
          break
        }
        // 如果预加载期间 project 就绪了，直接弹窗
        if (current?.project) {
          detailProject.value = current.project
          detailVisible.value = true
          return
        }
        // 预加载已完成 → slug 不会再来了，跳出等待
        if (isPreloadDone.value) break
      }
    } finally {
      detailLoadingFor.value = null
    }
    // 等待后仍无 slug，走本地信息弹窗
    if (!mod.slug) {
      showLocalModInfo(mod)
      return
    }
  }

  // 3. 有 slug 但 project 未就绪：并发请求 CF + MR
  detailLoadingFor.value = mod.file_name
  try {
    const project = await Promise.any([
      getProjectDetail('CurseForge', mod.slug, 'Mod').catch(e => {
        console.debug('[ModTab] CF 详情查询失败:', e)
        throw e
      }),
      getProjectDetail('Modrinth', mod.slug, 'Mod').catch(e => {
        console.debug('[ModTab] MR 详情查询失败:', e)
        throw e
      }),
    ])
    detailProject.value = project
    detailVisible.value = true
  } catch (e) {
    // Promise.any 在所有 promise 都 reject 时抛 AggregateError
    console.debug('[ModTab] CF/MR 详情查询均失败，回退本地信息:', e)
    showLocalModInfo(mod)
  } finally {
    detailLoadingFor.value = null
  }
}

/** 显示本地 Mod 信息弹窗（无法关联到 CF/MR 平台时使用） */
function showLocalModInfo(mod: tauri.ModInfo) {
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

/**
 * 前往百科按钮（参考 PCL2 PageDownloadCompDetail.BtnIntroWiki_Click）：
 * - 优先通过 slug 查 mcmod.cn 直链（先 CF 后 MR，因为 mcmod 数据库中 CF 收录更全）
 * - 查不到直链时打开 mcmod.cn 搜索页，关键字优先用译名，其次用文件名去扩展名+版本号
 *
 * 搜索 URL 格式：https://search.mcmod.cn/s?key=<keyword>
 * 关键字必须去除版本号等参数（如 "AI-Improvements-1.20-0.5.2" → "AI-Improvements"），
 * 否则百科搜索匹配不到结果。
 */
async function handleOpenWiki(mod: tauri.ModInfo) {
  // 有 slug：尝试 CF → MR 查 mcmod.cn 直链
  if (mod.slug) {
    try {
      let url = await getMcmodUrl('CurseForge', mod.slug)
      if (!url) url = await getMcmodUrl('Modrinth', mod.slug)
      if (url) {
        await openUrl(url)
        return
      }
    } catch (e) {
      console.debug('[ModTab] 查 mcmod 直链失败，回退搜索页:', e)
    }
  }
  // 回退：打开 mcmod.cn 搜索页（注意：URL 是 search.mcmod.cn/s?key=，不是 www.mcmod.cn/search?key=）
  const keyword = stripModVersion(mod.translated_name || mod.file_name)
  const searchUrl = `https://search.mcmod.cn/s?key=${encodeURIComponent(keyword)}`
  await openUrl(searchUrl)
}

/**
 * 去除 mod 名称中的版本号等信息，提取纯名称用于百科搜索
 *
 * 例：
 * - "AI-Improvements-1.20-0.5.2.jar" → "AI-Improvements"
 * - "AI-Improvements-1.20-0.5.2" → "AI-Improvements"
 * - "FabricAPI-0.92.2+1.20.4" → "FabricAPI"
 * - "create-1.20.1-6.0.4.jar" → "create"
 *
 * 规则：
 * 1. 先去文件扩展名（.jar / .disabled / .old）
 * 2. 在第一个匹配版本号的位置（如 1.20 / 0.92.2 / 6.0.4）截断
 * 3. 去掉末尾的连字符/下划线/点
 */
function stripModVersion(name: string): string {
  // 1. 去扩展名
  let s = name.replace(/\.jar(\.disabled|\.old)?$/i, '').replace(/\.(litemod)(\.disabled|\.old)?$/i, '')
  // 2. 在版本号处截断（版本号特征：-<数字>.<数字> 或 +<数字>.<数字> 或 _<数字>.<数字>）
  //    匹配如 -1.20 / -0.92.2 / -6.0.4 / +1.20.4 等
  const m = s.match(/^([^-\s+_]+(?:[-\s+_][^-\s+_]+)*?)[-+_]\d+\.\d+/)
  if (m) {
    return m[1].replace(/[-\s+_]+$/, '')
  }
  return s
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
  // 启动预加载事件监听（必须在 loadMods 之前启动，避免错过早期事件）
  startPreloadListener()
  await checkModable()
  if (isModableVersion.value) {
    await loadMods()
    // 预取整合包的 MC 版本号和 mods 目录路径，避免用户点击详情按钮时才请求造成卡顿
    prefetchVersionContext()
    // 触发后台预加载：批量查询每个 mod 的 CF/MR 工程详情
    // 后台异步执行，不阻塞 UI；结果通过 mods-preload-update 事件推送
    if (selectedId.value) {
      tauri.preloadModsDetail(selectedId.value).catch(e => {
        console.debug('[ModTab] 预加载启动失败:', e)
      })
    }
  }
})

onUnmounted(() => {
  stopPreloadListener()
})
</script>

<template>
  <div class="flex flex-1 flex-col overflow-hidden">
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

    <!-- Mod 管理主体：工具栏固定不滚动，列表区独立滚动 -->
    <div v-else class="flex h-full flex-col">
      <!-- 顶部工具栏（flex-none 固定，不随列表滚动） -->
      <section class="flex-none border-b border-gray-200 bg-white px-6 py-3">
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

      <!-- 列表滚动区（只有这里滚动，工具栏固定不动） -->
      <div class="flex-1 overflow-y-auto p-6">
        <!-- 加载中（与 VersionSelect 统一样式） -->
        <div v-if="modsLoading" class="flex h-full items-center justify-center">
          <div class="flex flex-col items-center gap-3 text-gray-400">
            <svg class="h-8 w-8 animate-spin" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
              <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
            </svg>
            <span class="text-sm">正在加载 Mod 列表...</span>
          </div>
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

        <!-- Mod 列表 -->
        <div v-else class="overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm">
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

            <!-- 图标：有 logo 用真实 logo，否则用默认图片（不再用字母色块） -->
            <div class="relative flex-none">
              <img
                :src="mod.logo_data || defaultModLogo"
                class="h-9 w-9 rounded-lg object-cover"
                :class="{ 'opacity-50 grayscale': !mod.is_enabled }"
                alt=""
                @error="(e) => { (e.target as HTMLImageElement).src = defaultModLogo }"
              >
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

            <!-- 操作区：五个按钮，默认隐藏，hover 时显示（参考 PCL2 ButtonStack opacity 动画） -->
            <!-- 加载中时强制显示，避免鼠标离开后 spinner 消失让用户以为没点到（防呆） -->
            <div
              class="flex flex-none items-center gap-1 transition-opacity duration-200"
              :class="detailLoadingFor === mod.file_name
                ? 'opacity-100'
                : 'opacity-0 group-hover:opacity-100'"
            >
              <Tooltip :text="detailLoadingFor === mod.file_name ? '正在加载详情...' : '查看详情'" position="top">
                <button
                  class="rounded-md p-1.5 transition-colors disabled:cursor-wait"
                  :class="detailLoadingFor === mod.file_name
                    ? 'text-blue-500 bg-blue-50 cursor-wait'
                    : 'text-gray-400 hover:bg-blue-50 hover:text-blue-600'"
                  :disabled="detailLoadingFor === mod.file_name"
                  @click="handleShowInfo(mod)"
                >
                  <!-- 加载中：旋转 spinner，让用户明确感知按钮已响应（防呆） -->
                  <svg v-if="detailLoadingFor === mod.file_name" class="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
                    <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
                    <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
                  </svg>
                  <InformationCircleIcon v-else class="h-4 w-4" />
                </button>
              </Tooltip>
              <Tooltip text="前往百科" position="top">
                <button
                  class="rounded-md p-1.5 text-gray-400 transition-colors hover:bg-emerald-50 hover:text-emerald-600"
                  @click="handleOpenWiki(mod)"
                >
                  <BookOpenIcon class="h-4 w-4" />
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

    <!-- Mod 详情弹窗（关联到 CF/MR 平台工程时弹出，复用社区资源详情组件） -->
    <ResourceDetail
      :visible="detailVisible"
      :project="detailProject"
      :version-id="selectedId || undefined"
      :game-version="versionGameVersion || undefined"
      :mods-dir="versionModsDir || undefined"
      @close="detailVisible = false"
    />
  </div>
</template>
