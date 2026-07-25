/**
 * 皮肤/披风操作 composable（从 SkinManager.vue 抽出）
 *
 * 封装 SkinManager 弹窗的全部业务逻辑（三分流：微软 / 外置 / 离线）：
 * - 加载皮肤/披风信息（loadInfo）：
 *   - 微软账号：从后端拉取 profile_json（含 skins / capes 列表）
 *   - 外置账号（yggdrasil）：调用 authlibGetSkinInfo，按 uploadableTextures 决定可上传项
 *   - 离线账号：使用本地默认皮肤或自定义皮肤
 * - 上传皮肤：
 *   - 微软账号：uploadSkin（Mojang API）
 *   - 外置账号：authlibUploadSkin（yggdrasil PUT /api/user/profile/{uuid}/skin）
 *   - 离线账号：saveCustomSkin（保存到本地 app data）
 * - 装备/取消披风（仅微软）：onEquipCape / onUnequipCape
 * - 上传/删除披风（仅外置）：onUploadAuthlibCape / onDeleteAuthlibCape
 * - 通用刷新流程（runWithRefresh）：执行 → 提示 → 重新加载 + 触发头像刷新
 * - 离线账号本地皮肤选择（onSelectLocalSkin）
 * - 下载当前皮肤到本地（saveSkinToLocal）
 * - image-cache 事件监听（onImageCached）：远程图片下载完成后自动替换为本地缓存 URL
 *
 * 设计原则：
 * - 接收所需的 computed 作为参数（uuid / username / loginType 等，来自 authStore）
 * - 返回状态 ref/computed 和 handler 函数
 * - 仅引入 toast，未引入 modal；遵循 toast.ts 推荐用法，使用 `toastSuccess`/`toastError` 前缀
 *
 * 使用方式：
 * ```ts
 * const uuid = computed(() => authStore.currentUser?.uuid ?? '')
 * const username = computed(() => authStore.currentUser?.name ?? '')
 * const loginType = computed(() => authStore.currentUser?.login_type ?? '')
 * const serverUrl = computed(() => authStore.currentUser?.server_url ?? '')
 * const {
 *   info, loading, uploading, skinUrl, capeUrl, variant, selectedLocalSkin,
 *   activeCape, activeSkin, authlibInfo, canUploadSkin, canUploadCape,
 *   loadInfo, pickAndUpload, onEquipCape, onUnequipCape,
 *   onSelectLocalSkin, onUploadCustomSkin, saveSkinToLocal,
 *   onUploadAuthlibCape, onDeleteAuthlibCape, onDeleteAuthlibSkin,
 *   isMicrosoft, isAuthlib, isOffline,
 * } = useSkinOperations({ uuid, username, loginType, serverUrl })
 * ```
 */
import { ref, computed, type ComputedRef } from 'vue'
import {
  getSkinCapeInfo, getSkinUrl, getCapeUrl, uploadSkin, equipCape, unequipCape,
  downloadUrlToFile, type SkinCapeInfo,
} from '@/utils/tauri'
import { pickFile, pickSavePath } from '@/utils/fileDialog'
import { onImageCached } from '@/composables/useImageCache'
import { toastSuccess, toastError } from '@/utils/toast'
import { saveCustomSkin } from '@/utils/api/auth'
import {
  authlibDeleteCape, authlibDeleteSkin, authlibGetSkinInfo, authlibUploadCape, authlibUploadSkin,
} from '@/utils/api/authlib'
import {
  STEVE_SKIN_URL, getDefaultSkinEntry, getLocalSkinName, setLocalSkinName, bumpSkinVersion,
  parseSkinUrl, parseSkinVariant,
} from '@/utils/default-skin'
import { safeCall } from '@/utils/async'
import type { AuthlibSkinCapeInfo } from '@/types/auth'

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

  // 派生登录类型布尔值（避免调用方反复写 === 'Microsoft'）
  const isMicrosoft = computed(() => loginType.value === 'Microsoft')
  const isAuthlib = computed(() => loginType.value === 'AuthlibInjector')
  const isOffline = computed(() => loginType.value === 'Legacy')

  const info = ref<SkinCapeInfo | null>(null)
  /** 外置账号的皮肤披风信息（含 uploadableTextures，仅 isAuthlib 时有值） */
  const authlibInfo = ref<AuthlibSkinCapeInfo | null>(null)
  const loading = ref(false)
  const uploading = ref(false)
  const skinUrl = ref<string | null>(null)
  const capeUrl = ref<string | null>(null)
  const variant = ref<'classic' | 'slim'>('classic')
  /** 离线账号当前选中的本地皮肤名称 */
  const selectedLocalSkin = ref<string | null>(null)
  /**
   * 外置账号是否正在用默认皮肤顶替（服务器未设置皮肤时为 true）
   *
   * 用于 SkinManager 弹窗显示 info 提示，并阻止"删除皮肤"按钮误操作。
   * 仅 isAuthlib 分支可能为 true；每次 loadInfo 重新计算。
   */
  const authlibUsingDefaultSkin = ref(false)

  /** 当前已装备的披风（微软） */
  const activeCape = computed(() => info.value?.capes.find(c => c.state === 'ACTIVE') ?? null)
  /** 当前已装备的皮肤（微软） */
  const activeSkin = computed(() => info.value?.skins.find(s => s.state === 'ACTIVE') ?? info.value?.skins[0] ?? null)

  /** 外置账号是否允许上传皮肤（uploadableTextures 包含 "skin"） */
  const canUploadSkin = computed(() => {
    if (!isAuthlib.value || !authlibInfo.value) return false
    return authlibInfo.value.uploadable_textures.includes('skin')
  })

  /** 外置账号是否允许上传披风（uploadableTextures 包含 "cape"） */
  const canUploadCape = computed(() => {
    if (!isAuthlib.value || !authlibInfo.value) return false
    return authlibInfo.value.uploadable_textures.includes('cape')
  })

  /** 监听 image-cached 事件，当后端下载完成后自动刷新远程 URL 为本地缓存 URL */
  onImageCached((remoteUrl, localUrl) => {
    if (skinUrl.value === remoteUrl) {
      skinUrl.value = localUrl
    }
    if (capeUrl.value === remoteUrl) {
      capeUrl.value = localUrl
    }
  })

  async function loadInfo() {
    const dev = import.meta.env.DEV
    dev && console.log('[SkinManager] loadInfo started, loginType:', loginType.value)
    loading.value = true
    skinUrl.value = null
    capeUrl.value = null
    authlibInfo.value = null
    authlibUsingDefaultSkin.value = false

    // 外置账号（yggdrasil）：从服务器拉取角色属性，解析 textures
    if (isAuthlib.value) {
      if (!serverUrl.value || !uuid.value) {
        toastError('外置账号缺少 server_url 或 uuid，无法加载皮肤信息')
        loading.value = false
        return
      }
      try {
        const data = await authlibGetSkinInfo(serverUrl.value, uuid.value)
        authlibInfo.value = data
        // 服务器未设置皮肤时（textures 为空 {}），skin_url 为 null
        // 此时用 Steve 顶上（与 yggdrasil 协议"未设置皮肤按 Steve 处理"一致），
        // 避免 3D 预览空白，并标记以显示提示
        if (data.skin_url) {
          skinUrl.value = data.skin_url
        } else {
          skinUrl.value = STEVE_SKIN_URL
          authlibUsingDefaultSkin.value = true
        }
        // 披风保持 null，不顶默认披风（披风不是必需品）
        capeUrl.value = data.cape_url
        variant.value = data.skin_model === 'slim' ? 'slim' : 'classic'
        dev && console.log('[SkinManager] authlib skin info loaded:', data, 'usingDefault:', authlibUsingDefaultSkin.value)
      } catch (e) {
        console.error('[SkinManager] authlibGetSkinInfo failed:', e)
        toastError(`获取皮肤信息失败: ${e}`)
      }
      info.value = null
      loading.value = false
      return
    }

    if (!isMicrosoft.value) {
      // 离线账号：使用本地默认皮肤或自定义皮肤（从注册表同步的内存缓存）
      const stored = getLocalSkinName(uuid.value)
      selectedLocalSkin.value = stored
      if (stored) {
        // 解析 skin 字段获取 URL 和变体（支持默认皮肤和自定义皮肤）
        const url = parseSkinUrl(stored)
        if (url) skinUrl.value = url
        variant.value = parseSkinVariant(stored)
      } else {
        // 未选择皮肤时使用 UUID 哈希默认
        const entry = getDefaultSkinEntry(uuid.value || username.value)
        skinUrl.value = entry.url
        variant.value = entry.variant
        selectedLocalSkin.value = entry.name
      }
      info.value = null
      loading.value = false
      dev && console.log('[SkinManager] offline account, using skin:', selectedLocalSkin.value)
      return
    }

    // 微软账号：从后端获取最新皮肤/披风信息（后端操作成功后会自动刷新 profile_json）
    try {
      info.value = await getSkinCapeInfo()
      dev && console.log('[SkinManager] getSkinCapeInfo ok:', info.value)
    } catch (e) {
      console.error('[SkinManager] getSkinCapeInfo failed:', e)
      toastError(`获取皮肤信息失败: ${e}`)
    }
    const skinResult = await safeCall(() => getSkinUrl(), '[SkinManager] getSkinUrl')
    skinUrl.value = skinResult?.url ?? null
    dev && console.log('[SkinManager] getSkinUrl ok:', skinResult?.cached ? 'cached' : 'remote')
    try {
      const result = await getCapeUrl()
      capeUrl.value = result?.url ?? null
      dev && console.log('[SkinManager] getCapeUrl ok:', result ? (result.cached ? 'cached' : 'remote') : 'no cape')
    } catch (e) {
      console.warn('[SkinManager] getCapeUrl failed:', e)
      capeUrl.value = null
    }
    variant.value = activeSkin.value?.variant === 'slim' ? 'slim' : 'classic'

    loading.value = false
    dev && console.log('[SkinManager] loadInfo done, skinUrl:', skinUrl.value ? 'has url' : 'null')
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
    if (isOffline.value) {
      await onUploadCustomSkin()
      return
    }
    if (isAuthlib.value) {
      if (!canUploadSkin.value) {
        toastError('此服务器不允许上传皮肤')
        return
      }
      try {
        const filePath = await pickFile({ title: '选择皮肤 PNG 文件', filters: [{ name: 'PNG 图片', extensions: ['png'] }] })
        if (!filePath) return
        await runWithRefresh('皮肤上传成功', async () => {
          if (!serverUrl.value) throw new Error('外置账号缺少 server_url')
          const model: 'slim' | 'default' = variant.value === 'slim' ? 'slim' : 'default'
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
      await runWithRefresh('皮肤上传成功', () => uploadSkin(filePath, variant.value))
    } catch (e) {
      toastError(String(e))
    }
  }

  /** 外置账号：删除皮肤（恢复默认） */
  async function onDeleteAuthlibSkin() {
    if (!isAuthlib.value || !serverUrl.value) return
    await runWithRefresh('皮肤已删除', async () => {
      await authlibDeleteSkin(serverUrl.value, uuid.value)
    })
  }

  /** 外置账号：上传披风 */
  async function onUploadAuthlibCape() {
    if (!isAuthlib.value || !canUploadCape.value || !serverUrl.value) return
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
    if (!isAuthlib.value || !serverUrl.value) return
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

  /** 上传/装备/取消操作后的通用流程：执行 → 提示 → 重新加载 + 触发头像刷新 */
  async function runWithRefresh(successMsg: string, fn: () => Promise<unknown>) {
    uploading.value = true
    try {
      await fn()
      toastSuccess(successMsg)
      await loadInfo()
      bumpSkinVersion()
    } catch (e) {
      toastError(String(e))
    } finally {
      uploading.value = false
    }
  }

  /** 离线账号：选择本地默认皮肤 */
  async function onSelectLocalSkin(skinName: string) {
    await setLocalSkinName(uuid.value, skinName)
    selectedLocalSkin.value = skinName
    const url = parseSkinUrl(skinName)
    if (url) skinUrl.value = url
    variant.value = parseSkinVariant(skinName)
    bumpSkinVersion()
    toastSuccess(`已切换为 ${skinName} 皮肤`)
  }

  /** 离线账号：上传自定义皮肤 PNG 文件 */
  async function onUploadCustomSkin() {
    try {
      const filePath = await pickFile({ title: '选择皮肤 PNG 文件', filters: [{ name: 'PNG 图片', extensions: ['png'] }] })
      if (!filePath) return

      uploading.value = true
      // 保存到 app data 并获取 skin 字段值
      const skinValue = await saveCustomSkin(uuid.value, filePath, variant.value)

      // 更新内存缓存和 UI
      selectedLocalSkin.value = skinValue
      const url = parseSkinUrl(skinValue)
      if (url) skinUrl.value = url
      variant.value = parseSkinVariant(skinValue)
      bumpSkinVersion()
      toastSuccess('自定义皮肤已应用')
    } catch (e) {
      toastError(String(e))
    } finally {
      uploading.value = false
    }
  }

  /** 下载当前皮肤 PNG 到本地（弹出保存对话框） */
  async function saveSkinToLocal() {
    if (!skinUrl.value) {
      toastError('当前无皮肤数据')
      return
    }
    const defaultName = `${username.value || 'skin'}_${variant.value === 'slim' ? 'alex' : 'steve'}.png`
    const savePath = await pickSavePath({ title: '保存皮肤', defaultPath: defaultName, filters: [{ name: 'PNG 图片', extensions: ['png'] }] })
    if (!savePath) return
    try {
      await downloadUrlToFile(skinUrl.value, savePath)
      toastSuccess(`皮肤已保存到：${savePath}`)
    } catch (e) {
      toastError('保存失败：' + String(e))
    }
  }

  return {
    // 派生状态
    isMicrosoft,
    isAuthlib,
    isOffline,
    // 状态
    info,
    authlibInfo,
    loading,
    uploading,
    skinUrl,
    capeUrl,
    variant,
    selectedLocalSkin,
    authlibUsingDefaultSkin,
    // computed
    activeCape,
    activeSkin,
    canUploadSkin,
    canUploadCape,
    // handler
    loadInfo,
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
