<script setup lang="ts">
/**
 * 通用缓存图片组件
 *
 * 复用后端 `image_cache_manager` 缓存能力，将远程图片 URL 转为本地缓存 URL，
 * 避免每次渲染都发起远程请求导致卡顿。
 *
 * 流程：
 * 1. `src` 变化时调用 `getCachedImageUrl(remoteUrl)`
 *    - 命中缓存：直接使用 `cache-image://` 本地 URL（零网络请求）
 *    - 未命中：先用远程 URL 渲染，后端异步下载完成后 emit `image-cached`
 * 2. 监听 `image-cached` 事件，匹配到当前 `src` 时切换为本地 URL
 * 3. 加载中显示旋转 spinner；加载失败或 `src` 为空时显示 fallback 插槽
 *
 * 复用约定：
 * - `getCachedImageUrl`：已有 `@/utils/api/image-cache` 工具
 * - `onImageCached`：已有 `@/composables/useImageCache`，内部使用全局单例 listener
 *   组件卸载时自动移除 handler，无 Tauri unlisten 竞态
 *
 * 使用：
 * ```vue
 * <CachedImage :src="project.logo_url" :alt="project.raw_name" class="w-full h-full object-cover">
 *   <template #fallback><CubeIcon class="w-5 h-5 text-gray-400" /></template>
 * </CachedImage>
 * ```
 */

import { ref, watch, onMounted, useAttrs } from 'vue'
import { getCachedImageUrl } from '@/utils/api/image-cache'
import { onImageCached } from '@/composables/useImageCache'

// 关闭自动继承：class 等 attrs 绑定到内部包裹层 div，img / spinner / fallback 各自独立控制
defineOptions({ inheritAttrs: false })
const attrs = useAttrs()

const props = withDefaults(defineProps<{
  /** 远程图片 URL（为空或 null 时直接渲染 fallback 插槽） */
  src?: string | null
  /** alt 文本 */
  alt?: string
  /** 是否在加载失败时回退到 fallback 插槽（默认 true） */
  fallbackOnError?: boolean
}>(), {
  src: '',
  alt: '',
  fallbackOnError: true,
})

/** 当前用于 <img> 的 URL（本地缓存或远程） */
const displayUrl = ref<string>('')
/** 图片加载失败标记 */
const failed = ref(false)
/** 图片加载完成标记（加载中显示 spinner，完成后显示真实图片） */
const loaded = ref(false)
/** 当前正在等待缓存的远程 URL（用于事件匹配） */
const pendingRemoteUrl = ref<string | null>(null)

/** 拉取缓存 URL 并设置 displayUrl */
async function refresh() {
  failed.value = false
  loaded.value = false
  if (!props.src) {
    displayUrl.value = ''
    pendingRemoteUrl.value = null
    return
  }
  try {
    const result = await getCachedImageUrl(props.src)
    if (!result.url) {
      displayUrl.value = ''
      pendingRemoteUrl.value = null
      return
    }
    displayUrl.value = result.url
    // 命中缓存则无需等待事件；未命中则记录远程 URL 以便事件匹配
    pendingRemoteUrl.value = result.cached ? null : props.src
  } catch {
    // 后端异常时回退到远程 URL
    displayUrl.value = props.src
    pendingRemoteUrl.value = null
  }
}

/** 监听 image-cached 事件，匹配到当前 pending URL 时切换为本地 URL */
onImageCached((remoteUrl, localUrl) => {
  if (pendingRemoteUrl.value === remoteUrl) {
    displayUrl.value = localUrl
    pendingRemoteUrl.value = null
    // 远程图可能已加载失败触发过 fallback，缓存就绪后重置失败标记，恢复真实图片
    failed.value = false
    // 本地缓存图需重新加载，先回到加载中态
    loaded.value = false
  }
})

function handleLoad() {
  loaded.value = true
}

function handleError() {
  if (props.fallbackOnError) {
    failed.value = true
  } else {
    // 不回退到 fallback 时保持 img 可见（即使破图），避免 spinner 常驻
    loaded.value = true
  }
}

onMounted(refresh)
watch(() => props.src, refresh)
</script>

<template>
  <div v-bind="attrs" class="relative overflow-hidden">
    <img
      v-if="displayUrl && !(failed && fallbackOnError)"
      :class="['w-full h-full object-cover transition-opacity duration-150', !loaded ? 'opacity-0' : 'opacity-100']"
      :src="displayUrl"
      :alt="alt"
      @load="handleLoad"
      @error="handleError"
    >
    <div
      v-if="props.src && !failed && !loaded"
      class="absolute inset-0 flex items-center justify-center"
    >
      <svg class="h-5 w-5 animate-spin text-primary-500" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
        <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
      </svg>
    </div>
    <div
      v-if="!props.src || failed"
      class="absolute inset-0 flex items-center justify-center"
    >
      <slot name="fallback" />
    </div>
  </div>
</template>
