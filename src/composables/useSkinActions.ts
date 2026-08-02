/**
 * 皮肤/披风操作交互切片（从 useSkinOperations.ts 抽取）
 *
 * 负责上传/披风/删除/选择本地皮肤等全部交互动作（三分流：微软 / 外置 / 离线），
 * 以及上传装备后的通用刷新流程（runWithRefresh：执行 → 提示 → 重新加载 + 触发头像刷新）。
 *
 * 依赖注入：
 * - state（UseSkinState）：皮肤状态切片创建的全部响应式状态与派生 computed
 * - loadInfo：主文件（useSkinOperations.ts）提供的信息加载函数，runWithRefresh 复用
 * - uuid / username / serverUrl：来自 authStore.currentUser 的 computed
 */

import type { ComputedRef } from 'vue'
import {
  uploadSkin, equipCape, unequipCape, downloadUrlToFile,
} from '@/utils/tauri'
import { pickFile, pickSavePath } from '@/utils/fileDialog'
import { toastSuccess, toastError } from '@/utils/toast'
import { saveCustomSkin } from '@/utils/api/auth'
import {
  authlibDeleteCape, authlibDeleteSkin, authlibUploadCape, authlibUploadSkin,
} from '@/utils/api/authlib'
import {
  setLocalSkinName, bumpSkinVersion, parseSkinUrl, parseSkinVariant,
} from '@/utils/default-skin'
import type { UseSkinState } from './useSkinState'

export interface UseSkinActionsOptions {
  /** 当前账号 UUID（来自 authStore.currentUser.uuid） */
  uuid: ComputedRef<string>
  /** 当前账号用户名（来自 authStore.currentUser.name） */
  username: ComputedRef<string>
  /** yggdrasil API 根地址（仅外置账号有效，其他账号为空字符串） */
  serverUrl: ComputedRef<string>
  /** 皮肤状态切片创建的全部响应式状态 */
  state: UseSkinState
  /** 信息加载函数（主文件提供），上传/装备成功后用于刷新 UI */
  loadInfo: () => Promise<void>
}

export function useSkinActions(options: UseSkinActionsOptions) {
  const { uuid, username, serverUrl, state, loadInfo } = options

  /** 上传/装备/取消操作后的通用流程：执行 → 提示 → 重新加载 + 触发头像刷新 */
  async function runWithRefresh(successMsg: string, fn: () => Promise<unknown>) {
    state.uploading.value = true
    try {
      await fn()
      toastSuccess(successMsg)
      await loadInfo()
      bumpSkinVersion()
    } catch (e) {
      toastError(String(e))
    } finally {
      state.uploading.value = false
    }
  }

  /**
   * 上传皮肤（三分流）
   *
   * - 微软：uploadSkin（Mojang API）
   * - 外置：authlibUploadSkin（yggdrasil API）
   * - 离线：onUploadCustomSkin（保存到本地 app data）
   *
   * 离线分支委托 onUploadCustomSkin 处理（避免重复弹文件选择对话框）。
   */
  async function pickAndUpload() {
    if (state.isOffline.value) {
      await onUploadCustomSkin()
      return
    }
    if (state.isAuthlib.value) {
      if (!state.canUploadSkin.value) {
        toastError('此服务器不允许上传皮肤')
        return
      }
      try {
        const filePath = await pickFile({ title: '选择皮肤 PNG 文件', filters: [{ name: 'PNG 图片', extensions: ['png'] }] })
        if (!filePath) return
        await runWithRefresh('皮肤上传成功', async () => {
          if (!serverUrl.value) throw new Error('外置账号缺少 server_url')
          const model: 'slim' | 'default' = state.variant.value === 'slim' ? 'slim' : 'default'
          await authlibUploadSkin(serverUrl.value, uuid.value, filePath, model)
        })
      } catch (e) {
        toastError(String(e))
      }
      return
    }
    // 微软账号
    try {
      const filePath = await pickFile({ title: '选择皮肤 PNG 文件', filters: [{ name: 'PNG 图片', extensions: ['png'] }] })
      if (!filePath) return
      await runWithRefresh('皮肤上传成功', () => uploadSkin(filePath, state.variant.value))
    } catch (e) {
      toastError(String(e))
    }
  }

  /** 外置账号：删除皮肤（恢复默认） */
  async function onDeleteAuthlibSkin() {
    if (!state.isAuthlib.value || !serverUrl.value) return
    await runWithRefresh('皮肤已删除', async () => {
      await authlibDeleteSkin(serverUrl.value, uuid.value)
    })
  }

  /** 外置账号：上传披风 */
  async function onUploadAuthlibCape() {
    if (!state.isAuthlib.value || !state.canUploadCape.value || !serverUrl.value) return
    try {
      const filePath = await pickFile({ title: '选择披风 PNG 文件', filters: [{ name: 'PNG 图片', extensions: ['png'] }] })
      if (!filePath) return
      await runWithRefresh('披风上传成功', async () => {
        await authlibUploadCape(serverUrl.value, uuid.value, filePath)
      })
    } catch (e) {
      toastError(String(e))
    }
  }

  /** 外置账号：删除披风 */
  async function onDeleteAuthlibCape() {
    if (!state.isAuthlib.value || !serverUrl.value) return
    await runWithRefresh('披风已删除', async () => {
      await authlibDeleteCape(serverUrl.value, uuid.value)
    })
  }

  async function onEquipCape(capeId: string) {
    await runWithRefresh('披风已装备', () => equipCape(capeId))
  }

  async function onUnequipCape() {
    await runWithRefresh('披风已取消', () => unequipCape())
  }

  /** 离线账号：选择本地默认皮肤 */
  async function onSelectLocalSkin(skinName: string) {
    await setLocalSkinName(uuid.value, skinName)
    state.selectedLocalSkin.value = skinName
    const url = parseSkinUrl(skinName)
    if (url) state.skinUrl.value = url
    state.variant.value = parseSkinVariant(skinName)
    bumpSkinVersion()
    toastSuccess(`已切换为 ${skinName} 皮肤`)
  }

  /** 离线账号：上传自定义皮肤 PNG 文件 */
  async function onUploadCustomSkin() {
    try {
      const filePath = await pickFile({ title: '选择皮肤 PNG 文件', filters: [{ name: 'PNG 图片', extensions: ['png'] }] })
      if (!filePath) return

      state.uploading.value = true
      // 保存到 app data 并获取 skin 字段值
      const skinValue = await saveCustomSkin(uuid.value, filePath, state.variant.value)

      // 更新内存缓存和 UI
      state.selectedLocalSkin.value = skinValue
      const url = parseSkinUrl(skinValue)
      if (url) state.skinUrl.value = url
      state.variant.value = parseSkinVariant(skinValue)
      bumpSkinVersion()
      toastSuccess('自定义皮肤已应用')
    } catch (e) {
      toastError(String(e))
    } finally {
      state.uploading.value = false
    }
  }

  /** 下载当前皮肤 PNG 到本地（弹出保存对话框） */
  async function saveSkinToLocal() {
    if (!state.skinUrl.value) {
      toastError('当前无皮肤数据')
      return
    }
    const defaultName = `${username.value || 'skin'}_${state.variant.value === 'slim' ? 'alex' : 'steve'}.png`
    const savePath = await pickSavePath({ title: '保存皮肤', defaultPath: defaultName, filters: [{ name: 'PNG 图片', extensions: ['png'] }] })
    if (!savePath) return
    try {
      await downloadUrlToFile(state.skinUrl.value, savePath)
      toastSuccess(`皮肤已保存到：${savePath}`)
    } catch (e) {
      toastError('保存失败：' + String(e))
    }
  }

  return {
    pickAndUpload,
    onEquipCape,
    onUnequipCape,
    onSelectLocalSkin,
    onUploadCustomSkin,
    saveSkinToLocal,
    // 外置账号专用
    onDeleteAuthlibSkin,
    onUploadAuthlibCape,
    onDeleteAuthlibCape,
  }
}
