/**
 * 全局文件拖拽安装 composable
 *
 * 在 App.vue 根组件调用 `useDragDrop()` 注册 Tauri v2 `onDragDropEvent`，
 * 根据文件扩展名路由到不同处理逻辑：
 *
 * - `.zip` / `.mrpack` → 整合包安装（弹窗输入实例名 → installLocalModpack → installMerged）
 * - `.jar` / `.litemod` / `.disabled` / `.old` → Mod 安装（弹窗选择目标版本 → installMod）
 * - `.rar` → 提示用户解压后重试
 * - 其他 → 提示无法识别
 *
 * 拖拽进入时通过 `dragState` 暴露 enter/over/leave 状态，驱动 DragOverlay 全局遮蔽层。
 *
 * 参考 PCL2 FormMain.xaml.vb 的 FileDrag 路由分发实现。
 */

import { onMounted, onUnmounted, reactive, readonly } from 'vue'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import router from '@/router'
import { installLocalModpack } from '@/utils/api/community'
import { installMod } from '@/utils/api/personalization'
import { listInstalledVersionsWithType, type InstalledVersionInfo } from '@/utils/api/version'
import { installMerged } from '@/utils/api/loader'
import { showError, showInfo, showModal, showPrompt } from '@/utils/modal'
import { toastError, toastSuccess } from '@/utils/toast'
import { useVersionStore } from '@/stores/version'
import type { InstallModpackResult } from '@/types/community'

/** 支持的整合包扩展名 */
const MODPACK_EXTENSIONS = ['zip', 'mrpack']
/** 支持的 Mod 扩展名（与 install_mod 命令一致） */
const MOD_EXTENSIONS = ['jar', 'litemod', 'disabled', 'old']

/** 拖拽状态：用于驱动 DragOverlay 全局遮蔽层 */
export interface DragDropState {
  /** 是否正在拖拽中（enter 后未 drop/leave） */
  active: boolean
  /** 当前拖拽提示文案（如 "松开以安装整合包"） */
  hint: string
  /** 拖拽类型分类：modpack / mod / multi-mod / unknown */
  kind: 'modpack' | 'mod' | 'multi-mod' | 'unknown'
}

/** 模块级单例状态：同一时刻只可能有一个拖拽会话，全局共享 */
const dragState = reactive<DragDropState>({
  active: false,
  hint: '',
  kind: 'unknown',
})

/** 暴露给 DragOverlay 的只读状态 */
export function useDragDropState() {
  return readonly(dragState)
}

/** 从路径提取小写扩展名（不含 .） */
function getExtension(path: string): string {
  const lower = path.toLowerCase()
  const dot = lower.lastIndexOf('.')
  return dot >= 0 ? lower.slice(dot + 1) : ''
}

/** 从路径提取文件名（含扩展名） */
function getFileName(path: string): string {
  // 兼容 Windows 反斜杠和 Unix 正斜杠
  const sep = path.includes('\\') ? '\\' : '/'
  const parts = path.split(sep).filter(Boolean)
  return parts[parts.length - 1] ?? path
}

/** 从路径提取不含扩展名的文件名 */
function getFileNameWithoutExt(path: string): string {
  const name = getFileName(path)
  const dot = name.lastIndexOf('.')
  return dot > 0 ? name.slice(0, dot) : name
}

/** 根据 paths 预判拖拽类型与提示文案（enter 时调用） */
function classifyDrag(paths: string[]): { kind: DragDropState['kind']; hint: string } {
  if (paths.length === 0) {
    return { kind: 'unknown', hint: '正在拖入文件...' }
  }
  if (paths.length === 1) {
    const ext = getExtension(paths[0])
    if (MODPACK_EXTENSIONS.includes(ext)) {
      return { kind: 'modpack', hint: '松开以安装整合包' }
    }
    if (MOD_EXTENSIONS.includes(ext)) {
      return { kind: 'mod', hint: '松开以安装 Mod' }
    }
    if (ext === 'rar') {
      return { kind: 'unknown', hint: '不支持 rar 格式，请使用 zip' }
    }
    return { kind: 'unknown', hint: '不支持的文件类型' }
  }
  // 多文件：必须全部为 Mod
  for (const p of paths) {
    if (!MOD_EXTENSIONS.includes(getExtension(p))) {
      return { kind: 'unknown', hint: '多文件仅支持 .jar/.litemod Mod' }
    }
  }
  return { kind: 'multi-mod', hint: `松开以安装 ${paths.length} 个 Mod` }
}

/** 隐藏遮蔽层 */
function hideOverlay(): void {
  dragState.active = false
  dragState.hint = ''
  dragState.kind = 'unknown'
}

/**
 * 处理整合包拖拽：弹窗输入实例名 → installLocalModpack → installMerged
 *
 * 进度通过 download_state 推送，前端 DownloadPanel 自动展示。
 * 完成后跳转到下载页轮询 install_merged 进度。
 */
async function handleModpackDrop(filePath: string): Promise<void> {
  const defaultName = getFileNameWithoutExt(filePath)

  showPrompt(
    '安装整合包',
    `请输入整合包实例名（将创建 versions/{实例名}/ 目录）：`,
    async (instanceName) => {
      if (!instanceName.trim()) {
        toastError('实例名不能为空')
        return
      }
      await runModpackInstall(filePath, instanceName.trim())
    },
    { defaultValue: defaultName, placeholder: '请输入实例名' },
  )
}

/** 执行整合包安装流程（install_local_modpack → install_merged） */
async function runModpackInstall(filePath: string, instanceName: string): Promise<void> {
  const versionStore = useVersionStore()
  // 跳转到下载页，让用户看到进度
  router.push({ name: 'downloads' })

  let result: InstallModpackResult
  try {
    result = await installLocalModpack({ filePath, instanceName })
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err)
    // 后端已 mark_failed 重置 is_active，前端需 finishDownload 让 Downloads.vue watch 触发 router.back()
    // 用 showModal 支持确认回调，用户点击确定后才退出下载页，避免弹窗一闪而过
    showModal({
      type: 'error',
      title: '整合包安装失败',
      message: msg,
      onConfirm: () => {
        versionStore.finishDownload()
      },
    })
    return
  }

  // 整合包专属部分完成，紧接着调用 install_merged 安装游戏本体
  toastSuccess(`整合包解析完成，开始安装 MC ${result.gameVersion}...`)

  try {
    await installMerged(
      result.gameVersion,
      result.loader === 'forge' ? result.loaderVersion : undefined,
      result.loader === 'neoforge' ? result.loaderVersion : undefined,
      result.loader === 'fabric' ? result.loaderVersion : undefined,
      result.loader === 'optifine' ? result.loaderVersion : undefined,
      undefined,
      instanceName,
    )
    toastSuccess(`整合包 ${instanceName} 安装完成`)
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err)
    // 同上：后端已 mark_failed，前端用 showModal + onConfirm 让用户点击确定后退出下载页
    showModal({
      type: 'error',
      title: '游戏本体安装失败',
      message: `整合包已解压，但游戏本体安装失败：${msg}`,
      onConfirm: () => {
        versionStore.finishDownload()
      },
    })
  }
}

/**
 * 处理 Mod 拖拽：弹窗选择目标版本 → installMod
 *
 * 与 PCL2 不同，MoLaunch 的 install_mod 命令需要 versionId 参数，
 * 因此需要先弹窗让用户选择目标版本。
 */
async function handleModDrop(filePath: string): Promise<void> {
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

/**
 * 处理单文件拖拽（路由分发）
 *
 * 与 PCL2 FormMain.FileDrag 一致的路由逻辑：
 * - 多文件拖拽：必须全部为 jar/litemod/disabled/old，否则提示一次只拖一个
 * - 单文件拖拽：按扩展名路由
 */
async function handleFileDrop(paths: string[]): Promise<void> {
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

  if (MODPACK_EXTENSIONS.includes(ext)) {
    await handleModpackDrop(filePath)
  } else if (MOD_EXTENSIONS.includes(ext)) {
    await handleModDrop(filePath)
  } else if (ext === 'rar') {
    showInfo(
      'RAR 格式不支持',
      'PCL/MoLaunch 无法处理 rar 格式的压缩包，请在解压后重新压缩为 zip 格式再试。',
    )
  } else {
    showInfo(
      '无法识别的文件',
      `不支持的文件扩展名 .${ext}。支持的类型：整合包（.zip/.mrpack）、Mod（.jar/.litemod）。`,
    )
  }
}

/** 处理多个 Mod 文件拖拽：弹窗选择版本后批量安装 */
async function handleMultiModDrop(paths: string[]): Promise<void> {
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
 * 注册全局拖拽事件监听
 *
 * 必须在 onMounted 中调用，onUnmounted 中取消监听。
 * 返回取消监听函数以便测试或手动卸载。
 *
 * 事件流：
 * - enter：拖拽进入窗口，根据 paths 预判类型，显示 DragOverlay
 * - over：拖拽在窗口内移动，保持 DragOverlay 显示
 * - leave：拖拽离开窗口，隐藏 DragOverlay
 * - drop：释放，隐藏 DragOverlay，分发到对应处理函数
 */
export function useDragDrop(): () => void {
  let unlisten: (() => void) | null = null

  onMounted(async () => {
    try {
      const webview = getCurrentWebview()
      unlisten = await webview.onDragDropEvent(async (event) => {
        const payload = event.payload
        switch (payload.type) {
          case 'enter': {
            const paths = payload.paths ?? []
            const { kind, hint } = classifyDrag(paths)
            dragState.active = true
            dragState.kind = kind
            dragState.hint = hint
            return
          }
          case 'over':
            // 保持遮蔽层显示，无需更新
            return
          case 'leave':
            hideOverlay()
            return
          case 'drop': {
            hideOverlay()
            const paths = payload.paths
            if (!paths || paths.length === 0) return
            await handleFileDrop(paths)
            return
          }
        }
      })
    } catch (err) {
      // onDragDropEvent 在某些平台可能不可用，静默失败
      console.warn('[DragDrop] 注册拖拽事件监听失败:', err)
    }
  })

  onUnmounted(() => {
    unlisten?.()
    unlisten = null
  })

  return () => {
    unlisten?.()
    unlisten = null
  }
}

// 暴露给 App.vue 使用的工具函数（便于测试）
export const dragDropUtils = {
  getExtension,
  getFileName,
  getFileNameWithoutExt,
}
