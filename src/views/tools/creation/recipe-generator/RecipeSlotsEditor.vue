<script setup lang="ts">
/**
 * 合成配方槽位网格编辑器
 *
 * crafting：3x3（或 2x2）网格 + 结果槽；其余类型：一行输入槽 + 结果槽。
 * 点击空格子请求编辑（父组件弹抽屉选择），点击已放置槽位可清除；
 * 结果槽滚轮可调整产出数量（1-64）；当前编辑中的槽位高亮。
 */
import { computed, onBeforeUnmount, ref } from 'vue'
import type { RecipeSlot, RecipeSlotContext, SlotValue } from '@/utils/recipe-generator/types'
import type { AtlasLayout } from '@/utils/recipe-generator/resources'
import { slotCaption } from '@/utils/recipe-generator/formatter'
import { resolveTagDisplay, type TagDisplay, type TagMember } from '@/utils/recipe-generator/tag-resolve'
import RecipeItemIcon from './RecipeItemIcon.vue'
import RecipeTagPopup from './RecipeTagPopup.vue'

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
    editingSlot?: RecipeSlot | null
  }>(),
  { twoByTwo: false, resultSlot: null, gridSlots: () => [], editingSlot: null },
)

const emit = defineEmits<{
  'update-slot': [slot: RecipeSlot, value: SlotValue | undefined]
  'update-count': [slot: RecipeSlot, count: number]
  'edit-slot': [slot: RecipeSlot]
}>()

type Display = { texture: string | null; label: string; count: number; members?: TagMember[] }

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
  if (value.kind === 'vanilla_tag' || value.kind === 'custom_tag') {
    const display = resolveTagDisplay(value, props.context)
    return { texture: display.texture, label: display.label, count: 1, members: display.members }
  }
  return null
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

const resultDisplay = computed(() =>
  displayFor(props.resultSlot ? props.values[props.resultSlot] : undefined),
)

function onSlotClick(slot: RecipeSlot) {
  if (props.values[slot]) emit('update-slot', slot, undefined)
  else if (slot !== props.resultSlot) emit('edit-slot', slot)
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

const hover = ref<TagDisplay | null>(null)
const hoverAnchor = ref<HTMLElement | null>(null)
let closeTimer: ReturnType<typeof setTimeout> | null = null

function onSlotHover(event: MouseEvent, display: Display | null) {
  if (!display?.members?.length) return
  clearCloseTimer()
  hover.value = { texture: display.texture, label: display.label, members: display.members }
  hoverAnchor.value = event.currentTarget as HTMLElement
}

function closeHover() {
  hover.value = null
  hoverAnchor.value = null
}

function scheduleClose() {
  clearCloseTimer()
  closeTimer = setTimeout(() => {
    closeTimer = null
    closeHover()
  }, 250)
}

function clearCloseTimer() {
  if (closeTimer) {
    clearTimeout(closeTimer)
    closeTimer = null
  }
}

onBeforeUnmount(clearCloseTimer)
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
          :class="{
            filled: cell?.display,
            editing: editingSlot === cell?.slot,
            'is-tag': !!cell?.display?.members?.length,
          }"
          @click="cell?.slot && onSlotClick(cell.slot)"
          @mouseenter="onSlotHover($event, cell?.display ?? null)"
          @mouseleave="scheduleClose"
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
          <span v-if="cell?.display?.members?.length" class="recipe-slot-tag-badge">#</span>
        </button>
      </div>
      <template v-if="resultSlot">
        <span class="recipe-grid-arrow">→</span>
        <button
          type="button"
          class="recipe-slot-cell recipe-result-cell"
          :class="{ filled: values[resultSlot], 'is-tag': !!resultDisplay?.members?.length }"
          @click="onSlotClick(resultSlot)"
          @wheel="onResultWheel($event, resultSlot)"
          @mouseenter="onSlotHover($event, resultDisplay)"
          @mouseleave="scheduleClose"
        >
          <RecipeItemIcon
            v-if="resultDisplay"
            :texture="resultDisplay.texture"
            :atlas-url="atlasUrl"
            :atlas="atlas"
            :size="38"
            :label="resultDisplay.label"
          />
          <span v-if="resultDisplay" class="recipe-slot-count">
            {{ resultDisplay.count }}
          </span>
          <span v-if="resultDisplay?.members?.length" class="recipe-slot-tag-badge">#</span>
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
          :class="{
            filled: entry.display,
            editing: editingSlot === entry.slot,
            'is-tag': !!entry.display?.members?.length,
          }"
          @click="onSlotClick(entry.slot)"
          @wheel="onResultWheel($event, entry.slot)"
          @mouseenter="onSlotHover($event, entry.display)"
          @mouseleave="scheduleClose"
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
          <span v-if="entry.display?.members?.length" class="recipe-slot-tag-badge">#</span>
        </button>
        <span class="recipe-slot-caption">{{ slotCaption(entry.slot) }}</span>
      </div>
      <span v-if="resultSlot" class="recipe-grid-arrow">→</span>
    </div>

    <Teleport to="body">
      <RecipeTagPopup
        v-if="hover"
        :display="hover"
        :atlas-url="atlasUrl"
        :atlas="atlas"
        :anchor="hoverAnchor"
        @enter="clearCloseTimer"
        @leave="scheduleClose"
      />
    </Teleport>
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
  border-color: var(--color-primary-500);
}

.recipe-slot-cell.filled {
  border-style: solid;
  border-color: #c9cdd4;
  background: #fff;
}

.recipe-slot-cell.editing {
  border-color: var(--color-primary-500);
  box-shadow: 0 0 0 2px rgb(var(--color-primary-rgb-500) / 0.25);
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
