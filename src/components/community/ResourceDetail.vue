<script setup lang="ts">
/**
 * 资源详情弹窗
 *
 * - 顶部资源预览 + 操作按钮（ResourceDetailHeader 子组件）
 * - 版本筛选 RadioButton（HorizontalFilter）
 * - 版本按游戏版本分组卡片（VersionGroupCard 子组件，可折叠/展开带动画）
 * - 加载进度条
 * - 下载进度浮层（DownloadProgressOverlay 子组件）
 */

import { ref, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ResourceProject, ResourceVersion, ResolvedDependency } from '@/types/community'
import { getProjectVersions, downloadResourceToPath, formatDownloadFilename, installModpack } from '@/utils/api/community'
import { installMerged } from '@/utils/api/loader'
import { getVersionLoaderInfo } from '@/utils/api/version'
import { useVersionStore } from '@/stores/version'
import { pickSavePath } from '@/utils/fileDialog'
import { toastSuccess, toastError, toastInfo } from '@/utils/toast'
import { showPrompt, showModal } from '@/utils/modal'
import { isCancelledError } from '@/utils/async'
import { loaderToFlag } from '@/utils/mod-display'
import { useVersionGroups, getFilterVersionName } from '@/composables/useVersionGroups'
import { useSearchProgress } from '@/composables/useSearchProgress'
import { useDependencyCheck } from '@/composables/useDependencyCheck'
import HorizontalFilter from '@/components/common/HorizontalFilter.vue'
import ResourceDetailHeader from './resource-detail/ResourceDetailHeader.vue'
import { ArchiveBoxXMarkIcon } from '@heroicons/vue/24/outline'
import VersionGroupCard from './resource-detail/VersionGroupCard.vue'
import DependencyConfirmDialog from './DependencyConfirmDialog.vue'

const versionStore = useVersionStore()

interface Props {
  visible: boolean
  project: ResourceProject | null
  versionId?: string
  /** 整合包对应的 MC 版本号，设置后自动选中顶部筛选 tag */
  gameVersion?: string
  /** 整合包的 mods 目录路径，设置后下载 Mod 默认保存到该目录 */
  modsDir?: string
  /** 是否禁止更新 Mod（版本独立设置 advance_disable_mod_update），开启后下载已存在文件时拦截 */
  disableModUpdate?: boolean
}

const props = defineProps<Props>()
const emit = defineEmits<{ close: [] }>()

const versions = ref<ResourceVersion[]>([])
const loading = ref(false)
const downloading = ref<string | null>(null)

const { groups, filterOptions, versionFilter, toggleGroup, setFilter, expandedOf, mountedOf } = useVersionGroups(() => versions.value)
const { percent, slowMerging, stageText, start, finish, fail } = useSearchProgress()

// 前置 Mod 检查
const {
  checking: depsChecking,
  installing: depsInstalling,
  missing: depsMissing,
  upToDate: depsUpToDate,
  check: checkDeps,
  install: installDeps,
  reset: resetDeps,
} = useDependencyCheck()

// 前置确认弹窗状态
const showDependencyDialog = ref(false)
// 暂存待安装的主 Mod（弹窗确认后用于 install 调用）
const pendingMainVersion = ref<ResourceVersion | null>(null)

watch(
  [() => props.visible, () => props.project],
  async ([v, p], [oldV, oldP]) => {
    // 仅在 visible 变为 true 或 project 变化时触发（避免 visible/gameVersion 单独变化重复加载）
    if (!v || !p) return
    if (v === oldV && p === oldP) return
    loading.value = true
    versions.value = []
    setFilter('')
    start(p.platform === 'CurseForge' ? 1 : p.platform === 'Modrinth' ? 2 : 0)
    try {
      versions.value = await getProjectVersions(p.platform, p.id)
      finish()
      // 整合包来自 ModTab 时自动选中对应版本筛选
      if (props.gameVersion) {
        const target = getFilterVersionName(props.gameVersion)
        if (target && filterOptions.value.includes(target)) {
          nextTick(() => setFilter(target))
        }
      }
    } catch (e: any) {
      toastError('加载版本列表失败: ' + (e?.message || String(e)))
      fail()
    } finally {
      loading.value = false
    }
  },
)

async function handleDownload(v: ResourceVersion) {
  if (!props.project) return
  const finalFileName = await formatDownloadFilename(v.file_name, props.project.translated_name)

  // 禁止更新 Mod 拦截：如果目标文件在 mods 目录已存在（即"更新"场景），阻止下载
  if (props.disableModUpdate && props.modsDir) {
    const targetPath = `${props.modsDir}/${finalFileName}`.replace(/\\/g, '/')
    try {
      const exists = await invoke<boolean>('plugin:fs|exists', { path: targetPath })
      if (exists) {
        toastError(`此版本已禁止更新 Mod：${finalFileName} 已存在。\n如需更新，请前往 版本设置 → 高级选项 关闭「禁止更新 Mod」`)
        return
      }
    } catch (e) {
      console.debug('[ResourceDetail] 检查文件存在性失败:', e)
    }
  }

  // 前置 Mod 检查：仅 ModTab 场景（有 modsDir + versionId + gameVersion + 是 Mod 类型）启用
  // Community 场景（无 modsDir）走原有"选择保存位置"流程
  const canCheckDeps = !!(props.modsDir && props.versionId && props.gameVersion
    && props.project.resource_type === 'Mod')
  if (canCheckDeps) {
    const hasMissing = await runDependencyCheck(v)
    if (hasMissing) {
      // 弹窗等用户确认，确认后由 handleDependencyConfirm 走 install 流程
      return
    }
    // 无缺失：继续走下方原有流程
  }

  const savePath = await pickSavePath({
    title: '选择保存位置',
    defaultPath: props.modsDir ? `${props.modsDir}/${finalFileName}` : finalFileName,
    filters: [{ name: '所有文件', extensions: ['*'] }],
  })
  if (!savePath) return

  downloading.value = v.id
  versionStore.startDownload(finalFileName)
  toastInfo(`开始下载: ${finalFileName}`)
  try {
    await downloadResourceToPath(v.download_url, finalFileName, savePath)
    // 不调用 finishDownload，由轮询检测 is_complete 自动完成
  } catch (e: any) {
    const msg = e?.message || String(e)
    // 后端已 mark_failed 重置 is_active，前端用 showModal + onConfirm 让用户点击确定后退出下载页
    showModal({
      type: 'error',
      title: '下载失败',
      message: msg,
      onConfirm: () => {
        versionStore.finishDownload()
      },
    })
  } finally {
    downloading.value = null
  }
}

/**
 * 触发前置 Mod 检查并打开确认弹窗
 *
 * 流程：
 * 1. 获取当前版本加载器（getVersionLoaderInfo），转 ModLoaderFlags
 * 2. 调用 check_mod_dependencies IPC
 * 3. 有缺失 → 暂存主 Mod 并打开弹窗
 *
 * @returns true=已弹窗等用户确认，false=无缺失或检查失败可直接下载
 */
async function runDependencyCheck(v: ResourceVersion): Promise<boolean> {
  if (!props.project || !props.versionId || !props.gameVersion) return false

  let modLoader = 0
  try {
    const info = await getVersionLoaderInfo(props.versionId)
    modLoader = loaderToFlag(info.loaderType)
  } catch (e) {
    console.debug('[ResourceDetail] 获取加载器信息失败，按 0 检查:', e)
  }

  try {
    const hasMissing = await checkDeps({
      versionId: props.versionId,
      platform: props.project.platform,
      modVersion: v,
      gameVersion: props.gameVersion,
      modLoader,
    })
    if (hasMissing) {
      pendingMainVersion.value = v
      showDependencyDialog.value = true
      return true
    }
  } catch (e: any) {
    // 检查失败不阻断下载，仅提示并降级到普通下载
    const msg = e?.message || String(e)
    console.debug('[ResourceDetail] 前置 Mod 检查失败:', msg)
    toastInfo('前置 Mod 检查失败，直接下载主 Mod')
  }
  return false
}

/**
 * 用户在 DependencyConfirmDialog 点击"确认安装"后回调
 *
 * 调用 install_mod_with_dependencies IPC，后端启动 DownloadSession 并发下载主 Mod + 勾选前置。
 * 进度通过 WS 推送到下载管理页。
 */
async function handleDependencyConfirm(selectedDeps: ResolvedDependency[]) {
  const main = pendingMainVersion.value
  if (!main || !props.versionId) return

  showDependencyDialog.value = false
  downloading.value = main.id
  versionStore.startDownload(main.file_name)
  const totalFiles = 1 + selectedDeps.length
  toastInfo(`开始下载: ${main.file_name}（含 ${selectedDeps.length} 个前置，共 ${totalFiles} 个文件）`)

  try {
    const result = await installDeps({
      versionId: props.versionId,
      mainVersion: main,
      deps: selectedDeps,
    })
    if (result.failedCount > 0) {
      // 部分失败：toast 警告，但仍由 WS 推送 mark_complete 触发退出
      toastInfo(`安装完成：成功 ${result.installedCount} / ${totalFiles}，失败 ${result.failedCount}`)
    } else {
      toastSuccess(`安装完成：共 ${result.installedCount} 个文件`)
    }
  } catch (e: any) {
    const msg = e?.message || String(e)
    if (isCancelledError(e)) {
      toastInfo('下载已取消')
      versionStore.finishDownload()
      return
    }
    showModal({
      type: 'error',
      title: '安装失败',
      message: msg,
      onConfirm: () => {
        versionStore.finishDownload()
      },
    })
  } finally {
    downloading.value = null
    pendingMainVersion.value = null
    resetDeps()
  }
}

/** 用户在 DependencyConfirmDialog 点击取消（不下载） */
function handleDependencyClose() {
  showDependencyDialog.value = false
  pendingMainVersion.value = null
  resetDeps()
}

/**
 * 安装整合包：下载原始包 + 解析 + 下载依赖 mods + 复制 overrides → 安装游戏本体 + 加载器
 * 进度走 download_state，复用 DownloadPanel
 *
 * 点击下载后先弹窗询问安装名称，用户可自定义；取消则中止安装。
 */
async function handleInstallModpack(v: ResourceVersion) {
  if (!props.project) return
  const { platform, resource_type, translated_name, raw_name, id: projectId } = props.project
  if (resource_type !== 'ModPack') return

  const defaultName = translated_name || raw_name || v.file_name.replace(/\.(zip|mrpack)$/i, '')

  // 弹窗询问安装名称，允许用户自定义（取消则中止）
  const instanceName = await promptForInstanceName(defaultName)
  if (!instanceName) return

  downloading.value = v.id
  versionStore.startDownload(instanceName)

  try {
    // 联机大厅阶段 3：传入平台来源元数据，后端安装完成后写入 modpack.meta.json
    // 用于创建联机房间时上报整合包信息，加入方据此判断是否需要一键安装
    const result = await installModpack({
      platform,
      downloadUrl: v.download_url,
      fileName: v.file_name,
      instanceName,
      projectId,
      fileId: v.id,
      modpackVersion: v.version,
      fileSize: v.size,
      name: translated_name || raw_name,
    })
    const loader = result.loader
    const lv = result.loaderVersion
    await installMerged(
      result.gameVersion,
      loader === 'forge' ? lv : undefined,
      loader === 'neoforge' ? lv : undefined,
      loader === 'fabric' || loader === 'quilt' ? lv : undefined,
      undefined, undefined, instanceName,
    )
    toastSuccess(`整合包 ${instanceName} 安装完成`)
  } catch (e: any) {
    const msg = e?.message || String(e)
    // 用户主动取消：仅 toast 提示并退出下载页，不弹错误窗
    if (isCancelledError(e)) {
      toastInfo('下载已取消')
      versionStore.finishDownload()
      return
    }
    // 真实失败：后端已 mark_failed 重置 is_active，前端用 showModal + onConfirm 让用户点击确定后退出下载页
    showModal({
      type: 'error',
      title: '整合包安装失败',
      message: msg,
      onConfirm: () => {
        versionStore.finishDownload()
      },
    })
  } finally {
    downloading.value = null
  }
}

/**
 * 弹窗询问整合包安装名称
 *
 * 将 callback 风格的 showPrompt 包装为 Promise，便于在 async 流程中 await。
 * 用户确认返回 trim 后的名称（空则回退默认名）；取消返回 null。
 */
function promptForInstanceName(defaultName: string): Promise<string | null> {
  return new Promise((resolve) => {
    showPrompt(
      '安装整合包',
      '请输入整合包的安装名称：',
      (value: string) => {
        const trimmed = value.trim()
        resolve(trimmed || defaultName)
      },
      {
        defaultValue: defaultName,
        placeholder: '请输入安装名称',
        onCancel: () => resolve(null),
      },
    )
  })
}
</script>

<template>
  <teleport to="body">
    <transition
      enter-active-class="transition ease-out duration-150"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition ease-in duration-100"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="visible && project"
        class="fixed inset-0 z-[9999] flex items-center justify-center p-4"
        @click.self="emit('close')"
      >
        <div class="absolute inset-0 bg-black/40" />
        <div class="relative w-full max-w-2xl bg-white rounded-lg shadow-xl flex flex-col max-h-[85vh]">
          <!-- 头部 + 操作按钮 -->
          <ResourceDetailHeader :project="project" @close="emit('close')" />

          <!-- 版本筛选 -->
          <div v-if="filterOptions.length > 1" class="px-4 py-2 border-b border-gray-100 bg-gray-50/50">
            <HorizontalFilter
              :model-value="versionFilter"
              :options="filterOptions.map(o => ({ label: o, value: o }))"
              @update:model-value="setFilter"
            />
          </div>

          <!-- 版本列表区 -->
          <div class="flex-1 overflow-y-auto p-2">
            <!-- 加载中：进度条 -->
            <div v-if="loading" class="py-12 px-4">
              <div class="flex flex-col items-center">
                <svg class="mb-4 h-8 w-8 animate-spin text-primary-500" viewBox="0 0 24 24" fill="none">
                  <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
                  <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
                </svg>
                <div class="w-full max-w-sm">
                  <div class="h-1.5 overflow-hidden rounded-full bg-gray-100">
                    <div
                      class="h-full rounded-full bg-gradient-to-r from-primary-400 to-primary-600 transition-all duration-300 ease-out"
                      :style="{ width: Math.min(100, percent) + '%' }"
                    />
                  </div>
                  <div class="mt-2 flex items-center justify-between text-xs">
                    <span class="text-gray-500">{{ stageText }}</span>
                    <span class="font-medium text-primary-600">{{ percent.toFixed(1) }}%</span>
                  </div>
                  <p v-if="slowMerging" class="mt-2 text-center text-[11px] text-gray-400">
                    资源有点多，稍安勿躁，静候处理
                  </p>
                </div>
              </div>
            </div>

            <!-- 版本分组卡片 -->
            <div v-else-if="groups.length > 0" class="space-y-1.5">
              <VersionGroupCard
                v-for="g in groups"
                :key="g.title"
                :title="g.title"
                :versions="g.versions"
                :expanded="expandedOf(g.title)"
                :mounted="mountedOf(g.title)"
                :downloading="downloading"
                :is-modpack="project.resource_type === 'ModPack'"
                @toggle="toggleGroup(g.title)"
                @download="handleDownload"
                @install="handleInstallModpack"
              />
            </div>

            <!-- 空状态 -->
            <div v-else class="py-12 flex flex-col items-center justify-center text-gray-400">
              <ArchiveBoxXMarkIcon class="w-10 h-10 mb-3" />
              <span class="text-sm">暂无版本数据</span>
            </div>

          </div>
        </div>
      </div>
    </transition>

    <!-- 前置 Mod 确认弹窗（独立 teleport，避免嵌套在详情弹窗内影响层级） -->
    <DependencyConfirmDialog
      :visible="showDependencyDialog"
      :missing="depsMissing"
      :up-to-date="depsUpToDate"
      :main-name="pendingMainVersion?.file_name || ''"
      :installing="depsInstalling"
      :checking="depsChecking"
      @close="handleDependencyClose"
      @confirm="handleDependencyConfirm"
    />
  </teleport>
</template>
