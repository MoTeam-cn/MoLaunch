/**
 * 皮肤/披风操作 composable（从 SkinManager.vue 抽出）
 *
 * 封装 SkinManager 弹窗的全部业务逻辑：
 * - 加载皮肤/披风信息（loadInfo）：微软账号从后端拉取，离线账号使用本地默认皮肤
 * - 上传皮肤（pickAndUpload）
 * - 装备/取消披风（onEquipCape / onUnequipCape）
 * - 通用刷新流程（runWithRefresh）：执行 → 提示 → 重新加载 + 触发头像刷新
 * - 离线账号本地皮肤选择（onSelectLocalSkin）
 * - 下载当前皮肤到本地（saveSkinToLocal）
 * - image-cache 事件监听（onImageCached）：远程图片下载完成后自动替换为本地缓存 URL
 *
 * 设计原则：
 * - 接收所需的 computed 作为参数（uuid / username / isMicrosoft，来自 authStore）
 * - 返回状态 ref/computed 和 handler 函数
 * - 仅引入 toast，未引入 modal；遵循 toast.ts 推荐用法，使用 `toastSuccess`/`toastError` 前缀
 *
 * 使用方式：
 * ```ts
 * const uuid = computed(() => authStore.currentUser?.uuid ?? '')
 * const username = computed(() => authStore.currentUser?.name ?? '')
 * const isMicrosoft = computed(() => authStore.currentUser?.login_type === 'Microsoft')
 * const {
 *   info, loading, uploading, skinUrl, capeUrl, variant, selectedLocalSkin,
 *   activeCape, activeSkin,
 *   loadInfo, pickAndUpload, onEquipCape, onUnequipCape,
 *   onSelectLocalSkin, saveSkinToLocal,
 * } = useSkinOperations({ uuid, username, isMicrosoft })
 * ```
 */
import { ref, computed, type ComputedRef } from 'vue'
import {
  getSkinCapeInfo, getSkinUrl, getCapeUrl, uploadSkin, equipCape, unequipCape,
  selectFile, saveFile, downloadUrlToFile, type SkinCapeInfo,
} from '@/utils/tauri'
import { onImageCached } from '@/composables/useImageCache'
import { toastSuccess, toastError } from '@/utils/toast'
import { saveCustomSkin } from '@/utils/api/auth'
import {
  defaultSkins, getDefaultSkinEntry, getLocalSkinName, setLocalSkinName, bumpSkinVersion,
  parseSkinUrl, parseSkinVariant,
} from '@/utils/default-skin'

interface UseSkinOperationsOptions {
  /** 当前账号 UUID（来自 authStore.currentUser.uuid） */
  uuid: ComputedRef<string>
  /** 当前账号用户名（来自 authStore.currentUser.name） */
  username: ComputedRef<string>
  /** 是否微软账号（来自 authStore.currentUser.login_type） */
  isMicrosoft: ComputedRef<boolean>
}

/**
 * 皮肤/披风操作 composable
 *
 * 副作用：构造时立即启动 `image-cached` 事件监听（onImageCached 内部基于 useTauriEvent，onUnmounted 自动清理）。
 */
export function useSkinOperations(options: UseSkinOperationsOptions) {
  const { uuid, username, isMicrosoft } = options

  const info = ref<SkinCapeInfo | null>(null)
  const loading = ref(false)
  const uploading = ref(false)
  const skinUrl = ref<string | null>(null)
  const capeUrl = ref<string | null>(null)
  const variant = ref<'classic' | 'slim'>('classic')
  /** 离线账号当前选中的本地皮肤名称 */
  const selectedLocalSkin = ref<string | null>(null)

  /** 当前已装备的披风 */
  const activeCape = computed(() => info.value?.capes.find(c => c.state === 'ACTIVE') ?? null)
  /** 当前已装备的皮肤 */
  const activeSkin = computed(() => info.value?.skins.find(s => s.state === 'ACTIVE') ?? info.value?.skins[0] ?? null)

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
    dev && console.log('[SkinManager] loadInfo started, isMicrosoft:', isMicrosoft.value)
    loading.value = true
    skinUrl.value = null
    capeUrl.value = null

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
    try {
      const result = await getSkinUrl()
      skinUrl.value = result?.url ?? null
      dev && console.log('[SkinManager] getSkinUrl ok:', result?.cached ? 'cached' : 'remote')
    } catch (e) {
      console.error('[SkinManager] getSkinUrl failed:', e)
    }
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

  async function pickAndUpload() {
    try {
      const filePath = await selectFile('选择皮肤 PNG 文件', [{ name: 'PNG 图片', extensions: ['png'] }])
      if (!filePath) return
      await runWithRefresh('皮肤上传成功', () => uploadSkin(filePath, variant.value))
    } catch (e) {
      toastError(String(e))
    }
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
      const filePath = await selectFile('选择皮肤 PNG 文件', [{ name: 'PNG 图片', extensions: ['png'] }])
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
    const savePath = await saveFile('保存皮肤', defaultName, [{ name: 'PNG 图片', extensions: ['png'] }])
    if (!savePath) return
    try {
      await downloadUrlToFile(skinUrl.value, savePath)
      toastSuccess(`皮肤已保存到：${savePath}`)
    } catch (e) {
      toastError('保存失败：' + String(e))
    }
  }

  return {
    // 状态
    info,
    loading,
    uploading,
    skinUrl,
    capeUrl,
    variant,
    selectedLocalSkin,
    // computed
    activeCape,
    activeSkin,
    // handler
    loadInfo,
    pickAndUpload,
    onEquipCape,
    onUnequipCape,
    onSelectLocalSkin,
    onUploadCustomSkin,
    saveSkinToLocal,
  }
}
