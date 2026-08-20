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
import type { RecipeLayout } from './recipe-layouts'
import { RECIPE_IMAGE_WIDTH } from './recipe-layouts'
import { barrierDisplayFor, displayFor, type Display } from './slot-display'
const RecipeSlotHotspot = defineAsyncComponent(() => import('./RecipeSlotHotspot.vue'))
const RecipeTagPopup = defineAsyncComponent(() => import('./RecipeTagPopup.vue'))
import type { TagDisplay } from '@/utils/recipe-generator/tag-resolve'

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

const layoutSlots = computed(() =>
  (Object.entries(props.layout.slots) as [RecipeSlot, { x1: number; y1: number; x2: number; y2: number }][]).map(([slot, box]) => ({
    slot,
    box,
    display: displayFor(props.values[slot], props.context),
    disabled: props.twoByTwo && TWO_BY_TWO_DISABLED_SLOTS.has(slot),
  })),
)

const barrierDisplay = computed<Display | null>(() => barrierDisplayFor(props.context))

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

function slotIconSize(box: { x1: number; y1: number; x2: number; y2: number }): number {
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
      <RecipeSlotHotspot
        v-for="entry in layoutSlots"
        :key="entry.slot"
        :slot="entry.slot"
        :box="entry.box"
        :display="entry.display"
        :disabled="entry.disabled"
        :editing="editingSlot === entry.slot"
        :barrier-display="barrierDisplay"
        :atlas-url="atlasUrl"
        :atlas="atlas"
        :icon-size="slotIconSize(entry.box)"
        @click="onSlotClick"
        @wheel="onSlotWheel"
        @hover="onSlotHover"
        @leave="scheduleClose"
      />
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
</style>