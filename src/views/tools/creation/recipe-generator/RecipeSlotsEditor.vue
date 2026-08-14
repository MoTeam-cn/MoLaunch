<script setup lang="ts">
/**
 * 合成配方槽位编辑器：工作台背景图 + 槽位热点
 *
 * 槽位由布局数据（背景图 + 像素盒）驱动，各配方类型显示对应工作台界面；
 * 点击空热点请求编辑（父组件弹抽屉选择），点击已放置槽位清除；
 * 滚轮可调整槽位数量（1-64）；2x2 时禁用槽以 barrier 遮罩。
 */
import { computed, onBeforeUnmount, onMounted, ref, defineAsyncComponent } from 'vue'
import type { RecipeSlot, RecipeSlotContext, SlotValue } from '@/utils/recipe-generator/types'
import type { AtlasLayout } from '@/utils/recipe-generator/resources'
import {
  RECIPE_IMAGE_HEIGHT,
  RECIPE_IMAGE_WIDTH,
  type RecipeLayout,
  type RecipeLayoutSlotBox,
} from './recipe-layouts'
import { resolveTagDisplay, type TagDisplay, type TagMember } from '@/utils/recipe-generator/tag-resolve'
const RecipeItemIcon = defineAsyncComponent(() => import('./RecipeItemIcon.vue'))
const RecipeTagPopup = defineAsyncComponent(() => import('./RecipeTagPopup.vue'))

const TWO_BY_TWO_DISABLED_SLOTS = new Set<RecipeSlot>([
  'crafting.3',
  'crafting.6',
  'crafting.7',
  'crafting.8',
  'crafting.9',
])

const props = withDefaults(
  defineProps<{
    layout: RecipeLayout
    values: Partial<Record<RecipeSlot, SlotValue>>
    context: RecipeSlotContext
    atlasUrl: string
    atlas: AtlasLayout
    twoByTwo?: boolean
    editingSlot?: RecipeSlot | null
  }>(),
  { twoByTwo: false, editingSlot: null },
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

const barrierDisplay = computed<Display | null>(() => {
  const item = props.context.itemsById['minecraft:barrier']
  return item ? { texture: item.texture, label: item.name, count: 1 } : null
})

const layoutSlots = computed(() =>
  (Object.entries(props.layout.slots) as [RecipeSlot, RecipeLayoutSlotBox][]).map(([slot, box]) => ({
    slot,
    box,
    display: displayFor(props.values[slot]),
    disabled: props.twoByTwo && TWO_BY_TWO_DISABLED_SLOTS.has(slot),
  })),
)

const stageRef = ref<HTMLElement | null>(null)
const stageWidth = ref(0)
let stageObserver: ResizeObserver | null = null

onMounted(() => {
  if (!stageRef.value) return
  stageObserver = new ResizeObserver((entries) => {
    stageWidth.value = entries[0]?.contentRect.width ?? 0
  })
  stageObserver.observe(stageRef.value)
})

onBeforeUnmount(() => {
  clearCloseTimer()
  stageObserver?.disconnect()
  stageObserver = null
})

function slotBoxStyle(box: RecipeLayoutSlotBox) {
  return {
    left: `${(box.x1 / RECIPE_IMAGE_WIDTH) * 100}%`,
    top: `${(box.y1 / RECIPE_IMAGE_HEIGHT) * 100}%`,
    width: `${((box.x2 - box.x1) / RECIPE_IMAGE_WIDTH) * 100}%`,
    height: `${((box.y2 - box.y1) / RECIPE_IMAGE_HEIGHT) * 100}%`,
  }
}

function slotIconSize(box: RecipeLayoutSlotBox): number {
  if (!stageWidth.value) return 32
  const backgroundSize = Math.min(box.x2 - box.x1, box.y2 - box.y1)
  return Math.max(1, Math.round((backgroundSize / RECIPE_IMAGE_WIDTH) * stageWidth.value * 0.9))
}

function onSlotClick(slot: RecipeSlot) {
  if (props.values[slot]) {
    closeHover()
    emit('update-slot', slot, undefined)
  } else {
    emit('edit-slot', slot)
  }
}

function onSlotWheel(event: WheelEvent, slot: RecipeSlot) {
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
</script>

<template>
  <div class="recipe-slots-editor">
    <div
      ref="stageRef"
      class="recipe-layout-stage"
      :style="{ backgroundImage: `url('${layout.image}')` }"
    >
      <div
        v-for="entry in layoutSlots"
        :key="entry.slot"
        class="recipe-layout-hotspot"
        :class="{
          filled: !!entry.display && !entry.disabled,
          editing: editingSlot === entry.slot && !entry.disabled,
          'is-tag': !!entry.display?.members?.length,
          disabled: entry.disabled,
        }"
        :style="slotBoxStyle(entry.box)"
        :data-recipe-slot="entry.slot"
        @click="!entry.disabled && onSlotClick(entry.slot)"
        @wheel="!entry.disabled && onSlotWheel($event, entry.slot)"
        @mouseenter="onSlotHover($event, entry.disabled ? null : entry.display)"
        @mouseleave="scheduleClose"
      >
        <RecipeItemIcon
          v-if="entry.disabled && barrierDisplay"
          :texture="barrierDisplay.texture"
          :atlas-url="atlasUrl"
          :atlas="atlas"
          :size="slotIconSize(entry.box)"
          :label="barrierDisplay.label"
        />
        <RecipeItemIcon
          v-else-if="entry.display"
          :texture="entry.display.texture"
          :atlas-url="atlasUrl"
          :atlas="atlas"
          :size="slotIconSize(entry.box)"
          :label="entry.display.label"
        />
        <span v-if="entry.display && !entry.disabled" class="recipe-slot-count">
          {{ entry.display.count }}
        </span>
        <span v-if="entry.display?.members?.length && !entry.disabled" class="recipe-slot-tag-badge">#</span>
      </div>
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
  width: 100%;
}

.recipe-layout-stage {
  position: relative;
  width: 100%;
  max-width: 30rem;
  aspect-ratio: 696 / 292;
  border: 1px solid #e5e6eb;
  border-radius: 8px;
  background-color: #f7f8fa;
  background-repeat: no-repeat;
  background-size: cover;
}

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
