<script setup lang="ts">
/**
 * 合成配方槽位网格编辑器
 *
 * crafting：3x3（或 2x2）网格 + 结果槽；其余类型：一行输入槽 + 结果槽。
 * 点击已放置的槽位可清除；结果槽滚轮可调整产出数量（1-64）。
 */
import { computed } from 'vue'
import type { RecipeSlot, RecipeSlotContext, SlotValue } from '@/utils/recipe-generator/types'
import type { AtlasLayout } from '@/utils/recipe-generator/resources'
import RecipeItemIcon from './RecipeItemIcon.vue'

const props = withDefaults(
  defineProps<{
    slots: RecipeSlot[]
    values: Partial<Record<RecipeSlot, SlotValue>>
    context: RecipeSlotContext
    atlasUrl: string
    atlas: AtlasLayout
    twoByTwo?: boolean
    resultSlot?: RecipeSlot | null
    gridSlots?: RecipeSlot[]
  }>(),
  { twoByTwo: false, resultSlot: null, gridSlots: () => [] },
)

const emit = defineEmits<{
  'update-slot': [slot: RecipeSlot, value: SlotValue | undefined]
  'update-count': [slot: RecipeSlot, count: number]
}>()

type Display = { texture: string | null; label: string; count: number }

function displayFor(value: SlotValue | undefined): Display | null {
  if (!value) return null
  if (value.kind === 'item') {
    const item = props.context.itemsById[value.id]
    const name = item?.name ?? value.id
    return {
      texture: item?.texture ?? null,
      label: item && item.zh ? `${name}（${item.zh}）` : name,
      count: value.count ?? 1,
    }
  }
  if (value.kind === 'custom_item') {
    const item = props.context.customItemsByUid[value.uid]
    return {
      texture: item?.texture || null,
      label: item?.name ?? '未知自定义物品',
      count: value.count ?? 1,
    }
  }
  if (value.kind === 'vanilla_tag') return { texture: null, label: `#${value.id}`, count: 1 }
  return {
    texture: null,
    label: `#${props.context.customTagsByUid[value.uid]?.id ?? '未知标签'}`,
    count: 1,
  }
}

const gridDisplay = computed(() => {
  const size = props.twoByTwo ? 2 : 3
  const cells = Array.from({ length: size * size }, (_, index) => {
    const slot = props.gridSlots[index]
    return slot ? { slot, display: displayFor(props.values[slot]) } : null
  })
  return { size, cells }
})

const rowSlots = computed(() =>
  props.slots
    .filter((slot) => !props.gridSlots.includes(slot))
    .map((slot) => ({ slot, display: displayFor(props.values[slot]) })),
)

function onSlotClick(slot: RecipeSlot) {
  if (props.values[slot]) emit('update-slot', slot, undefined)
}

function onResultWheel(event: WheelEvent, slot: RecipeSlot) {
  if (slot !== props.resultSlot) return
  const value = props.values[slot]
  if (!value || (value.kind !== 'item' && value.kind !== 'custom_item')) return
  event.preventDefault()
  const current = value.count ?? 1
  const delta = event.deltaY > 0 ? -1 : 1
  emit('update-count', slot, Math.max(1, Math.min(64, current + delta)))
}
</script>

<template>
  <div class="recipe-slots-editor">
    <!-- crafting 网格 -->
    <div v-if="gridDisplay.size" class="recipe-slot-grid-wrap">
      <div class="recipe-slot-grid" :class="{ 'is-two-by-two': twoByTwo }">
        <button
          v-for="(cell, index) in gridDisplay.cells"
          :key="index"
          type="button"
          class="recipe-slot-cell"
          :class="{ filled: cell?.display }"
          @click="cell?.slot && onSlotClick(cell.slot)"
        >
          <RecipeItemIcon
            v-if="cell?.display"
            :texture="cell.display.texture"
            :atlas-url="atlasUrl"
            :atlas="atlas"
            :size="34"
            :label="cell.display.label"
          />
          <span v-if="cell?.display" class="recipe-slot-count">
            {{ cell.display.count > 1 ? cell.display.count : '' }}
          </span>
        </button>
      </div>
      <template v-if="resultSlot">
        <span class="recipe-grid-arrow">→</span>
        <button
          type="button"
          class="recipe-slot-cell recipe-result-cell"
          :class="{ filled: values[resultSlot] }"
          @click="onSlotClick(resultSlot)"
          @wheel="onResultWheel($event, resultSlot)"
        >
          <RecipeItemIcon
            v-if="displayFor(values[resultSlot])"
            :texture="displayFor(values[resultSlot])?.texture"
            :atlas-url="atlasUrl"
            :atlas="atlas"
            :size="38"
            :label="displayFor(values[resultSlot])?.label"
          />
          <span v-if="displayFor(values[resultSlot])" class="recipe-slot-count">
            {{ displayFor(values[resultSlot])?.count }}
          </span>
        </button>
      </template>
    </div>

    <!-- 其余类型：输入槽 + 结果槽 一行 -->
    <div v-else class="recipe-slot-row">
      <div
        v-for="entry in rowSlots"
        :key="entry.slot"
        class="recipe-slot-item"
        :class="{ 'is-result': entry.slot === resultSlot }"
      >
        <button
          type="button"
          class="recipe-slot-cell"
          :class="{ filled: entry.display }"
          @click="onSlotClick(entry.slot)"
          @wheel="onResultWheel($event, entry.slot)"
        >
          <RecipeItemIcon
            v-if="entry.display"
            :texture="entry.display.texture"
            :atlas-url="atlasUrl"
            :atlas="atlas"
            :size="36"
            :label="entry.display.label"
          />
          <span v-if="entry.display" class="recipe-slot-count">
            {{ entry.display.count }}
          </span>
        </button>
        <span class="recipe-slot-caption">{{ entry.slot.split('.')[1] }}</span>
      </div>
      <span v-if="resultSlot" class="recipe-grid-arrow">→</span>
    </div>
  </div>
</template>

<style scoped>
.recipe-slots-editor {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
}

.recipe-slot-grid-wrap {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.recipe-slot-grid {
  display: grid;
  grid-template-columns: repeat(3, 3.25rem);
  grid-auto-rows: 3.25rem;
  gap: 0.4rem;
  padding: 0.5rem;
  border: 1px solid #e5e6eb;
  border-radius: 6px;
  background: #f7f8fa;
}

.recipe-slot-grid.is-two-by-two {
  grid-template-columns: repeat(2, 3.25rem);
}

.recipe-slot-cell {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 3.25rem;
  height: 3.25rem;
  border: 2px dashed #e5e6eb;
  border-radius: 4px;
  background: #fff;
  cursor: pointer;
  transition: border-color 0.15s ease;
}

.recipe-slot-cell:hover {
  border-color: #165dff;
}

.recipe-slot-cell.filled {
  border-style: solid;
  border-color: #c9cdd4;
  background: #fff;
}

.recipe-result-cell {
  width: 3.75rem;
  height: 3.75rem;
  border-style: solid;
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

.recipe-grid-arrow {
  color: #86909c;
  font-size: 1.25rem;
}

.recipe-slot-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
}

.recipe-slot-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.25rem;
}

.recipe-slot-caption {
  color: #86909c;
  font-size: 0.65rem;
  text-transform: capitalize;
}
</style>
