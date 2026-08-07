/**
 * 全局文件拖拽 - 文件命中与路由分发
 *
 * 仅负责根据文件数量与扩展名选择对应安装处理器；具体安装业务由 handlers.ts 提供。
 */
import { detectPackageType } from '@/utils/api/frp-manager'
import { showError } from '@/utils/modal'
import {
  MODPACK_EXTENSIONS,
  MOD_EXTENSIONS,
  getExtension,
} from './state'
import {
  handleFrpProviderDrop,
  handleModDrop,
  handleModpackDrop,
  handleMultiModDrop,
} from './handlers'

/**
 * 处理文件拖拽路由。
 *
 * 多文件仅允许 Mod；单个 zip 需要读取内容特征后区分整合包与 Frp 厂商包。
 */
export async function handleFileDrop(paths: string[]): Promise<void> {
  if (paths.length === 0) return

  if (paths.length > 1) {
    for (const path of paths) {
      if (!MOD_EXTENSIONS.includes(getExtension(path))) {
        showError(
          '多文件拖拽限制',
          '一次只能拖入一个文件，或多个 .jar / .litemod / .disabled / .old Mod 文件。',
        )
        return
      }
    }
    await handleMultiModDrop(paths)
    return
  }

  const filePath = paths[0]
  const ext = getExtension(filePath)

  if (ext === 'zip') {
    let type: 'frp_provider' | 'modpack' | 'unknown' = 'unknown'
    let providerName: string | undefined
    try {
      const result = await detectPackageType(filePath)
      type = result.type
      providerName = result.providerName
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
        '无法识别该 zip 包的类型。支持：整合包（.zip/.mrpack）、Frp 厂商包（含 manifest.json 且具备 frp 特征字段）。',
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
