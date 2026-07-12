<script setup lang="ts">
/**
 * 3D 皮肤模型组件（基于 skinview3d）
 *
 * skinview3d 是专为 Minecraft 皮肤设计的 3D 查看器，基于 three.js：
 * - 支持 1.8 皮肤 / HD 皮肤 / 披风 / 鞘翅 / 耳朵
 * - 自动检测 Slim（Alex）/ Default（Steve）模型
 * - 内置鼠标控制（旋转/缩放）和动画
 *
 * API: https://github.com/bs-community/skinview3d
 */

import { ref, watch, onMounted, onUnmounted, shallowRef } from 'vue'
import { SkinViewer, WalkingAnimation, IdleAnimation } from 'skinview3d'

const props = withDefaults(defineProps<{
  /** 皮肤 PNG dataURL（data:image/png;base64,...） */
  skinUrl: string | null
  /** 披风 PNG dataURL，null 表示无披风 */
  capeUrl?: string | null
  /** 皮肤模型：'classic' Steve | 'slim' Alex（skinview3d 也会自动检测） */
  variant?: 'classic' | 'slim'
  /** 画布高度（px） */
  height?: number
  /** 动画类型：'idle' 站立 | 'walk' 行走 | 'none' 无 */
  animation?: 'idle' | 'walk' | 'none'
}>(), {
  skinUrl: null,
  capeUrl: null,
  variant: 'classic',
  height: 320,
  animation: 'idle',
})

const container = ref<HTMLDivElement | null>(null)
// 用 shallowRef 避免 skinview3d 对象被 Vue 深度代理
const viewer = shallowRef<SkinViewer | null>(null)

/** 初始化 skinview3d 查看器 */
function initViewer() {
  const el = container.value
  if (!el) return

  // 清理旧实例
  destroyViewer()

  // 创建 canvas 元素
  const canvas = document.createElement('canvas')
  canvas.style.display = 'block'
  canvas.style.width = '100%'
  canvas.style.height = '100%'
  canvas.style.imageRendering = 'pixelated'
  el.appendChild(canvas)

  // 创建 SkinViewer 实例
  const skinViewer = new SkinViewer({
    canvas,
    width: el.clientWidth,
    height: props.height,
    skin: props.skinUrl || undefined,
  })

  // 加载披风
  if (props.capeUrl) {
    skinViewer.loadCape(props.capeUrl)
  } else {
    skinViewer.loadCape(null)
  }

  // 设置模型（如果指定了 slim，覆盖自动检测）
  if (props.variant === 'slim') {
    skinViewer.loadSkin(props.skinUrl!, { model: 'slim' })
  }

  // 自动旋转
  skinViewer.autoRotate = true
  skinViewer.autoRotateSpeed = 0.5

  // 动画
  applyAnimation(skinViewer)

  // 灯光调整（让模型更亮一些，减少阴影）
  skinViewer.globalLight.intensity = 3.0
  skinViewer.cameraLight.intensity = 0.6

  // 缩放（让模型大小合适）
  skinViewer.zoom = 0.85

  viewer.value = skinViewer
}

/** 应用动画 */
function applyAnimation(skinViewer: SkinViewer) {
  switch (props.animation) {
    case 'walk':
      skinViewer.animation = new WalkingAnimation()
      break
    case 'idle':
      skinViewer.animation = new IdleAnimation()
      break
    case 'none':
      skinViewer.animation = null
      break
  }
}

/** 销毁查看器 */
function destroyViewer() {
  if (!viewer.value) return
  viewer.value.dispose()
  const canvas = viewer.value.canvas
  if (canvas.parentNode) {
    canvas.parentNode.removeChild(canvas)
  }
  viewer.value = null
}

/** 监听 props 变化 */
watch(() => props.skinUrl, (newUrl) => {
  if (!viewer.value || !newUrl) return
  viewer.value.loadSkin(newUrl, props.variant === 'slim' ? { model: 'slim' } : {})
})

watch(() => props.capeUrl, (newUrl) => {
  if (!viewer.value) return
  viewer.value.loadCape(newUrl || null)
})

watch(() => props.variant, (newVariant) => {
  if (!viewer.value || !props.skinUrl) return
  viewer.value.loadSkin(props.skinUrl, newVariant === 'slim' ? { model: 'slim' } : {})
})

watch(() => props.animation, () => {
  if (viewer.value) applyAnimation(viewer.value)
})

watch(() => props.height, (newHeight) => {
  if (viewer.value) viewer.value.height = newHeight
})

onMounted(() => {
  if (props.skinUrl) initViewer()
})

onUnmounted(() => {
  destroyViewer()
})
</script>

<template>
  <div
    ref="container"
    class="skin-3d-container relative w-full"
    :style="{ height: height + 'px' }"
  >
    <div v-if="!skinUrl" class="flex h-full items-center justify-center text-xs text-gray-300">
      无皮肤数据
    </div>
  </div>
</template>

<style scoped>
.skin-3d-container :deep(canvas) {
  display: block;
  image-rendering: pixelated;
}
</style>
