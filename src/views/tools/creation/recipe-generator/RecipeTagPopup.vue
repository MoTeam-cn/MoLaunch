<script setup lang="ts">
/**
 * 标签槽位悬停浮层：展示标签全部成员物品贴图，内容超宽时左右缓慢滑动
 */
import { nextTick, onBeforeUnmount, reactive, ref, watch } from 'vue'
import type { AtlasLayout } from '@/utils/recipe-generator/resources'
import type { TagDisplay } from '@/utils/recipe-generator/tag-resolve'
import RecipeItemIcon from './RecipeItemIcon.vue'

const props = defineProps<{
  display: TagDisplay
  atlasUrl: string
  atlas: AtlasLayout
  anchor: HTMLElement | null
}>()

const emit = defineEmits<{
  enter: []
  leave: []
}>()

const popupRef = ref<HTMLElement | null>(null)
const viewportRef = ref<HTMLElement | null>(null)
const trackRef = ref<HTMLElement | null>(null)
const isSliding = ref(false)
const pos = reactive({ top: 0, left: 0 })

function onWindowScroll() {
  updatePosition()
}

function updatePosition() {
  const anchor = props.anchor
  const popup = popupRef.value
  if (!anchor || !popup) return
  const rect = anchor.getBoundingClientRect()
  const width = popup.offsetWidth
  const height = popup.offsetHeight
  let top = rect.bottom + 8
  if (top + height > window.innerHeight - 8) top = Math.max(8, rect.top - height - 8)
  pos.left = Math.min(Math.max(8, rect.left), Math.max(8, window.innerWidth - width - 8))
  pos.top = top
}

function setupSlide() {
  const popup = popupRef.value
  const viewport = viewportRef.value
  const track = trackRef.value
  if (!popup || !viewport || !track) return
  const distance = track.scrollWidth - viewport.clientWidth
  if (distance > 0) {
    popup.style.setProperty('--slide-distance', `${distance}px`)
    popup.style.setProperty('--slide-duration', `${Math.max(14, distance / 30)}s`)
    isSliding.value = true
  } else {
    isSliding.value = false
  }
}

watch(
  () => [props.display, props.anchor],
  async () => {
    await nextTick()
    updatePosition()
    setupSlide()
    window.removeEventListener('scroll', onWindowScroll, true)
    window.addEventListener('scroll', onWindowScroll, true)
  },
  { immediate: true },
)

onBeforeUnmount(() => {
  window.removeEventListener('scroll', onWindowScroll, true)
})
</script>

<template>
  <div
    ref="popupRef"
    class="recipe-tag-popup"
    :style="{ top: `${pos.top}px`, left: `${pos.left}px` }"
    @mouseenter="emit('enter')"
    @mouseleave="emit('leave')"
  >
    <div class="recipe-tag-popup-header">
      <span class="recipe-tag-popup-title">{{ display.label }}</span>
      <span class="recipe-tag-popup-count">{{ display.members.length }} 个物品</span>
    </div>
    <div ref="viewportRef" class="recipe-tag-popup-viewport">
      <div ref="trackRef" class="recipe-tag-popup-track" :class="{ 'is-sliding': isSliding }">
        <div
          v-for="member in display.members"
          :key="member.id"
          class="recipe-tag-popup-item"
          :title="member.label"
        >
          <RecipeItemIcon
            :texture="member.texture"
            :atlas-url="atlasUrl"
            :atlas="atlas"
            :size="34"
            :label="member.label"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.recipe-tag-popup {
  position: fixed;
  z-index: 1100;
  width: 320px;
  max-width: min(80vw, 380px);
  padding: 8px 10px;
  border: 1px solid #e5e6eb;
  border-radius: 8px;
  background: #fff;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
}

.recipe-tag-popup-header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 8px;
}

.recipe-tag-popup-title {
  overflow: hidden;
  color: var(--color-primary-600);
  font-size: 0.8rem;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.recipe-tag-popup-count {
  flex-shrink: 0;
  color: #86909c;
  font-size: 0.7rem;
}

.recipe-tag-popup-viewport {
  overflow: hidden;
  border-radius: 4px;
}

.recipe-tag-popup-track {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  width: max-content;
  will-change: transform;
}

.recipe-tag-popup-track.is-sliding {
  animation: recipe-tag-slide var(--slide-duration, 20s) ease-in-out infinite alternate;
}

.recipe-tag-popup-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

@keyframes recipe-tag-slide {
  from {
    transform: translateX(0);
  }
  to {
    transform: translateX(calc(var(--slide-distance, 0px) * -1));
  }
}
</style>
