/**
 * 资源包/光影管理操作 composable（编排层）
 *
 * 聚合 usePackList + 单项操作 handlers（启停/删除/安装/打开目录/定位文件）
 * + 详情弹窗桥接（usePackDetailQuery）+ 更新/更改版本对话框状态。
 */

import { type ComputedRef, type Ref, ref } from 'vue'
import { usePackList } from './usePackList'
import { usePackDetailQuery } from './usePackDetailQuery'
import { pickFile } from '@/utils/fileDialog'
import { toastSuccess, toastError, toastInfo } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import * as tauri from '@/utils/tauri'
import type { PackInfo, PackKind } from '@/utils/tauri'

interface UsePackOperationsOptions {
  /** 当前选中的版本 ID（来自 useVersionSettings） */
  selectedId: ComputedRef<string | null>
  /** 内容类型（资源包 / 光影） */
  kind: Ref<PackKind>
}

export function usePackOperations(options: UsePackOperationsOptions) {
  const { selectedId, kind } = options
  const list = usePackList(options)

  // 详情弹窗（关联 CF/MR 平台工程时展示，无 project 则本地信息弹窗）
  const { detailVisible, detailProject, detailLoadingFor, handleShowInfo } = usePackDetailQuery()

  // 更新/更改版本对话框状态
  const updatePackFor = ref<PackInfo | null>(null)
  const updateVisible = ref(false)

  function openUpdateDialog(pack: PackInfo) {
    updatePackFor.value = pack
    updateVisible.value = true
  }

  /** 详情按钮事件桥接：把 packs / isPreloadDone refs 转发给 composable */
  function onShowInfo(pack: PackInfo) {
    handleShowInfo(pack, list.packs, list.isPreloadDone)
  }

  /** 更新安装完成：静默重载列表 + 重新触发预加载（为新版本重新查询平台工程） */
  async function onPackUpdated() {
    await list.loadPacks(true)
    if (selectedId.value) {
      tauri.preloadPacksDetail(selectedId.value, kind.value).catch(e => {
        console.debug('[PackTab] 更新后预加载启动失败:', e)
      })
    }
  }

  /**
   * 启用/禁用 Pack（原地更新字段，不重载列表）
   * 后端 toggle_pack 返回新文件名并同步 options.txt。
   */
  async function handleToggle(pack: PackInfo) {
    if (!selectedId.value) return
    const enable = !pack.is_enabled
    try {
      const newFileName = await tauri.togglePack(selectedId.value, pack.file_name, enable, kind.value)
      const idx = list.packs.value.findIndex(p => p.file_name === pack.file_name)
      if (idx !== -1) {
        list.packs.value[idx] = { ...list.packs.value[idx], file_name: newFileName, is_enabled: enable }
      }
      toastSuccess(`${enable ? '已启用' : '已禁用'}：${pack.enabled_name}`)
    } catch (e) {
      toastError(`操作失败：${String(e)}`)
    }
  }

  function handleDelete(pack: PackInfo) {
    if (!selectedId.value) return
    showConfirm('删除', `确定要删除 "${pack.enabled_name}" 吗？此操作不可恢复。`, async () => {
      try {
        await tauri.deletePack(selectedId.value!, pack.file_name, kind.value)
        toastSuccess(`已删除：${pack.enabled_name}`)
        await list.loadPacks()
      } catch (e) {
        toastError(`删除失败：${String(e)}`)
      }
    })
  }

  async function handleInstall() {
    if (!selectedId.value) return
    try {
      const file = await pickFile({
        title: kind.value === 'resourcepack' ? '选择要安装的资源包' : '选择要安装的光影包',
        filters: [{ name: 'ZIP 文件', extensions: ['zip', 'disabled', 'old'] }],
      })
      if (!file) { toastInfo('已取消安装'); return }
      await tauri.installPack(selectedId.value, file, kind.value)
      toastSuccess('安装成功')
      await list.loadPacks()
    } catch (e) {
      toastError(`安装失败：${String(e)}`)
    }
  }

  async function handleOpenDir() {
    if (!selectedId.value) return
    try {
      await tauri.openPacksDir(selectedId.value, kind.value)
    } catch (e) {
      toastError(`打开文件夹失败：${String(e)}`)
    }
  }

  async function handleOpenFile(pack: PackInfo) {
    if (!selectedId.value) return
    try {
      await tauri.revealPackFile(selectedId.value, pack.file_name, kind.value)
    } catch (e) {
      toastError(`定位文件失败：${String(e)}`)
    }
  }

  return {
    ...list,
    detailVisible, detailProject, detailLoadingFor,
    updatePackFor, updateVisible,
    openUpdateDialog, onShowInfo, onPackUpdated,
    handleToggle, handleDelete, handleInstall, handleOpenDir, handleOpenFile,
  }
}
