/**
 * 整合包一键安装流程（联机大厅阶段 4 新增）
 *
 * 加入方在房间详情或大厅浏览页点击「一键安装」时调用。流程：
 * 1. 平台字符串映射（`curseforge` → `CurseForge` / `modrinth` → `Modrinth`）
 * 2. 弹窗询问安装名称（默认整合包名）
 * 3. 跳转下载页 → versionStore.startDownload 占位
 * 4. getProjectVersions 反查平台版本列表
 * 5. 按 fileId 匹配定位 ResourceVersion
 * 6. installModpack（下载 + 解析 + 依赖 mods + overrides）
 * 7. installMerged（安装 MC 本体 + 加载器）
 * 8. 失败时 showModal + finishDownload，遵循项目统一流程
 *
 * # 复用约定
 * - 与 `useDragDrop.runModpackInstall` / `ResourceDetail.handleInstallModpack` 共享
 *   installMerged / showModal / versionStore 调用约定，但入参为平台元数据
 *   （无本地文件路径或资源详情页上下文），故独立封装。
 * - 阶段 5 大厅浏览页卡片「加入并安装」可复用此 composable。
 */
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useVersionStore } from '@/stores/version'
import { getProjectVersions, installModpack } from '@/utils/api/community'
import { installMerged } from '@/utils/api/loader'
import { showPrompt, showModal } from '@/utils/modal'
import { toastSuccess, toastError, toastInfo } from '@/utils/toast'
import { isCancelledError } from '@/utils/async'
import type { ModpackMeta } from '@/types/online'
import type { Platform, ResourceVersion } from '@/types/community'

/** 平台字符串映射：后端 modpack.source 小写 → 前端 Platform 类型 */
function toPlatform(source: string): Platform | null {
  if (source === 'curseforge') return 'CurseForge'
  if (source === 'modrinth') return 'Modrinth'
  return null
}

/** 在版本列表中按 fileId 匹配定位 ResourceVersion */
function findVersionByFileId(versions: ResourceVersion[], fileId: string): ResourceVersion | null {
  return versions.find((v) => v.id === fileId) ?? null
}

/**
 * 整合包一键安装 composable
 *
 * @returns `install(modpack)`：触发安装流程；`installing`：安装中标志
 */
export function useModpackInstall() {
  const router = useRouter()
  const versionStore = useVersionStore()
  const installing = ref(false)

  async function install(modpack: ModpackMeta): Promise<boolean> {
    if (installing.value) return false

    const platform = toPlatform(modpack.source)
    if (!platform) {
      toastError(`不支持的整合包来源：${modpack.source}`)
      return false
    }

    // 1. 弹窗询问安装名称（默认整合包名）
    const defaultName = modpack.name || `modpack-${modpack.fileId.slice(0, 8)}`
    const instanceName = await new Promise<string | null>((resolve) => {
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
    if (!instanceName) return false

    installing.value = true
    versionStore.startDownload(instanceName)
    router.push('/apps/downloads')

    try {
      // 2. 反查平台版本列表
      const versions = await getProjectVersions(platform, modpack.projectId)
      const target = findVersionByFileId(versions, modpack.fileId)
      if (!target) {
        throw new Error(
          `未在 ${platform} 项目 ${modpack.projectId} 下找到文件 ID ${modpack.fileId}，可能已下架`,
        )
      }

      // 3. installModpack：下载 + 解析 + 依赖 mods + overrides
      const result = await installModpack({
        platform,
        downloadUrl: target.download_url,
        fileName: target.file_name,
        instanceName,
        projectId: modpack.projectId,
        fileId: modpack.fileId,
        modpackVersion: modpack.modpackVersion ?? target.version,
        fileSize: modpack.fileSize ?? target.size,
        name: modpack.name,
      })

      // 4. installMerged：安装 MC 本体 + 加载器
      toastSuccess(`整合包解析完成，开始安装 MC ${result.gameVersion}...`)
      await installMerged(
        result.gameVersion,
        result.loader === 'forge' ? result.loaderVersion : undefined,
        result.loader === 'neoforge' ? result.loaderVersion : undefined,
        result.loader === 'fabric' || result.loader === 'quilt' ? result.loaderVersion : undefined,
        result.loader === 'optifine' ? result.loaderVersion : undefined,
        undefined,
        instanceName,
      )
      toastSuccess(`整合包 ${instanceName} 安装完成`)
      return true
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err)
      // 用户主动取消：仅 toast 提示并退出下载页，不弹错误窗
      if (isCancelledError(err)) {
        toastInfo('下载已取消')
        versionStore.finishDownload()
        return false
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
      return false
    } finally {
      installing.value = false
    }
  }

  return { installing, install }
}
