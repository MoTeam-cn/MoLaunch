/**
 * Mod 列表 - 单项操作切片
 *
 * 启用/禁用、删除、安装、打开目录、打开文件位置。依赖由主 composable 传入。
 */
import { type Ref, type ComputedRef } from 'vue'
import * as tauri from '@/utils/tauri'
import { pickFile } from '@/utils/fileDialog'
import { toastSuccess, toastError, toastInfo } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { modTitle } from '@/utils/mod-display'
import type { ModInfo } from '@/utils/tauri'

export interface UseModListItemOpsDeps {
  /** 当前选中的版本 ID（来自 useVersionSettings） */
  selectedId: ComputedRef<string | null>
  /** Mod 本地名称显示风格（0=文件名 1=译名 2=译名+文件名） */
  modLocalNameStyle: Ref<number>
  /** Mod 列表（原地更新用） */
  mods: Ref<ModInfo[]>
  /** 列表重新加载（删除/安装后刷新） */
  loadMods: (silent?: boolean) => Promise<void>
}

export function useModListItemOps(deps: UseModListItemOpsDeps) {
  const { selectedId, modLocalNameStyle, mods, loadMods } = deps

  /**
   * 启用/禁用 Mod
   *
   * 核心设计：**原地更新 mod 字段，不重新加载列表**。
   * 后端 toggle_mod 返回新文件名，前端按 file_name 找到对应 mod 原地更新三个字段：
   * - `file_name`：禁用后变 `xxx.jar.disabled`，启用后变回 `xxx.jar`
   * - `is_enabled`：取反
   * - `enabled_name`：保持不变（永远是去后缀的名称）
   *
   * 这样 mod 在列表中的位置完全不动，project 字段也保留。
   */
  async function handleToggleMod(mod: ModInfo) {
    if (!selectedId.value) return
    const enable = !mod.is_enabled
    try {
      const newFileName = await tauri.toggleMod(selectedId.value, mod.file_name, enable)
      const idx = mods.value.findIndex(m => m.file_name === mod.file_name)
      if (idx !== -1) {
        mods.value[idx] = {
          ...mods.value[idx],
          file_name: newFileName,
          is_enabled: enable,
        }
      }
      toastSuccess(`${enable ? '已启用' : '已禁用'}：${mod.enabled_name}`)
    } catch (e) {
      toastError(`操作失败：${String(e)}`)
    }
  }

  function handleDeleteMod(mod: ModInfo) {
    if (!selectedId.value) return
    showConfirm(
      '删除 Mod',
      `确定要删除 "${modTitle(mod, modLocalNameStyle.value)}" 吗？此操作不可恢复。`,
      async () => {
        try {
          await tauri.deleteMod(selectedId.value!, mod.file_name)
          toastSuccess(`Mod 已删除：${mod.enabled_name}`)
          await loadMods()
        } catch (e) {
          toastError(`删除失败：${String(e)}`)
        }
      },
    )
  }

  async function handleInstallMod() {
    if (!selectedId.value) return
    try {
      const files = await pickFile({
        title: '选择要安装的 Mod',
        filters: [
          { name: 'Mod 文件', extensions: ['jar', 'litemod', 'disabled', 'old'] },
        ],
      })
      if (!files) { toastInfo('已取消安装'); return }
      await tauri.installMod(selectedId.value, files)
      toastSuccess('Mod 安装成功')
      await loadMods()
    } catch (e) {
      toastError(`安装失败：${String(e)}`)
    }
  }

  async function handleOpenModsDir() {
    if (!selectedId.value) return
    try {
      await tauri.openModsDir(selectedId.value)
    } catch (e) {
      toastError(`打开文件夹失败：${String(e)}`)
    }
  }

  /** 打开单个 Mod 的文件位置 */
  async function handleOpenFile(mod: ModInfo) {
    if (!selectedId.value) return
    try {
      await tauri.revealModFile(selectedId.value, mod.file_name)
    } catch (e) {
      toastError(`打开文件位置失败：${String(e)}`)
    }
  }

  return {
    handleToggleMod,
    handleDeleteMod,
    handleInstallMod,
    handleOpenModsDir,
    handleOpenFile,
  }
}
