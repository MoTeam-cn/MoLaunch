/**
 * 全局文件拖拽 - 文件类型分发与安装处理
 *
 * 根据扩展名路由：`.zip`/`.mrpack` → 整合包、`.jar`/`.litemod`/`.disabled`/`.old` → Mod、
 * `.rar` → 拒绝并提示解压、其他 → 提示无法识别。
 * 安装执行流水线在 helpers.ts（formatToLabel / runModpackInstall）。
 */
import { previewLocalModpack } from '@/utils/api/community'
import { installMod } from '@/utils/api/personalization'
import { listInstalledVersionsWithType, type InstalledVersionInfo } from '@/utils/api/version'
import {
  detectPackageType,
  installProviderFromZip,
} from '@/utils/api/frp-manager'
import { showConfirmAsync, showError, showInfo, showModal, showPrompt } from '@/utils/modal'
import { toastError, toastSuccess } from '@/utils/toast'
import type { ModpackPreview } from '@/types/community'
import {
  MODPACK_EXTENSIONS,
  MOD_EXTENSIONS,
  getExtension,
  getFileNameWithoutExt,
} from './state'
import { formatToLabel, runModpackInstall } from './helpers'

export { formatToLabel, runModpackInstall } from './helpers'

/** 安装完成后的全局刷新通知（Frp 页面监听并重载厂商列表） */
function notifyProvidersChanged(): void {
  window.dispatchEvent(new CustomEvent('frp:providers-changed'))
}

/**
 * 处理 frp 厂商包拖拽：确认后安装，复用存量增量更新逻辑
 *
 * 若该厂商已安装且包版本号变化，后端 install_provider_from_zip 自动执行
 * 增量更新（同版本返回"已是最新版本"）。
 */
export async function handleFrpProviderDrop(
  filePath: string,
  providerName?: string,
): Promise<void> {
  const label = providerName ? `「${providerName}」` : ''
  const confirmed = await showConfirmAsync(
    '安装 Frp 厂商包',
    `检测到 ${label}frp 厂商包，是否安装？\n\n若该厂商已安装且包版本更新，将自动执行增量更新（仅替换变更文件，保留 frpc 与认证数据）。`,
  )
  if (!confirmed) return

  try {
    await installProviderFromZip(filePath)
    toastSuccess('厂商安装/更新成功')
    notifyProvidersChanged()
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err)
    showError('厂商安装失败', msg)
  }
}

/**
 * 处理整合包拖拽：预览整合包 → 弹窗输入实例名 → 询问可选 Mod → installLocalModpack → installMerged
 *
 * 拖拽安装时弹窗询问用户是否下载可选 Mod：
 * - CF: required=false 的 Mod 列表
 * - MR: env.client=optional 的文件列表
 * - HMCL/MMC/MCBBS: 无可选概念，直接安装
 *
 * 进度通过 download_state 推送，前端 DownloadPanel 自动展示。
 */
export async function handleModpackDrop(filePath: string): Promise<void> {
  // 1. 预览整合包：获取格式 + 可选 Mod 列表
  let preview: ModpackPreview
  try {
    preview = await previewLocalModpack(filePath)
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err)
    showError('整合包预览失败', msg)
    return
  }

  const defaultName = getFileNameWithoutExt(filePath)
  const loaderInfo = preview.loader
    ? ` / ${preview.loader}${preview.loaderVersion ? ' ' + preview.loaderVersion : ''}`
    : ''
  const formatLabel = formatToLabel(preview.format)

  // 2. 弹窗输入实例名
  showPrompt(
    '安装整合包',
    `检测到 ${formatLabel} 整合包（游戏 ${preview.gameVersion}${loaderInfo}）\n请输入整合包实例名（将创建 versions/{实例名}/ 目录）：`,
    async (instanceName) => {
      if (!instanceName.trim()) {
        toastError('实例名不能为空')
        return
      }
      const name = instanceName.trim()

      // 3. 无可选 Mod：直接安装（后端默认 includeOptional=true）
      if (preview.optionalMods.length === 0) {
        await runModpackInstall(filePath, name)
        return
      }

      // 4. 有可选 Mod：弹窗询问是否下载
      const modList = preview.optionalMods
        .slice(0, 20)
        .map((m) => `  - ${m.displayName}`)
        .join('\n')
      const moreHint =
        preview.optionalMods.length > 20
          ? `\n  ...等 ${preview.optionalMods.length} 个`
          : ''
      showModal({
        type: 'info',
        title: '下载可选 Mod',
        message: `整合包含 ${preview.optionalMods.length} 个可选 Mod：\n${modList}${moreHint}\n\n是否下载这些可选 Mod？`,
        confirmText: '下载',
        cancelText: '不下载',
        showCancel: true,
        onConfirm: () => {
          // 用户选择下载可选 Mod
          runModpackInstall(filePath, name, true)
        },
        onCancel: () => {
          // 用户选择不下载可选 Mod
          runModpackInstall(filePath, name, false)
        },
      })
    },
    { defaultValue: defaultName, placeholder: '请输入实例名' },
  )
}

/**
 * 处理 Mod 拖拽：弹窗选择目标版本 → installMod
 *
 * install_mod 命令需要 versionId 参数，因此需要先弹窗让用户选择目标版本。
 */
export async function handleModDrop(filePath: string): Promise<void> {
  let versions: InstalledVersionInfo[] = []
  try {
    versions = await listInstalledVersionsWithType()
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err)
    showError('获取版本列表失败', msg)
    return
  }

  if (versions.length === 0) {
    showInfo(
      '无可用版本',
      '当前没有任何已安装的 Minecraft 版本。请先下载一个版本，再拖入 Mod 文件。',
    )
    return
  }

  // 构造版本列表文案
  const versionListText = versions.map((v, i) => `${i + 1}. ${v.id}`).join('\n')

  // 使用 showPrompt 让用户输入版本编号
  showPrompt(
    '选择目标版本',
    `请输入要安装到的版本编号（1~${versions.length}）：\n\n${versionListText}`,
    async (input) => {
      const idx = parseInt(input.trim(), 10)
      if (isNaN(idx) || idx < 1 || idx > versions.length) {
        toastError(`请输入 1~${versions.length} 之间的数字`)
        return
      }
      const targetVersion = versions[idx - 1].id
      try {
        await installMod(targetVersion, filePath)
        toastSuccess(`Mod 已安装到 ${targetVersion}`)
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err)
        showError('Mod 安装失败', msg)
      }
    },
    { defaultValue: '1', placeholder: `1~${versions.length}` },
  )
}

/** 处理多个 Mod 文件拖拽：弹窗选择版本后批量安装 */
export async function handleMultiModDrop(paths: string[]): Promise<void> {
  let versions: InstalledVersionInfo[] = []
  try {
    versions = await listInstalledVersionsWithType()
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err)
    showError('获取版本列表失败', msg)
    return
  }

  if (versions.length === 0) {
    showInfo(
      '无可用版本',
      '当前没有任何已安装的 Minecraft 版本。请先下载一个版本，再拖入 Mod 文件。',
    )
    return
  }

  const versionListText = versions.map((v, i) => `${i + 1}. ${v.id}`).join('\n')

  showPrompt(
    '选择目标版本',
    `将安装 ${paths.length} 个 Mod 文件，请输入目标版本编号（1~${versions.length}）：\n\n${versionListText}`,
    async (input) => {
      const idx = parseInt(input.trim(), 10)
      if (isNaN(idx) || idx < 1 || idx > versions.length) {
        toastError(`请输入 1~${versions.length} 之间的数字`)
        return
      }
      const targetVersion = versions[idx - 1].id
      let success = 0
      let failed = 0
      for (const p of paths) {
        try {
          await installMod(targetVersion, p)
          success++
        } catch {
          failed++
        }
      }
      if (failed === 0) {
        toastSuccess(`已安装 ${success} 个 Mod 到 ${targetVersion}`)
      } else {
        showError(
          '部分 Mod 安装失败',
          `成功 ${success} 个，失败 ${failed} 个（可能存在同名文件）`,
        )
      }
    },
    { defaultValue: '1', placeholder: `1~${versions.length}` },
  )
}

/**
 * 处理单文件拖拽（路由分发）
 *
 * - 多文件拖拽：必须全部为 jar/litemod/disabled/old，否则提示一次只拖一个
 * - 单文件拖拽：按扩展名路由
 */
export async function handleFileDrop(paths: string[]): Promise<void> {
  if (paths.length === 0) return

  // 多文件拖拽：必须全部为 Mod 文件
  if (paths.length > 1) {
    for (const p of paths) {
      const ext = getExtension(p)
      if (!MOD_EXTENSIONS.includes(ext)) {
        showError(
          '多文件拖拽限制',
          '一次只能拖入一个文件，或多个 .jar / .litemod / .disabled / .old Mod 文件。',
        )
        return
      }
    }
    // 多个 Mod 文件逐个安装到同一版本
    await handleMultiModDrop(paths)
    return
  }

  // 单文件拖拽：按扩展名路由
  const filePath = paths[0]
  const ext = getExtension(filePath)

  if (ext === 'zip') {
    // zip 可能是整合包也可能是 frp 厂商包：先读内容特征判断，再路由
    let type: 'frp_provider' | 'modpack' | 'unknown' = 'unknown'
    let providerName: string | undefined
    try {
      const res = await detectPackageType(filePath)
      type = res.type
      providerName = res.providerName
    } catch {
      // 检测失败时不阻塞，按无法识别处理
    }
    if (type === 'frp_provider') {
      await handleFrpProviderDrop(filePath, providerName)
    } else if (type === 'modpack') {
      await handleModpackDrop(filePath)
    } else {
      showError(
        '无法识别压缩包',
        `无法识别该 zip 包的类型。支持：整合包（.zip/.mrpack）、Frp 厂商包（含 manifest.json 且具备 frp 特征字段）。`,
      )
    }
  } else if (MODPACK_EXTENSIONS.includes(ext)) {
    await handleModpackDrop(filePath)
  } else if (MOD_EXTENSIONS.includes(ext)) {
    await handleModDrop(filePath)
  } else if (ext === 'rar') {
    showError(
      'RAR 格式不支持',
      'MoLaunch 无法处理 rar 格式的压缩包，请解压后重新压缩为 zip 格式再试。',
    )
  } else {
    showError(
      '无法识别的文件',
      `不支持的文件扩展名 .${ext}。支持的类型：整合包（.zip/.mrpack）、Mod（.jar/.litemod）。`,
    )
  }
}