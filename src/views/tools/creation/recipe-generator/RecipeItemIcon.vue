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

const region = computed(() => (props.texture ? props.atlas.layout[props.texture] : undefined))

const backgroundStyle = computed(() => {
  const regionValue = region.value
  if (!regionValue) return {}
  // 图集内贴图为 16x16 紧密排列，需按 元素尺寸/贴图宽 等比放大背景，
  // 否则 32px+ 元素会同时露出相邻多个贴图
  const scale = props.size / regionValue[2]
  return {
    backgroundImage: `url(${props.atlasUrl})`,
    backgroundSize: `${props.atlas.size[0] * scale}px ${props.atlas.size[1] * scale}px`,
    backgroundPosition: `-${regionValue[0] * scale}px -${regionValue[1] * scale}px`,
  }
})
</script>

<template>
  <Tooltip :text="label">
    <span
      class="recipe-item-icon"
      :style="backgroundStyle"
      :aria-label="label"
    >
      <span v-if="!region" class="recipe-item-icon-placeholder">?</span>
    </span>
  </Tooltip>
</template>

<style scoped>
.recipe-item-icon {
  position: relative;
  display: inline-flex;
  width: v-bind(size + 'px');
  height: v-bind(size + 'px');
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
  border: 1px dashed #c9cdd4;
  border-radius: 2px;
  background: #f2f3f5;
  color: #86909c;
  font-size: 12px;
}
</style>
