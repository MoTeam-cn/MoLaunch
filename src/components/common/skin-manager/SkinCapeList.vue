<script setup lang="ts">
/**
 * 披风列表（仅微软账号）
 *
 * - 展示所有可用披风的图标，点击装备
 * - 图标由披风 PNG 纹理裁剪而来（getCapeIcon），非完整披风图
 * - 披风图片缓存由 getSkinCapeInfo 返回时填充（cached_url 字段），无需额外请求
 * - 未命中缓存的披风监听 image-cached 事件，下载完成后重新加载
 * - 当前已装备披风高亮 + "取消当前披风"按钮
 * - emit equip/unequip，业务逻辑由父组件处理
 */
import { ref, watch } from 'vue'
import type { SkinCapeInfo } from '@/utils/tauri'
import { getCapeIcon } from '@/utils/cape-icon'
import { onImageCached } from '@/composables/useImageCache'
import Tooltip from '@/components/common/Tooltip.vue'

const props = defineProps<{
  capes: SkinCapeInfo['capes']
  activeCape: SkinCapeInfo['capes'][number] | null
  uploading: boolean
}>()

const emit = defineEmits<{
  equip: [capeId: string]
  unequip: []
}>()

/** cape id → 图标 dataURL */
const iconMap = ref<Map<string, string>>(new Map())
/** 加载失败的 cape id 集合，回退到占位图标 */
const failedIds = ref<Set<string>>(new Set())
/** cape id → 远程 URL（未命中缓存时记录，用于 image-cached 事件匹配后重新加载） */
const pendingRemoteUrls = ref<Map<string, string>>(new Map())

/** 占位 SVG data URL（披风形状图标） */
const placeholderSvg = 'data:image/svg+xml,' + encodeURIComponent(
  '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="%239ca3af">' +
  '<path d="M3 5a2 2 0 012-2h10a2 2 0 012 2v10a2 2 0 01-2 2H5a2 2 0 01-2-2V5zm2 1a1 1 0 011-1h8a1 1 0 110 2H6a1 1 0 01-1-1z"/>' +
  '</svg>'
)

/**
 * 为单个披风加载图标
 *
 * 直接使用 getSkinCapeInfo 返回的 cached_url（缓存命中时为本地 URL，未命中时为远程 URL）
 * 未命中缓存时记录远程 URL，等待 image-cached 事件后重新加载
 */
async function loadIcon(cape: SkinCapeInfo['capes'][number]) {
  // 优先用 cached_url，回退到 url
  const imageUrl = cape.cached_url || cape.url
  if (!imageUrl) {
    failedIds.value.add(cape.id)
    return
  }
  try {
    const icon = await getCapeIcon(imageUrl)
    iconMap.value.set(cape.id, icon)
    // 未命中缓存时记录远程 URL，用于事件匹配
    if (cape.cached === false && cape.url) {
      pendingRemoteUrls.value.set(cape.id, cape.url)
    } else {
      pendingRemoteUrls.value.delete(cape.id)
    }
  } catch {
    failedIds.value.add(cape.id)
  }
}

/** 为所有披风加载图标 */
async function loadIcons() {
  iconMap.value = new Map()
  failedIds.value = new Set()
  pendingRemoteUrls.value = new Map()
  for (const cape of props.capes) {
    await loadIcon(cape)
  }
}

/** 监听 image-cached 事件，当披风 PNG 下载完成后重新加载图标（从本地缓存读取） */
onImageCached((remoteUrl) => {
  for (const [capeId, pendingUrl] of pendingRemoteUrls.value) {
    if (pendingUrl === remoteUrl) {
      const cape = props.capes.find(c => c.id === capeId)
      if (cape) loadIcon(cape)
      break
    }
  }
})

watch(() => props.capes, loadIcons, { immediate: true, deep: true })
</script>

<template>
  <div class="rounded-lg border border-gray-100 p-4 md:col-span-2">
    <div class="mb-3 flex items-center justify-between">
      <div class="text-sm font-medium text-gray-700">披风列表</div>
      <!-- 保留原生 button：移除披风按钮（px-2 py-1 text-xs + border），
           Button.vue 的 scoped size 类固定 padding 会破坏紧凑尺寸 -->
      <button
        v-if="activeCape"
        class="rounded-md border border-red-200 px-2 py-1 text-xs text-red-500 transition-colors hover:bg-red-50 disabled:opacity-50"
        :disabled="uploading"
        @click="emit('unequip')"
      >取消当前披风</button>
    </div>
    <div v-if="capes.length > 0" class="grid grid-cols-2 gap-2 sm:grid-cols-3 md:grid-cols-4">
      <!-- 保留原生 button：披风列表项（group flex + border + 选中态），
           Button.vue 的 scoped size 类与布局不适合网格列表项 -->
      <button
        v-for="cape in capes"
        :key="cape.id"
        class="group flex items-center gap-2 rounded-md border p-2 transition-colors disabled:opacity-50"
        :class="cape.state === 'ACTIVE' ? 'border-primary-500 bg-primary-50' : 'border-gray-200 hover:bg-gray-50'"
        :disabled="uploading || cape.state === 'ACTIVE'"
        @click="emit('equip', cape.id)"
      >
        <!-- 披风图标（从披风纹理裁剪） -->
        <div class="flex h-8 w-5 flex-none items-center justify-center overflow-hidden rounded bg-gray-50">
          <img
            v-if="iconMap.has(cape.id)"
            :src="iconMap.get(cape.id)"
            :alt="cape.display_name"
            class="h-full w-full object-contain"
            style="image-rendering: pixelated;"
          />
          <img
            v-else
            :src="placeholderSvg"
            class="h-5 w-5"
            alt="placeholder"
          />
        </div>
        <!-- 名称 -->
        <div class="flex min-w-0 flex-1 items-center gap-1">
          <Tooltip :text="cape.display_name" position="top">
            <span
              class="text-xs"
              :class="cape.state === 'ACTIVE' ? 'text-primary-700 font-medium' : 'text-gray-600'"
            >{{ cape.display_name }}</span>
          </Tooltip>
        </div>
      </button>
    </div>
    <div v-else class="py-6 text-center text-xs text-gray-400">暂无披风</div>
  </div>
</template>
