<script setup lang="ts">
/**
 * 合成配方槽位热点：单个槽位的背景盒 + 图标 + 数量/标签角标
 */
import { defineAsyncComponent } from 'vue'
const RecipeItemIcon = defineAsyncComponent(() => import('./RecipeItemIcon.vue'))
import type { AtlasLayout } from '@/utils/recipe-generator/resources'
import type { RecipeSlot } from '@/utils/recipe-generator/types'
import type { RecipeLayoutSlotBox } from './recipe-layouts'
import { RECIPE_IMAGE_HEIGHT, RECIPE_IMAGE_WIDTH } from './recipe-layouts'
import type { Display } from './slot-display'

withDefaults(
  defineProps<{
    slotId: RecipeSlot
    box: RecipeLayoutSlotBox
    display: Display | null
    disabled: boolean
    editing: boolean
    barrierDisplay: Display | null
    atlasUrl: string
    atlas: AtlasLayout
    iconSize: number
  }>(),
  { display: null, barrierDisplay: null },
)

const emit = defineEmits<{
  click: [slot: RecipeSlot]
  wheel: [event: WheelEvent, slot: RecipeSlot]
  hover: [event: MouseEvent, display: Display | null]
  leave: []
}>()

function slotBoxStyle(box: RecipeLayoutSlotBox) {
  return {
    left: `${(box.x1 / RECIPE_IMAGE_WIDTH) * 100}%`,
    top: `${(box.y1 / RECIPE_IMAGE_HEIGHT) * 100}%`,
    width: `${((box.x2 - box.x1) / RECIPE_IMAGE_WIDTH) * 100}%`,
    height: `${((box.y2 - box.y1) / RECIPE_IMAGE_HEIGHT) * 100}%`,
  }
}
</script>

<template>
  <div
    class="recipe-layout-hotspot"
    :class="{
      filled: !!display && !disabled,
      editing: editing && !disabled,
      'is-tag': !!display?.members?.length,
      disabled,
    }"
    :style="slotBoxStyle(box)"
    :data-recipe-slot="slotId"
    @click="!disabled && emit('click', slotId)"
    @wheel="!disabled && emit('wheel', $event, slotId)"
    @mouseenter="emit('hover', $event, disabled ? null : display)"
    @mouseleave="emit('leave')"
  >
    <RecipeItemIcon
      v-if="disabled && barrierDisplay"
      :texture="barrierDisplay.texture"
      :atlas-url="atlasUrl"
      :atlas="atlas"
      :size="iconSize"
      :label="barrierDisplay.label"
    />
    <RecipeItemIcon
      v-else-if="display"
      :texture="display.texture"
      :atlas-url="atlasUrl"
      :atlas="atlas"
      :size="iconSize"
      :label="display.label"
    />
    <span v-if="display && !disabled" class="recipe-slot-count">
      {{ display.count }}
    </span>
    <span v-if="display?.members?.length && !disabled" class="recipe-slot-tag-badge">#</span>
  </div>
</template>

<style scoped>
.recipe-layout-hotspot {
  position: absolute;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2px dashed transparent;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.12);
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    background-color 0.15s ease;
}

.recipe-layout-hotspot:hover {
  border-color: var(--color-primary-500);
  background: rgba(255, 255, 255, 0.32);
}

.recipe-layout-hotspot.filled {
  border-style: solid;
  border-color: rgba(255, 255, 255, 0.55);
  background: rgba(0, 0, 0, 0.18);
}

.recipe-layout-hotspot.editing {
  border-color: var(--color-primary-500);
  box-shadow: 0 0 0 2px rgb(var(--color-primary-rgb-500) / 0.4);
}

.recipe-layout-hotspot.disabled {
  border-style: solid;
  border-color: rgba(255, 255, 255, 0.25);
  background: rgba(0, 0, 0, 0.42);
  cursor: not-allowed;
}

.recipe-slot-count {
  position: absolute;
  right: 1px;
  bottom: 0;
  padding: 0 2px;
  border-radius: 2px;
  background: rgba(0, 0, 0, 0.6);
  color: #fff;
  font-size: 10px;
  font-weight: 600;
  line-height: 14px;
  pointer-events: none;
}

.recipe-slot-tag-badge {
  position: absolute;
  top: 2px;
  left: 4px;
  color: var(--color-primary-500);
  font-size: 11px;
  font-weight: 700;
  line-height: 1;
  pointer-events: none;
}
</style>