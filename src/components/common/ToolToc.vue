<script setup lang="ts">
/**
 * 消息区右侧 TOC 导航条（实验性 AI 聊天专用）
 *
 * 工具页已重构为顶部子菜单、不再使用 TOC；本组件现仅服务于实验性
 * AI 聊天页的消息快捷跳转（扫描容器内 [data-toc-card] 生成目录）。
 *
 * 默认收起：只显示一列灰色短横线（每项一条）。
 * hover 整个 TOC 区：展开为带标题的方格条（宽度 + 透明度动画）。
 * 当前可见项（滚动高亮）使用主题色。
 * 点击跳转：在滚动容器内手动计算 scrollTop，预留顶部标题栏偏移，避免被遮住。
 * 条目数 < minItems 时不显示。
 *
 * 扫描时机：除 refreshKey 变化外，通过 MutationObserver 监听容器内
 * `[data-toc-card]` 的新增/移除/标题变更自动重扫（覆盖会话切换/删除等
 * 带过渡动画的 DOM 变动，避免在旧内容淡出期间扫到残留节点）。
 */

import { ref, onMounted, onBeforeUnmount, nextTick, watch } from 'vue'

interface TocItem {
  id: string
  title: string
}

const props = defineProps<{
  /** 递增此值触发重新扫描（如分类切换后） */
  refreshKey?: number | string
  /** 滚动容器选择器，默认 .tools-scroll-container */
  containerSelector?: string
  /** 跳转时顶部预留偏移（避免被遮住），单位 px */
  scrollOffset?: number
  /** 最少条目数，低于则不显示（默认 3） */
  minItems?: number
}>()

const items = ref<TocItem[]>([])
const activeId = ref('')
const hovered = ref(false)
const scrollContainer = ref<HTMLElement | null>(null)
let scrollHandler: (() => void) | null = null
let observer: MutationObserver | null = null

/** 判断一批节点中是否包含 TOC 卡片（自身带 data-toc-card 或后代包含） */
function nodesContainTocCard(nodes: NodeList): boolean {
  for (const n of nodes) {
    if (!(n instanceof Element)) continue
    if (n.hasAttribute('data-toc-card') || n.querySelector('[data-toc-card]')) return true
  }
  return false
}

/** 监听容器内 TOC 卡片的增删/标题变更，自动重扫（不依赖 refreshKey 的时序） */
function observeContainer() {
  observer?.disconnect()
  observer = null
  const container = scrollContainer.value
  if (!container) return
  observer = new MutationObserver((mutations) => {
    let dirty = false
    for (const m of mutations) {
      // attributeFilter 已限定 data-toc-card / data-toc-title，命中即需重扫
      if (m.type === 'attributes') {
        dirty = true
        break
      }
      if (nodesContainTocCard(m.addedNodes) || nodesContainTocCard(m.removedNodes)) {
        dirty = true
        break
      }
    }
    if (dirty) refresh()
  })
  observer.observe(container, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ['data-toc-card', 'data-toc-title'],
  })
}

/** 扫描滚动容器内的 [data-toc-card] 元素，构建 TOC 列表 */
function scanItems() {
  const selector = props.containerSelector || '.tools-scroll-container'
  const container = document.querySelector(selector) as HTMLElement | null
  const root: ParentNode = container || document
  const cards = root.querySelectorAll('[data-toc-card]')
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
  unbindScroll()
  const selector = props.containerSelector || '.tools-scroll-container'
  scrollContainer.value = document.querySelector(selector) as HTMLElement | null
  if (!scrollContainer.value) return

  scrollHandler = () => {
    if (!scrollContainer.value || items.value.length === 0) return
    const containerRect = scrollContainer.value.getBoundingClientRect()
    const offset = props.scrollOffset ?? 16
    let closest: { id: string; dist: number } | null = null

    for (const item of items.value) {
      const el = document.getElementById(item.id)
      if (!el) continue
      const rect = el.getBoundingClientRect()
      // 卡片顶部相对容器顶部的距离，减去预留偏移
      const dist = rect.top - containerRect.top - offset
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

/** 点击 TOC 项跳转：手动计算 scrollTop，预留顶部偏移避免被标题栏遮住 */
function jumpTo(id: string) {
  const container = scrollContainer.value
  const target = document.getElementById(id)
  if (!container || !target) return
  const containerRect = container.getBoundingClientRect()
  const targetRect = target.getBoundingClientRect()
  const offset = props.scrollOffset ?? 16
  // 目标相对容器顶部的距离 - 预留偏移 = 需要滚动的距离
  const delta = targetRect.top - containerRect.top - offset
  container.scrollTo({
    top: container.scrollTop + delta,
    behavior: 'smooth',
  })
  activeId.value = id
}

function refresh() {
  nextTick(() => {
    nextTick(() => {
      setTimeout(() => {
        scanItems()
        bindScroll()
        observeContainer()
      }, 0)
    })
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
  observer?.disconnect()
  observer = null
})
</script>

<template>
  <!-- 条目数达标时才显示。absolute 悬浮，不占布局空间，不影响滚动条位置 -->
  <div
    v-if="items.length >= (props.minItems ?? 3)"
    class="absolute right-4 top-1/2 -translate-y-1/2 z-10"
    @mouseenter="hovered = true"
    @mouseleave="hovered = false"
  >
    <!-- 容器：宽度 + 圆角过渡动画。收起时窄条，展开时带背景的卡片 -->
    <div
      class="flex flex-col items-stretch gap-1 rounded-lg transition-all duration-200 ease-out"
      :class="hovered
        ? 'bg-white/95 backdrop-blur-sm shadow-md border border-gray-100 p-1.5 w-32'
        : 'bg-transparent p-0.5 w-4'"
    >
      <div
        v-for="item in items"
        :key="item.id"
        role="button"
        tabindex="0"
        class="flex items-center rounded-md cursor-pointer outline-none transition-colors duration-150"
        :class="[
          hovered ? 'px-2 py-1.5' : 'px-0 py-1 justify-center',
          activeId === item.id
            ? hovered
              ? 'bg-primary-50 text-primary-700'
              : ''
            : hovered
              ? 'text-gray-600 hover:bg-gray-100'
              : '',
        ]"
        @click="jumpTo(item.id)"
        @keydown.enter="jumpTo(item.id)"
      >
        <!-- 短横线（收起时显示） -->
        <span
          v-if="!hovered"
          class="block rounded-full transition-all duration-200"
          :class="activeId === item.id
            ? 'w-3 h-1 bg-primary-500'
            : 'w-2 h-1 bg-gray-300'"
        />
        <!-- 展开时的标题文字 -->
        <span
          v-if="hovered"
          class="text-xs font-medium truncate transition-opacity duration-200"
          :class="activeId === item.id ? 'text-primary-700' : 'text-gray-700'"
        >{{ item.title }}</span>
      </div>
    </div>
  </div>
</template>
