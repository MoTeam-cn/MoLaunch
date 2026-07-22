<script setup lang="ts">
/**
 * 工具页右侧 TOC 导航条
 *
 * 自动扫描当前页面内 [data-toc-card] 元素，生成图标方格条。
 * - 每个方格显示工具标题前 2 字
 * - hover 时用 Tooltip 显示完整标题
 * - 点击滚动跳转到对应卡片
 * - 滚动时自动高亮当前可见项
 * - 工具数 < 3 时不显示
 *
 * 用法：父组件切换分类后递增 refreshKey 触发重新扫描。
 */

import { ref, onMounted, onBeforeUnmount, nextTick, watch } from 'vue'
import Tooltip from '@/components/common/Tooltip.vue'

interface TocItem {
  id: string
  title: string
}

const props = defineProps<{
  /** 递增此值触发重新扫描（如分类切换后） */
  refreshKey?: number | string
  /** 滚动容器选择器，默认 .tools-scroll-container */
  containerSelector?: string
}>()

const items = ref<TocItem[]>([])
const activeId = ref('')
const scrollContainer = ref<HTMLElement | null>(null)
let scrollHandler: (() => void) | null = null

/** 扫描 DOM 中的 [data-toc-card] 元素，构建 TOC 列表 */
function scanItems() {
  const cards = document.querySelectorAll('[data-toc-card]')
  const list: TocItem[] = []
  cards.forEach((el) => {
    const id = el.getAttribute('data-toc-card') || ''
    const title = el.getAttribute('data-toc-title') || ''
    if (id && title) list.push({ id, title })
  })
  items.value = list
}

/** 查找滚动容器并绑定 scroll 事件 */
function bindScroll() {
  // 清理旧监听
  unbindScroll()
  const selector = props.containerSelector || '.tools-scroll-container'
  scrollContainer.value = document.querySelector(selector) as HTMLElement | null
  if (!scrollContainer.value) return

  scrollHandler = () => {
    if (!scrollContainer.value || items.value.length === 0) return
    const containerTop = scrollContainer.value.getBoundingClientRect().top
    let closest: { id: string; dist: number } | null = null

    for (const item of items.value) {
      const el = document.getElementById(item.id)
      if (!el) continue
      const rect = el.getBoundingClientRect()
      // 计算卡片顶部与容器顶部的距离（正值=在下方，负值=已滚过）
      const dist = rect.top - containerTop
      // 优先选择距离顶部 20px 以内且最接近 0 的（即刚滚到顶部的卡片）
      // 如果所有卡片都在上方（dist < 0），选最后一个
      if (!closest) {
        closest = { id: item.id, dist }
      } else if (dist >= 0 && dist < Math.abs(closest.dist)) {
        closest = { id: item.id, dist }
      } else if (dist < 0 && closest.dist < 0 && dist > closest.dist) {
        closest = { id: item.id, dist }
      }
    }
    if (closest) activeId.value = closest.id
  }

  scrollContainer.value.addEventListener('scroll', scrollHandler, { passive: true })
  scrollHandler()
}

function unbindScroll() {
  if (scrollHandler && scrollContainer.value) {
    scrollContainer.value.removeEventListener('scroll', scrollHandler)
  }
  scrollHandler = null
}

/** 点击 TOC 项跳转到对应卡片 */
function jumpTo(id: string) {
  const el = document.getElementById(id)
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'start' })
    activeId.value = id
  }
}

/** 取标题前 2 字作为方格标识 */
function shortLabel(title: string): string {
  // 移除常见前缀（如"工具"、"管理"）取核心词
  const cleaned = title.replace(/^(工具|管理|器)$/, '')
  const base = cleaned.length >= 2 ? cleaned : title
  return base.slice(0, 2)
}

function refresh() {
  nextTick(() => {
    scanItems()
    bindScroll()
  })
}

onMounted(() => {
  refresh()
})

watch(
  () => props.refreshKey,
  () => refresh(),
)

onBeforeUnmount(() => {
  unbindScroll()
})
</script>

<template>
  <!-- 工具数 ≥ 3 时才显示 -->
  <div
    v-if="items.length >= 3"
    class="flex flex-col items-center gap-1.5 sticky top-1/2 -translate-y-1/2"
  >
    <div
      v-for="item in items"
      :key="item.id"
      role="button"
      tabindex="0"
      class="w-9 h-9 flex items-center justify-center rounded-md text-xs font-medium cursor-pointer transition-all duration-150 outline-none focus:ring-2 focus:ring-primary-300"
      :class="[
        activeId === item.id
          ? 'bg-primary-500 text-white shadow-sm scale-105'
          : 'bg-gray-100 text-gray-500 hover:bg-gray-200 hover:text-gray-700',
      ]"
      @click="jumpTo(item.id)"
      @keydown.enter="jumpTo(item.id)"
    >
      <Tooltip :text="item.title" position="left" :delay="200">
        <span>{{ shortLabel(item.title) }}</span>
      </Tooltip>
    </div>
  </div>
</template>
