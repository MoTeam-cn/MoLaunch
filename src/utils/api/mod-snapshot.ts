/**
 * 实例快照 API（version_mods_manager 的 snapshot_* action）
 *
 * 快照只记录 mods 目录清单（文件名 / 大小 / 启用状态），不复制 jar；
 * 回滚时按差异补回或禁用，无法找回的文件会在 unresolved 中返回。
 */
import { VERSION_MODS_ACTIONS, versionModsManager } from './version-mods-manager'

/** 快照里的文件条目 */
export interface SnapshotFile {
  name: string
  size: number
  enabled: boolean
}

/** 快照元数据 */
export interface SnapshotMeta {
  id: string
  /** manual / auto-before-install / auto-before-update / auto-before-delete */
  kind: string
  /** 创建时间（Unix 毫秒） */
  created_at: number
  files: SnapshotFile[]
}

/** 快照差异 */
export interface SnapshotDiff {
  id: string
  /** 快照里有、当前没有 */
  missing: string[]
  /** 当前有、快照里没有 */
  added: string[]
  /** 启用状态不同 */
  toggled: string[]
}

/** 回滚结果 */
export interface SnapshotRestoreResult {
  restored: string[]
  disabled: string[]
  unresolved: string[]
}

/** 创建快照 */
export async function createModSnapshot(
  versionId: string,
  kind: string,
  note?: string,
): Promise<SnapshotMeta> {
  return versionModsManager<SnapshotMeta>(VERSION_MODS_ACTIONS.SNAPSHOT_CREATE, {
    versionId,
    kind,
    note: note ?? null,
  })
}

/** 列出快照（新的在前） */
export async function listModSnapshots(versionId: string): Promise<SnapshotMeta[]> {
  return versionModsManager<SnapshotMeta[]>(VERSION_MODS_ACTIONS.SNAPSHOT_LIST, { versionId })
}

/** 对比快照与当前状态 */
export async function diffModSnapshot(versionId: string, id: string): Promise<SnapshotDiff> {
  return versionModsManager<SnapshotDiff>(VERSION_MODS_ACTIONS.SNAPSHOT_DIFF, { versionId, id })
}

/** 回滚到指定快照 */
export async function restoreModSnapshot(
  versionId: string,
  id: string,
): Promise<SnapshotRestoreResult> {
  return versionModsManager<SnapshotRestoreResult>(VERSION_MODS_ACTIONS.SNAPSHOT_RESTORE, {
    versionId,
    id,
  })
}

/** 删除快照 */
export async function deleteModSnapshot(versionId: string, id: string): Promise<void> {
  return versionModsManager<void>(VERSION_MODS_ACTIONS.SNAPSHOT_DELETE, { versionId, id })
}
