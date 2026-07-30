/**
 * 资源详情页下载 + 前置 Mod 检查 composable
 *
 * 从 ResourceDetail.vue 抽出，负责：
 * - 普通下载流程（选目录 → 下载到指定路径）
 * - 前置 Mod 依赖检查（check_mod_dependencies）
 * - 前置确认弹窗状态管理
 * - 前置项目详情懒加载缓存（VersionGroupCard 展开时触发）
 *
 * # 复用约定
 * - 前置检查/安装复用 useDependencyCheck composable
 * - 项目详情查询复用 getProjectDetail API
 * - 文件名格式化复用 formatDownloadFilename
 * - 保存路径选择复用 pickSavePath
 *
 * # 响应式说明
 * - 直接接收 ResourceDetail 的 props（Vue 3 reactive proxy），composable 内部
 *   通过 options.xxx 访问以拿到最新值。不要在调用方解构 props 后再传入，
 *   否则原始值会丢失响应式（用户切换资源时 composable 仍看到旧 project）。
 */
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ResourceProject, ResourceVersion, ResolvedDependency } from '@/types/community'
import { getProjectDetail, downloadResourceToPath, formatDownloadFilename } from '@/utils/api/community'
import { getVersionLoaderInfo } from '@/utils/api/version'
import { getVersionGameVersion, getVersionModsDir } from '@/utils/api/personalization'
import { useVersionStore } from '@/stores/version'
import { pickSavePath, pickDirectory } from '@/utils/fileDialog'
import { toastSuccess, toastError, toastInfo } from '@/utils/toast'
import { showModal } from '@/utils/modal'
import { isCancelledError } from '@/utils/async'
import { loaderToFlag } from '@/utils/mod-display'
import { useDependencyCheck } from '@/composables/useDependencyCheck'

export interface UseResourceDownloadOptions {
  /** 资源项目（props.project，reactive） */
  project: ResourceProject | null
  /** 当前版本 ID（Community 场景下可能为空，回退 versionStore.selectedVersion） */
  versionId?: string
  /** 整合包对应的 MC 版本号 */
  gameVersion?: string
  /** mods 目录路径 */
  modsDir?: string
  /** 是否禁止更新 Mod */
  disableModUpdate?: boolean
}

export function useResourceDownload(options: UseResourceDownloadOptions) {
  const versionStore = useVersionStore()

  // 下载中标志（值=正在下载的 version_id，null=空闲）
  const downloading = ref<string | null>(null)
  // 下载阶段（按钮文字分阶段显示）
  // idle=空闲 / requesting=请求中（前置检查/准备）/ waiting=等待用户确认前置 / downloading=下载中
  const downloadStage = ref<'idle' | 'requesting' | 'waiting' | 'downloading'>('idle')

  // 前置确认弹窗状态
  const showDependencyDialog = ref(false)
  // 暂存待安装的主 Mod（弹窗确认后用于 install 调用）
  const pendingMainVersion = ref<ResourceVersion | null>(null)
  // 暂存前置检查解析出的上下文（versionId/gameVersion/modsDir/modLoader）
  // Community 场景下 versionId 为空，modsDir 为 undefined；版本管理场景下均有值
  const pendingContext = ref<{
    versionId: string
    gameVersion: string
    modsDir: string | undefined
    modLoader: number
  } | null>(null)

  /**
   * 从 game_versions 数组中推断真实游戏版本（过滤加载器名称）
   *
   * CurseForge 的 gameVersions 可能包含 "Forge"/"Fabric" 等加载器名称，
   * Modrinth 的 game_versions 只含纯版本号。统一过滤已知加载器名取第一个。
   */
  function inferGameVersion(gameVersions: string[]): string {
    const loaderNames = ['Forge', 'Fabric', 'Quilt', 'NeoForge', 'LiteLoader', 'Rift']
    return gameVersions.find(gv => !loaderNames.includes(gv)) || ''
  }

  // 前置项目详情缓存（key=version_id），VersionGroupCard 展开前置列表时懒加载
  const depsMap = ref<Map<string, ResourceProject[]>>(new Map())
  // 正在加载前置的 version_id 集合
  const depsLoadingSet = ref<Set<string>>(new Set())

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

  /**
   * 懒加载查询版本的前置项目详情
   *
   * VersionGroupCard 展开前置列表时触发，对 version.dependencies 里每个 project_id
   * 调用 getProjectDetail 查询详情（logo/名称/平台），存入 depsMap。
   * 已缓存的不再重复查询；单个查询失败不阻断整体，过滤 null。
   */
  async function handleLoadDeps(v: ResourceVersion) {
    if (depsMap.value.has(v.id) || depsLoadingSet.value.has(v.id)) return
    if (!options.project || v.dependencies.length === 0) return

    const next = new Set(depsLoadingSet.value)
    next.add(v.id)
    depsLoadingSet.value = next

    try {
      const platform = options.project.platform
      const projects = await Promise.all(
        v.dependencies.map(pid =>
          getProjectDetail(platform, pid, 'Mod').catch(() => null),
        ),
      )
      const map = new Map(depsMap.value)
      map.set(v.id, projects.filter((p): p is ResourceProject => p !== null))
      depsMap.value = map
    } finally {
      const after = new Set(depsLoadingSet.value)
      after.delete(v.id)
      depsLoadingSet.value = after
    }
  }

  /**
   * 下载资源版本到用户选择的路径
   *
   * 流程：
   * 1. 立即设 downloading = v.id（按钮显示 loading，防止检查依赖期间重复点击）
   * 2. 禁止更新 Mod 拦截（modsDir 下已存在同名文件则阻止）
   * 3. Mod 类型触发前置检查，有缺失则弹窗等用户确认（downloading 保持）
   * 4. 选目录 → 下载
   *
   * finally 中仅在未进入前置弹窗时清空 downloading，
   * 弹窗等待期间保持 loading，由 handleDependencyConfirm/handleDependencyClose 清空。
   */
  async function handleDownload(v: ResourceVersion) {
    if (!options.project) return
    // 立即设 loading，防止用户在检查依赖期间重复点击
    downloading.value = v.id
    // 进入请求阶段（前置检查或直接下载准备）
    downloadStage.value = 'requesting'

    try {
      const finalFileName = await formatDownloadFilename(v.file_name, options.project.translated_name)

      // 禁止更新 Mod 拦截：如果目标文件在 mods 目录已存在（即"更新"场景），阻止下载
      if (options.disableModUpdate && options.modsDir) {
        const targetPath = `${options.modsDir}/${finalFileName}`.replace(/\\/g, '/')
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

      // 前置 Mod 检查：仅 Mod 类型 + 版本管理场景（有 versionId）+ 有 dependencies 时启用
      // Community 场景（无 versionId）：不检查前置，直接走文件选择对话框
      if (options.project.resource_type === 'Mod' && v.dependencies.length > 0) {
        const versionId = options.versionId || versionStore.selectedVersion || ''
        if (versionId) {
          // 版本管理场景：检查前置
          const hasMissing = await runDependencyCheck(v)
          if (hasMissing) {
            // runDependencyCheck 内部已将 downloadStage 设为 'waiting'
            return
          }
        }
        // Community 场景或无缺失：继续走文件选择
      }

      // 实际下载阶段
      downloadStage.value = 'downloading'
      const savePath = await pickSavePath({
        title: '选择保存位置',
        defaultPath: options.modsDir ? `${options.modsDir}/${finalFileName}` : finalFileName,
        filters: [{ name: '所有文件', extensions: ['*'] }],
      })
      if (!savePath) return

      versionStore.startDownload(finalFileName)
      toastInfo(`开始下载: ${finalFileName}`)
      try {
        await downloadResourceToPath(v.download_url, finalFileName, savePath)
        // 兜底：WS 可能因时序未收到 is_complete，IPC 返回成功后直接 finishDownload
        // 若 WS 已 finishDownload（downloading=false），此处无副作用（finishDownload 幂等）
        if (versionStore.downloading) {
          versionStore.finishDownload()
          toastSuccess(`${finalFileName} 下载完成`)
        }
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
      }
    } finally {
      // 仅在未进入前置弹窗时清空 loading
      if (!showDependencyDialog.value) {
        downloading.value = null
        downloadStage.value = 'idle'
      }
    }
  }

  /**
   * 触发前置 Mod 检查并打开确认弹窗
   *
   * 流程：
   * 1. 解析 versionId（Community 场景可为空）
   * 2. 版本管理场景：并行获取 gameVersion / modsDir / loaderType
   *    Community 场景：从 modVersion 自身推断 gameVersion 和 modLoader，不扫描已安装
   * 3. 调用 check_mod_dependencies IPC
   * 4. 有缺失 → 暂存主 Mod + 上下文，打开弹窗
   *
   * @returns true=已弹窗等用户确认，false=无缺失或检查失败可直接下载
   */
  async function runDependencyCheck(v: ResourceVersion): Promise<boolean> {
    if (!options.project) return false

    // 解析 versionId（Community 场景下 options.versionId 可为空）
    const versionId = options.versionId || versionStore.selectedVersion || ''

    let gameVersion: string
    let modsDir: string | undefined
    let modLoader: number

    if (versionId) {
      // 版本管理场景：从版本设置获取 gameVersion/modsDir/loaderType
      const [gv, md, loaderInfo] = await Promise.all([
        options.gameVersion ? Promise.resolve(options.gameVersion) : getVersionGameVersion(versionId).catch(() => null),
        options.modsDir ? Promise.resolve(options.modsDir) : getVersionModsDir(versionId).catch(() => ''),
        getVersionLoaderInfo(versionId).catch(() => ({ loaderType: 'release', loaderVersion: '' })),
      ])
      if (!gv) {
        console.debug('[ResourceDetail] 前置检查跳过：无 gameVersion (versionId=%s)', versionId)
        return false
      }
      gameVersion = gv
      modsDir = md || undefined
      modLoader = loaderToFlag(loaderInfo.loaderType)
    } else {
      // Community 场景：从 modVersion 自身推断 gameVersion 和 modLoader，不扫描已安装
      gameVersion = inferGameVersion(v.game_versions)
      modLoader = v.mod_loaders
      modsDir = undefined
      console.debug('[ResourceDetail] Community 场景：推断 gameVersion=%s modLoader=%d', gameVersion, modLoader)
    }

    if (!gameVersion) {
      console.debug('[ResourceDetail] 前置检查跳过：无法推断 gameVersion')
      return false
    }

    pendingContext.value = { versionId, gameVersion, modsDir, modLoader }

    console.debug(
      '[ResourceDetail] 前置检查：mod=%s platform=%s 依赖数=%d game=%s loader=%d',
      v.file_name, options.project.platform, v.dependencies?.length ?? 0, gameVersion, modLoader,
    )

    try {
      const hasMissing = await checkDeps({
        versionId: versionId || undefined,
        modsDir,
        platform: options.project.platform,
        modVersion: v,
        gameVersion,
        modLoader,
      })
      console.debug(
        '[ResourceDetail] 前置检查完成：缺失=%d 已满足=%d',
        depsMissing.value.length, depsUpToDate.value.length,
      )
      if (hasMissing) {
        pendingMainVersion.value = v
        showDependencyDialog.value = true
        // 进入等待阶段：等用户在弹窗确认
        downloadStage.value = 'waiting'
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
   * - 版本管理场景（有 versionId）：直接 install，下载到版本 mods 目录
   * - Community 场景（无 versionId）：先选保存文件夹，install 时传 targetDir
   *
   * 调用 install_mod_with_dependencies IPC，后端启动 DownloadSession 并发下载主 Mod + 勾选前置。
   * 进度通过 WS 推送到下载管理页。
   */
  async function handleDependencyConfirm(selectedDeps: ResolvedDependency[]) {
    const main = pendingMainVersion.value
    const ctx = pendingContext.value
    if (!main || !ctx || !options.project) return

    showDependencyDialog.value = false

    // Community 场景（无 versionId）：先选保存文件夹
    let targetDir: string | undefined
    if (!ctx.versionId) {
      const dir = await pickDirectory({
        title: '选择保存文件夹（主 Mod + 前置将下载到此目录）',
      })
      if (!dir) {
        // 用户取消，清空 loading 和状态
        downloading.value = null
        pendingMainVersion.value = null
        pendingContext.value = null
        resetDeps()
        return
      }
      targetDir = dir
    }

    downloading.value = main.id
    // 进入下载阶段
    downloadStage.value = 'downloading'
    versionStore.startDownload(main.file_name)
    const totalFiles = 1 + selectedDeps.length
    toastInfo(`开始下载: ${main.file_name}（含 ${selectedDeps.length} 个前置，共 ${totalFiles} 个文件）`)

    try {
      const result = await installDeps({
        versionId: ctx.versionId || undefined,
        targetDir,
        mainVersion: main,
        deps: selectedDeps,
      })
      if (result.failedCount > 0) {
        // 部分失败：toast 警告，但仍由 WS 推送 mark_complete 触发退出
        toastInfo(`安装完成：成功 ${result.installedCount} / ${totalFiles}，失败 ${result.failedCount}`)
      } else {
        toastSuccess(`安装完成：共 ${result.installedCount} 个文件`)
      }
      // 兜底：WS 可能因时序未收到 is_complete，IPC 返回成功后直接 finishDownload
      // 若 WS 已 finishDownload（downloading=false），此处无副作用（finishDownload 幂等）
      if (versionStore.downloading) {
        versionStore.finishDownload()
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
      downloadStage.value = 'idle'
      pendingMainVersion.value = null
      pendingContext.value = null
      resetDeps()
    }
  }

  /** 用户在 DependencyConfirmDialog 点击取消（不下载） */
  function handleDependencyClose() {
    showDependencyDialog.value = false
    downloading.value = null
    downloadStage.value = 'idle'
    pendingMainVersion.value = null
    pendingContext.value = null
    resetDeps()
  }

  return {
    downloading,
    downloadStage,
    showDependencyDialog,
    pendingMainVersion,
    depsMap,
    depsLoadingSet,
    depsChecking,
    depsInstalling,
    depsMissing,
    depsUpToDate,
    handleDownload,
    handleDependencyConfirm,
    handleDependencyClose,
    handleLoadDeps,
  }
}
