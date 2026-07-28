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
 * 3. 加载失败时显示 fallback 插槽（或空）
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

// 关闭自动继承，class 等 attrs 仅绑定到 <img>，避免污染 fallback 插槽
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
/** 当前正在等待缓存的远程 URL（用于事件匹配） */
const pendingRemoteUrl = ref<string | null>(null)

/** 拉取缓存 URL 并设置 displayUrl */
async function refresh() {
  failed.value = false
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
  }
})

function handleError() {
  if (props.fallbackOnError) {
    failed.value = true
  }
}

onMounted(refresh)
watch(() => props.src, refresh)
</script>

<template>
  <img
    v-if="displayUrl && !(failed && fallbackOnError)"
    v-bind="attrs"
    :src="displayUrl"
    :alt="alt"
    @error="handleError"
  >
  <slot v-else name="fallback" />
</template>
