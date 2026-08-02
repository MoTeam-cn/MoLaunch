/**
 * 皮肤/披风操作状态切片（从 useSkinOperations.ts 抽取）
 *
 * 负责全部状态声明与派生计算：
 * - 登录类型派生布尔值（isMicrosoft / isAuthlib / isOffline）
 * - 皮肤/披风信息、URL、变体、本地皮肤等响应式状态
 * - 当前装备披风/皮肤、上传权限等 computed
 * - image-cached 事件监听（后端下载完成后自动刷新远程 URL 为本地缓存 URL）
 *
 * 无业务动作；上传/披风/选择/删除等交互由 useSkinActions.ts 负责。
 */

import { ref, computed, type ComputedRef, type Ref } from 'vue'
import type { SkinCapeInfo, SkinInfo, CapeInfo } from '@/utils/tauri'
import type { AuthlibSkinCapeInfo } from '@/types/auth'
import { onImageCached } from '@/composables/useImageCache'

export interface UseSkinStateOptions {
  /** 登录类型（来自 authStore.currentUser.login_type） */
  loginType: ComputedRef<string>
}

export interface UseSkinState {
  isMicrosoft: ComputedRef<boolean>
  isAuthlib: ComputedRef<boolean>
  isOffline: ComputedRef<boolean>
  info: Ref<SkinCapeInfo | null>
  authlibInfo: Ref<AuthlibSkinCapeInfo | null>
  loading: Ref<boolean>
  uploading: Ref<boolean>
  skinUrl: Ref<string | null>
  capeUrl: Ref<string | null>
  variant: Ref<'classic' | 'slim'>
  selectedLocalSkin: Ref<string | null>
  authlibUsingDefaultSkin: Ref<boolean>
  activeCape: ComputedRef<CapeInfo | null>
  activeSkin: ComputedRef<SkinInfo | null>
  canUploadSkin: ComputedRef<boolean>
  canUploadCape: ComputedRef<boolean>
}

/**
 * 创建皮肤/披风操作状态切片
 *
 * 副作用：构造时立即启动 `image-cached` 事件监听（onImageCached 内部基于 useTauriEvent，onUnmounted 自动清理）。
 */
export function useSkinState(options: UseSkinStateOptions): UseSkinState {
  const { loginType } = options

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

  return {
    isMicrosoft,
    isAuthlib,
    isOffline,
    info,
    authlibInfo,
    loading,
    uploading,
    skinUrl,
    capeUrl,
    variant,
    selectedLocalSkin,
    authlibUsingDefaultSkin,
    activeCape,
    activeSkin,
    canUploadSkin,
    canUploadCape,
  }
}
