<script setup lang="ts">
/**
 * 版本设置 - Mod 管理子页（顶层容器）
 * 子组件位于 mod-tab/（ModToolbar / ModListItem / ModEmptyState），列表项视觉与按钮设计细节见 ModListItem.vue。
 */
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import * as tauri from '@/utils/tauri'
import { showSuccess, showError } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { useVersionSettings } from '@/composables/useVersionSettings'
import { useModsPreload } from '@/composables/useModsPreload'
import { useModDetailQuery } from '@/composables/useModDetailQuery'
import { modTitle } from '@/utils/mod-display'
import ResourceDetail from '@/components/community/ResourceDetail.vue'
import ModListItem from './mod-tab/ModListItem.vue'
import ModToolbar from './mod-tab/ModToolbar.vue'
import ModEmptyState from './mod-tab/ModEmptyState.vue'

const router = useRouter()
const { selectedId, isModable } = useVersionSettings()

const mods = ref<tauri.ModInfo[]>([])
const modsLoading = ref(false)
const modFilter = ref<'all' | 'enabled' | 'disabled'>('all')
const modSearch = ref('')
const isModableVersion = ref(false)
const checkingModable = ref(false)
const modLocalNameStyle = ref(0)
/** 此版本是否禁止更新 Mod（advance_disable_mod_update），开启后 ResourceDetail 下载已存在文件时拦截 */
const disableModUpdate = ref(false)

/**
 * 预加载事件监听：后端 `preload_mods_detail_cmd` 批量查询 CF/MR 后，
 * 通过 `mods-preload-update` 事件推送每个 mod 的 project，本 composable 自动更新 mods 数组。
 */
const { startListener: startPreloadListener, stopListener: stopPreloadListener, isPreloadDone } = useModsPreload(mods)

const { detailVisible, detailProject, detailLoadingFor, handleShowInfo, handleOpenWiki } = useModDetailQuery()

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
  if (!selectedId.value) { isModableVersion.value = false; return }
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
  // 读取版本独立设置：是否禁止更新 Mod
  try {
    const p = await tauri.getVersionPersonalization(selectedId.value)
    disableModUpdate.value = p.advance_disable_mod_update
  } catch (e) {
    console.debug('[ModTab] 获取禁止更新 Mod 配置失败:', e)
    disableModUpdate.value = false
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

const filterOptions = computed(() => [
  { v: 'all' as const, l: '全部', count: mods.value.length },
  { v: 'enabled' as const, l: '已启用', count: enabledCount.value },
  { v: 'disabled' as const, l: '已禁用', count: disabledCount.value },
])

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
    `确定要删除 "${modTitle(mod, modLocalNameStyle.value)}" 吗？此操作不可恢复。`,
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

/** 详情按钮事件桥接：把 mods/isPreloadDone refs 传给 composable（模板中 ref 会自动解包，需在脚本中转发） */
function onShowInfo(mod: tauri.ModInfo) {
  handleShowInfo(mod, mods, isPreloadDone)
}

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
    <ModEmptyState
      v-if="!isModableVersion && !checkingModable"
      variant="not-modable"
      :mods-count="0"
      @go-download="router.push('/apps/downloads')"
      @go-select="router.push('/apps/versions/select')"
    />

    <!-- Mod 管理主体：工具栏固定不滚动，列表区独立滚动 -->
    <div v-else class="flex h-full flex-col">
      <ModToolbar
        v-model:modFilter="modFilter"
        v-model:modSearch="modSearch"
        :mods-loading="modsLoading"
        :filter-options="filterOptions"
        @install="handleInstallMod"
        @open-dir="handleOpenModsDir"
        @refresh="loadMods"
      />

      <!-- 列表滚动区（只有这里滚动，工具栏固定不动） -->
      <div class="flex-1 overflow-y-auto p-6">
        <ModEmptyState v-if="modsLoading" variant="loading" :mods-count="0" />
        <ModEmptyState
          v-else-if="filteredMods.length === 0"
          :variant="mods.length === 0 ? 'empty' : 'no-match'"
          :mods-count="mods.length"
          @install="handleInstallMod"
        />
        <!-- Mod 列表 -->
        <div v-else class="overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm">
          <ul class="divide-y divide-gray-100">
            <ModListItem
              v-for="mod in filteredMods"
              :key="mod.file_name"
              :mod="mod"
              :detail-loading-for="detailLoadingFor"
              :mod-local-name-style="modLocalNameStyle"
              @toggle="handleToggleMod"
              @delete="handleDeleteMod"
              @show-info="onShowInfo"
              @open-wiki="handleOpenWiki"
              @open-file="handleOpenFile"
            />
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
      :disable-mod-update="disableModUpdate"
      @close="detailVisible = false"
    />
  </div>
</template>
