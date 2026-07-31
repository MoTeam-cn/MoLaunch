/**
 * 全局文件拖拽 - 状态与路径工具
 *
 * 模块级单例状态：同一时刻只可能有一个拖拽会话，全局共享。
 * 路径工具函数（getExtension / getFileName / getFileNameWithoutExt）暴露给
 * handlers 与外部测试使用。
 */

import { reactive, readonly } from 'vue'

/** 支持的整合包扩展名 */
export const MODPACK_EXTENSIONS = ['zip', 'mrpack']
/** 支持的 Mod 扩展名（与 install_mod 命令一致） */
export const MOD_EXTENSIONS = ['jar', 'litemod', 'disabled', 'old']

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
export const dragState = reactive<DragDropState>({
  active: false,
  hint: '',
  kind: 'unknown',
})

/** 暴露给 DragOverlay 的只读状态 */
export function useDragDropState() {
  return readonly(dragState)
}

/** 从路径提取小写扩展名（不含 .） */
export function getExtension(path: string): string {
  const lower = path.toLowerCase()
  const dot = lower.lastIndexOf('.')
  return dot >= 0 ? lower.slice(dot + 1) : ''
}

/** 从路径提取文件名（含扩展名） */
export function getFileName(path: string): string {
  // 兼容 Windows 反斜杠和 Unix 正斜杠
  const sep = path.includes('\\') ? '\\' : '/'
  const parts = path.split(sep).filter(Boolean)
  return parts[parts.length - 1] ?? path
}

/** 从路径提取不含扩展名的文件名 */
export function getFileNameWithoutExt(path: string): string {
  const name = getFileName(path)
  const dot = name.lastIndexOf('.')
  return dot > 0 ? name.slice(0, dot) : name
}

/** 根据 paths 预判拖拽类型与提示文案（enter 时调用） */
export function classifyDrag(paths: string[]): { kind: DragDropState['kind']; hint: string } {
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
export function hideOverlay(): void {
  dragState.active = false
  dragState.hint = ''
  dragState.kind = 'unknown'
}

// 暴露给 App.vue 使用的工具函数（便于测试）
export const dragDropUtils = {
  getExtension,
  getFileName,
  getFileNameWithoutExt,
}
