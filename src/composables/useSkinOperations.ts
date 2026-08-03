/**
 * 皮肤/披风操作 composable（按登录方式三分流：微软 / 外置 / 离线）
 *
 * 拆为两个切片：useSkinState（状态）/ useSkinActions（交互），本文件负责 loadInfo 与组装；
 * 仅用 toast，不引入 modal。
 */
import type { ComputedRef } from 'vue'
import { getSkinCapeInfo, getSkinUrl, getCapeUrl } from '@/utils/tauri'
import { toastError } from '@/utils/toast'
import { authlibGetSkinInfo } from '@/utils/api/authlib'
import {
  STEVE_SKIN_URL, getDefaultSkinEntry, getLocalSkinName, parseSkinUrl, parseSkinVariant,
} from '@/utils/default-skin'
import { safeCall } from '@/utils/async'
import { useSkinState } from './useSkinState'
import { useSkinActions } from './useSkinActions'

interface UseSkinOperationsOptions {
  /** 当前账号 UUID（来自 authStore.currentUser.uuid） */
  uuid: ComputedRef<string>
  /** 当前账号用户名（来自 authStore.currentUser.name） */
  username: ComputedRef<string>
  /** 登录类型：'Microsoft' / 'AuthlibInjector' / 'Legacy'（来自 authStore.currentUser.login_type） */
  loginType: ComputedRef<string>
  /** yggdrasil API 根地址（仅外置账号有效，其他账号为空字符串） */
  serverUrl: ComputedRef<string>
}

/**
 * 皮肤/披风操作 composable
 *
 * 副作用：构造时立即启动 `image-cached` 事件监听（onImageCached 内部基于 useTauriEvent，onUnmounted 自动清理）。
 */
export function useSkinOperations(options: UseSkinOperationsOptions) {
  const { uuid, username, loginType, serverUrl } = options

  const state = useSkinState({ loginType })

  async function loadInfo() {
    const dev = import.meta.env.DEV
    dev && console.log('[SkinManager] loadInfo started, loginType:', loginType.value)
    state.loading.value = true
    state.skinUrl.value = null
    state.capeUrl.value = null
    state.authlibInfo.value = null
    state.authlibUsingDefaultSkin.value = false

    // 外置账号（yggdrasil）：从服务器拉取角色属性，解析 textures
    if (state.isAuthlib.value) {
      if (!serverUrl.value || !uuid.value) {
        toastError('外置账号缺少 server_url 或 uuid，无法加载皮肤信息')
        state.loading.value = false
        return
      }
      try {
        const data = await authlibGetSkinInfo(serverUrl.value, uuid.value)
        state.authlibInfo.value = data
        // 服务器未设置皮肤时（textures 为空 {}），skin_url 为 null
        // 此时用 Steve 顶上（与 yggdrasil 协议"未设置皮肤按 Steve 处理"一致），
        // 避免 3D 预览空白，并标记以显示提示
        if (data.skin_url) {
          state.skinUrl.value = data.skin_url
        } else {
          state.skinUrl.value = STEVE_SKIN_URL
          state.authlibUsingDefaultSkin.value = true
        }
        // 披风保持 null，不顶默认披风（披风不是必需品）
        state.capeUrl.value = data.cape_url
        state.variant.value = data.skin_model === 'slim' ? 'slim' : 'classic'
        dev && console.log('[SkinManager] authlib skin info loaded:', data, 'usingDefault:', state.authlibUsingDefaultSkin.value)
      } catch (e) {
        console.error('[SkinManager] authlibGetSkinInfo failed:', e)
        toastError(`获取皮肤信息失败: ${e}`)
      }
      state.info.value = null
      state.loading.value = false
      return
    }

    if (!state.isMicrosoft.value) {
      // 离线账号：使用本地默认皮肤或自定义皮肤（从注册表同步的内存缓存）
      const stored = getLocalSkinName(uuid.value)
      state.selectedLocalSkin.value = stored
      if (stored) {
        // 解析 skin 字段获取 URL 和变体（支持默认皮肤和自定义皮肤）
        const url = parseSkinUrl(stored)
        if (url) state.skinUrl.value = url
        state.variant.value = parseSkinVariant(stored)
      } else {
        // 未选择皮肤时使用 UUID 哈希默认
        const entry = getDefaultSkinEntry(uuid.value || username.value)
        state.skinUrl.value = entry.url
        state.variant.value = entry.variant
        state.selectedLocalSkin.value = entry.name
      }
      state.info.value = null
      state.loading.value = false
      dev && console.log('[SkinManager] offline account, using skin:', state.selectedLocalSkin.value)
      return
    }

    // 微软账号：从后端获取最新皮肤/披风信息（后端操作成功后会自动刷新 profile_json）
    try {
      state.info.value = await getSkinCapeInfo()
      dev && console.log('[SkinManager] getSkinCapeInfo ok:', state.info.value)
    } catch (e) {
      console.error('[SkinManager] getSkinCapeInfo failed:', e)
      toastError(`获取皮肤信息失败: ${e}`)
    }
    const skinResult = await safeCall(() => getSkinUrl(), '[SkinManager] getSkinUrl')
    state.skinUrl.value = skinResult?.url ?? null
    dev && console.log('[SkinManager] getSkinUrl ok:', skinResult?.cached ? 'cached' : 'remote')
    try {
      const result = await getCapeUrl()
      state.capeUrl.value = result?.url ?? null
      dev && console.log('[SkinManager] getCapeUrl ok:', result ? (result.cached ? 'cached' : 'remote') : 'no cape')
    } catch (e) {
      console.warn('[SkinManager] getCapeUrl failed:', e)
      state.capeUrl.value = null
    }
    state.variant.value = state.activeSkin.value?.variant === 'slim' ? 'slim' : 'classic'

    state.loading.value = false
    dev && console.log('[SkinManager] loadInfo done, skinUrl:', state.skinUrl.value ? 'has url' : 'null')
  }

  const actions = useSkinActions({
    uuid,
    username,
    serverUrl,
    state,
    loadInfo,
  })

  return {
    ...state,
    loadInfo,
    ...actions,
  }
}
