<script setup lang="ts">
/**
 * 玩家皮肤头像组件
 *
 * 实现规格：
 * - 控件容器：64x64
 * - 脸层（ImgBack）：48x48，居中显示，从皮肤 (Scale*8, Scale*8) 裁剪 8x8
 * - 头发层（ImgFore）：56x56，居中显示（比脸层大 1/6，向外扩展形成立体感），
 *   从皮肤 (Scale*40, Scale*8) 裁剪 8x8
 * - 头发层仅在图片存在透明像素（或颜色差异）时才叠加，避免纯色白底覆盖
 *
 * 本组件用两层 div + img 模拟双 Image 重叠效果：
 * - 外层容器：size x size
 * - 脸层：size * (48/64) 居中
 * - 头发层：size * (56/64) 居中，仅当有头发内容时显示
 */

import { ref, watch, computed, onMounted } from 'vue'
import { getSkinUrl } from '@/utils/tauri'
import { authlibGetSkinInfo } from '@/utils/api/authlib'
import { onImageCached } from '@/composables/useImageCache'
import { getDefaultSkin, STEVE_SKIN_URL, skinVersion } from '@/utils/default-skin'
import { loadImage, clipImageRegion } from '@/utils/image-crop'

const props = withDefaults(defineProps<{
  /** 玩家 UUID（用于触发重新加载） */
  uuid?: string
  /** 头像尺寸（px，容器尺寸） */
  size?: number
  /** 是否叠加头发层（ImgFore） */
  overlay?: boolean
  /** 用户名（用于回退显示） */
  username?: string
  /** 是否禁用圆角（用于列表小头像） */
  rounded?: boolean
  /** 登录类型：'Microsoft' | 'AuthlibInjector' | 'Offline'，决定皮肤加载策略 */
  loginType?: string
  /** 直接传入皮肤 PNG URL（优先级最高，用于皮肤选择网格等场景） */
  skinUrl?: string
  /** yggdrasil API 根地址（仅 AuthlibInjector 账号需要，用于从服务器拉取皮肤） */
  serverUrl?: string
}>(), {
  uuid: '',
  size: 64,
  overlay: true,
  username: '',
  rounded: true,
  loginType: '',
  skinUrl: '',
  serverUrl: '',
})

/** 脸层图片（8x8 区域） */
const faceDataUrl = ref<string | null>(null)
/** 头发层图片（8x8 区域，可能为 null 表示无头发） */
const hairDataUrl = ref<string | null>(null)
const loadFailed = ref(false)
const loading = ref(false)
/** 当前加载的远程 URL（用于 image-cached 事件匹配后重新加载） */
const currentRemoteUrl = ref<string | null>(null)

const avatarLetter = computed(() => {
  const name = props.username || ''
  return name.charAt(0).toUpperCase() || '?'
})

const avatarGradient = computed(() => {
  const gradients = [
    'from-blue-400 to-blue-600',
    'from-green-400 to-green-600',
    'from-purple-400 to-purple-600',
    'from-orange-400 to-orange-600',
    'from-pink-400 to-pink-600',
    'from-teal-400 to-teal-600',
  ]
  let hash = 0
  for (const c of props.username || '') hash = (hash * 31 + c.charCodeAt(0)) | 0
  return gradients[Math.abs(hash) % gradients.length]
})

/** 脸层显示尺寸（48/64 of 容器） */
const faceSize = computed(() => Math.round(props.size * 48 / 64))
/** 头发层显示尺寸（56/64 of 容器） */
const hairSize = computed(() => Math.round(props.size * 56 / 64))

/**
 * 从皮肤 PNG 裁剪头像
 *
 * 裁剪逻辑：
 * - Scale = 图片宽度 / 64（支持高清皮肤，如 128x64）
 * - 脸层：Clip(Scale*8, Scale*8, Scale*8, Scale*8)
 * - 头发层（附加层）：Clip(Scale*40, Scale*8, Scale*8, Scale*8)
 *   仅当图片有透明像素时才叠加（避免纯色白底覆盖）
 *
 * 透明像素检查：
 *   - 检查 (1,1)、(W-1,H-1)、(W-2,H/2-2) 三点是否有透明像素
 *   - 或检查这三点与 (Scale*41, Scale*9) 颜色是否不同
 *   - 满足任一条件才叠加头发层
 */
async function loadAvatar() {
  // 无数据源时直接回退
  if (!props.skinUrl && !props.uuid) {
    faceDataUrl.value = null
    hairDataUrl.value = null
    loadFailed.value = true
    currentRemoteUrl.value = null
    return
  }
  loading.value = true
  loadFailed.value = false
  try {
    // 优先级：直接传入的 skinUrl > AuthlibInjector（yggdrasil API）> Offline（本地默认）> Microsoft（后端获取）
    let pngUrl: string
    if (props.skinUrl) {
      pngUrl = props.skinUrl
      currentRemoteUrl.value = null  // 直接传入的 URL 不参与缓存刷新
    } else if (props.loginType === 'AuthlibInjector') {
      // 外置账号：从 yggdrasil 服务器拉取角色属性，解析 textures 取皮肤 URL
      // 失败或无皮肤时回退到 Steve（与 yggdrasil 协议"未设置皮肤按 Steve 处理"一致）
      if (props.serverUrl && props.uuid) {
        try {
          const info = await authlibGetSkinInfo(props.serverUrl, props.uuid)
          pngUrl = info.skin_url ?? STEVE_SKIN_URL
        } catch {
          pngUrl = STEVE_SKIN_URL
        }
      } else {
        pngUrl = STEVE_SKIN_URL
      }
      currentRemoteUrl.value = null
    } else if (props.loginType === 'Offline') {
      // 离线账号：使用用户选择的皮肤（从注册表同步到内存）或 uuid hash 默认
      pngUrl = getDefaultSkin(props.uuid || props.username)
      currentRemoteUrl.value = null
    } else {
      // 微软账号：获取皮肤 URL（CachedImage），失败时静默回退到默认皮肤
      try {
        const result = await getSkinUrl(props.uuid)
        if (result) {
          pngUrl = result.url
          // 仅当返回的是远程 URL（未命中缓存）时才记录，用于事件匹配
          currentRemoteUrl.value = result.cached ? null : result.url
        } else {
          pngUrl = getDefaultSkin(props.uuid || props.username)
          currentRemoteUrl.value = null
        }
      } catch {
        pngUrl = getDefaultSkin(props.uuid || props.username)
        currentRemoteUrl.value = null
      }
    }

    // 加载为 Image 对象
    const img = await loadImage(pngUrl)

    // 计算缩放比例（Scale）
    const w = img.naturalWidth
    const h = img.naturalHeight
    if (w < 32 || h < 32) {
      throw new Error(`skin image too small: ${w}x${h}`)
    }
    const scale = w / 64
    const cellSize = Math.max(1, Math.round(scale * 8))

    // 裁剪脸层：从 (Scale*8, Scale*8) 取 8x8 区域
    faceDataUrl.value = clipImageRegion(img, scale * 8, scale * 8, cellSize, cellSize, undefined, undefined, true)

    // 裁剪头发层（附加层）：从 (Scale*40, Scale*8) 取 8x8 区域
    // 仅当图片有透明像素时才叠加
    hairDataUrl.value = null
    if (props.overlay && w >= 64 && h >= 32) {
      // 透明检查：(1,1)、(W-1,H-1)、(W-2,H/2-2) 是否有透明像素
      // 或这三点与 (Scale*41, Scale*9) 颜色是否不同
      const tmp = document.createElement('canvas')
      tmp.width = w
      tmp.height = h
      const tmpCtx = tmp.getContext('2d', { willReadFrequently: true })
      if (tmpCtx) {
        tmpCtx.drawImage(img, 0, 0)
        const checkPoints = [
          [1, 1],
          [w - 1, h - 1],
          [w - 2, Math.floor(h / 2) - 2],
        ]
        const hairSample = tmpCtx.getImageData(Math.round(scale * 41), Math.round(scale * 9), 1, 1).data
        let hasTransparency = false
        let colorDifferent = false
        for (const [px, py] of checkPoints) {
          const p = tmpCtx.getImageData(px, py, 1, 1).data
          if (p[3] === 0) { hasTransparency = true; break }
          // 颜色差异检查（RGB）
          if (Math.abs(p[0] - hairSample[0]) > 5 ||
              Math.abs(p[1] - hairSample[1]) > 5 ||
              Math.abs(p[2] - hairSample[2]) > 5) {
            colorDifferent = true
          }
        }
        // 还需检查头发层区域本身是否有非透明内容
        const hairDataUrlTmp = clipImageRegion(img, scale * 40, scale * 8, cellSize, cellSize, undefined, undefined, true)
        const hairCanvas = document.createElement('canvas')
        hairCanvas.width = cellSize
        hairCanvas.height = cellSize
        const hairCtx = hairCanvas.getContext('2d', { willReadFrequently: true })
        if (hairCtx) {
          const hairImg = await loadImage(hairDataUrlTmp)
          hairCtx.drawImage(hairImg, 0, 0)
          const hairData = hairCtx.getImageData(0, 0, cellSize, cellSize).data
          let hairHasContent = false
          for (let i = 3; i < hairData.length; i += 4) {
            if (hairData[i] > 0) { hairHasContent = true; break }
          }
          // 叠加条件：图片有透明像素 或 颜色差异，且头发层有内容
          if ((hasTransparency || colorDifferent) && hairHasContent) {
            hairDataUrl.value = hairDataUrlTmp
          }
        }
      }
    }
  } catch {
    loadFailed.value = true
  } finally {
    loading.value = false
  }
}

/** 监听 image-cached 事件，当后端下载完成后重新加载头像（从本地缓存读取） */
onImageCached((remoteUrl) => {
  if (currentRemoteUrl.value === remoteUrl) {
    currentRemoteUrl.value = null  // 清除标记避免重复加载
    loadAvatar()
  }
})

onMounted(loadAvatar)
watch(() => [props.uuid, props.loginType, props.skinUrl, props.serverUrl, skinVersion.value], loadAvatar)
</script>

<template>
  <div
    class="relative flex items-center justify-center overflow-hidden"
    :class="rounded ? 'rounded-full' : 'rounded-md'"
    :style="{ height: `${size}px`, width: `${size}px` }"
  >
    <!-- 皮肤裁剪头像：双层叠加 -->
    <template v-if="faceDataUrl && !loadFailed">
      <!-- 脸层（ImgBack）：48/64 居中 -->
      <img
        :src="faceDataUrl"
        :width="faceSize"
        :height="faceSize"
        alt="avatar"
        class="absolute"
        :style="{
          width: faceSize + 'px',
          height: faceSize + 'px',
          imageRendering: 'pixelated',
        }"
      />
      <!-- 头发层（ImgFore）：56/64 居中（比脸层大 1/6，形成立体感） -->
      <img
        v-if="hairDataUrl"
        :src="hairDataUrl"
        :width="hairSize"
        :height="hairSize"
        alt="hair"
        class="absolute pointer-events-none"
        :style="{
          width: hairSize + 'px',
          height: hairSize + 'px',
          imageRendering: 'pixelated',
        }"
      />
    </template>

    <!-- 加载中 -->
    <div
      v-else-if="loading"
      class="flex h-full w-full items-center justify-center bg-gray-100"
      :class="rounded ? 'rounded-full' : 'rounded-md'"
    >
      <svg class="h-1/2 w-1/2 animate-spin text-gray-300" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
        <path d="M12 2a10 10 0 0110 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
      </svg>
    </div>

    <!-- 回退：首字母渐变背景 -->
    <div
      v-else
      class="flex h-full w-full items-center justify-center bg-gradient-to-br font-bold text-white"
      :class="[avatarGradient, rounded ? 'rounded-full' : 'rounded-md']"
      :style="{ fontSize: `${size * 0.4}px` }"
    >
      {{ avatarLetter }}
    </div>
  </div>
</template>
