<script setup lang="ts">
/**
 * 物品图标：从内置纹理图集切图显示（16x16 源图按像素缩放）
 */
import { computed, defineAsyncComponent } from 'vue'
import type { AtlasLayout } from '@/utils/recipe-generator/resources'
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))

const props = withDefaults(
  defineProps<{
    texture: string | null
    atlasUrl: string
    atlas: AtlasLayout
    size?: number
    label?: string
  }>(),
  { size: 32, label: '' },
)

const region = computed(() =>
  props.texture ? props.atlas?.layout?.[props.texture] : undefined,
)

/** 没有贴图时用名称首字母兜底，至少能看出是哪个物品 */
const initial = computed(() => (props.label || '?').trim().charAt(0).toUpperCase() || '?')

const iconStyle = computed<Record<string, string>>(() => {
  // 尺寸直接用内联样式：写在 CSS 里要靠 v-bind() 拼字符串，变量名会被表达式污染
  const style: Record<string, string> = {
    width: `${props.size}px`,
    height: `${props.size}px`,
  }
  const regionValue = region.value
  if (!regionValue) return style
  // 图集内贴图为 16x16 紧密排列，需按 元素尺寸/贴图宽 等比放大背景，
  // 否则 32px+ 元素会同时露出相邻多个贴图
  const scale = props.size / regionValue[2]
  style.backgroundImage = `url(${props.atlasUrl})`
  style.backgroundSize = `${props.atlas.size[0] * scale}px ${props.atlas.size[1] * scale}px`
  style.backgroundPosition = `-${regionValue[0] * scale}px -${regionValue[1] * scale}px`
  return style
})
</script>

<template>
  <Tooltip :text="label">
    <span
      class="recipe-item-icon"
      :style="iconStyle"
      :aria-label="label"
    >
      <span v-if="!region" class="recipe-item-icon-placeholder">{{ initial }}</span>
    </span>
  </Tooltip>
</template>

<style lang="scss" scoped>
.recipe-item-icon {
  position: relative;
  display: inline-flex;
  flex: none;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  border-radius: 2px;
  background-color: transparent;
  background-repeat: no-repeat;
  image-rendering: pixelated;
}

.recipe-item-icon-placeholder {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  border: 1px dashed var(--border-strong);
  border-radius: 2px;
  background: var(--surface-1);
  color: var(--text-3);
  font-size: 12px;
}
</style>
