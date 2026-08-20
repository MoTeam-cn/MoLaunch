<script setup lang="ts">
/**
 * 合成配方槽位选择抽屉：方向导航 + 物品/标签调色板
 */
import { computed, ref, defineAsyncComponent } from 'vue'
import { ArrowDownIcon, ArrowLeftIcon, ArrowRightIcon, ArrowUpIcon } from '@heroicons/vue/24/outline'
const Drawer = defineAsyncComponent(() => import('@/components/common/Drawer.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const SegmentedButtons = defineAsyncComponent(() => import('@/components/common/SegmentedButtons.vue'))
const ItemPalette = defineAsyncComponent(() => import('./ItemPalette.vue'))
const TagPalette = defineAsyncComponent(() => import('./TagPalette.vue'))
import { slotCaption } from '@/utils/recipe-generator/formatter'
import type { AssetItem, AtlasLayout } from '@/utils/recipe-generator/resources'
import type { RecipeSlot, SlotValue } from '@/utils/recipe-generator/types'
import { PALETTE_TAB_OPTIONS } from './recipe-state'

const props = defineProps<{
  visible: boolean
  editingSlot: RecipeSlot | null
  slots: RecipeSlot[]
  recipeType: string
  twoByTwo: boolean
  items: AssetItem[]
  tags: Record<string, string[]>
  atlas: AtlasLayout
  atlasUrl: string
}>()

const emit = defineEmits<{
  'update:visible': [visible: boolean]
  'move-to': [slot: RecipeSlot]
  pick: [value: SlotValue]
}>()

const activeTab = ref<'items' | 'tags'>('items')

const drawerTitle = computed(() => {
  if (!props.editingSlot) return '选择物品'
  const label = slotCaption(props.editingSlot)
  const isCraftingGrid = props.editingSlot.startsWith('crafting.') && !props.editingSlot.endsWith('.result')
  return isCraftingGrid ? `选择物品（第 ${label} 格）` : `选择${label}`
})

const drawerHint = computed(() => {
  if (!props.editingSlot) return ''
  const label = slotCaption(props.editingSlot)
  const isCraftingGrid = props.editingSlot.startsWith('crafting.') && !props.editingSlot.endsWith('.result')
  return isCraftingGrid ? `第 ${label} 格` : label
})

/** 当前编辑槽位四方向可移动性（crafting 按网格行列，其余为线性列表） */
const drawerNav = computed(() => {
  if (!props.editingSlot) return null
  const slots = props.slots
  const index = slots.indexOf(props.editingSlot)
  if (index < 0) return null
  const size = props.recipeType === 'crafting' ? (props.twoByTwo ? 2 : 3) : 0
  if (size > 0) {
    const col = index % size
    const row = Math.floor(index / size)
    return {
      up: row > 0,
      down: row < size - 1,
      left: col > 0,
      right: col < size - 1,
    }
  }
  return {
    up: index > 0,
    down: index < slots.length - 1,
    left: index > 0,
    right: index < slots.length - 1,
  }
})

function moveEditing(direction: 'up' | 'down' | 'left' | 'right') {
  const nav = drawerNav.value
  if (!nav || !nav[direction]) return
  const slots = props.slots
  const index = slots.indexOf(props.editingSlot!)
  const size = props.recipeType === 'crafting' ? (props.twoByTwo ? 2 : 3) : 0
  let next: number
  if (size > 0) {
    if (direction === 'up') next = index - size
    else if (direction === 'down') next = index + size
    else if (direction === 'left') next = index - 1
    else next = index + 1
  } else {
    next = direction === 'up' || direction === 'left' ? index - 1 : index + 1
  }
  emit('move-to', slots[next])
}
</script>

<template>
  <Drawer
    v-model:visible="visible"
    :title="drawerTitle"
    placement="right"
    :width="380"
    :mask-closable="true"
    :esc-to-close="true"
  >
    <div class="recipe-drawer-palette">
      <div class="recipe-drawer-nav">
        <p class="recipe-drawer-hint">您正在为「{{ drawerHint }}」选择物品</p>
        <div class="recipe-drawer-dpad">
          <Tooltip text="上移一格">
            <button
              class="recipe-dpad-btn"
              :disabled="!drawerNav?.up"
              aria-label="上移一格"
              @click="moveEditing('up')"
            >
              <ArrowUpIcon class="recipe-dpad-icon" />
            </button>
          </Tooltip>
          <Tooltip text="下移一格">
            <button
              class="recipe-dpad-btn"
              :disabled="!drawerNav?.down"
              aria-label="下移一格"
              @click="moveEditing('down')"
            >
              <ArrowDownIcon class="recipe-dpad-icon" />
            </button>
          </Tooltip>
          <Tooltip text="左移一格">
            <button
              class="recipe-dpad-btn"
              :disabled="!drawerNav?.left"
              aria-label="左移一格"
              @click="moveEditing('left')"
            >
              <ArrowLeftIcon class="recipe-dpad-icon" />
            </button>
          </Tooltip>
          <Tooltip text="右移一格">
            <button
              class="recipe-dpad-btn"
              :disabled="!drawerNav?.right"
              aria-label="右移一格"
              @click="moveEditing('right')"
            >
              <ArrowRightIcon class="recipe-dpad-icon" />
            </button>
          </Tooltip>
        </div>
      </div>
      <SegmentedButtons v-model="activeTab" :options="PALETTE_TAB_OPTIONS" button-class="flex-1" />
      <ItemPalette
        v-if="activeTab === 'items'"
        :items="items"
        :atlas-url="atlasUrl"
        :atlas="atlas"
        @pick="emit('pick', $event)"
      />
      <TagPalette v-else :tags="tags" @pick="emit('pick', $event)" />
    </div>
  </Drawer>
</template>

<style scoped>
.recipe-drawer-palette {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  height: 100%;
  min-height: 0;
}

.recipe-drawer-nav {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}

.recipe-drawer-hint {
  color: #86909c;
  font-size: 0.75rem;
}

.recipe-drawer-dpad {
  display: flex;
  gap: 0.25rem;
}

.recipe-dpad-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.5rem;
  height: 1.5rem;
  border: 1px solid #e5e6eb;
  border-radius: 4px;
  background: #fff;
  color: #4e5969;
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    background-color 0.15s ease,
    color 0.15s ease;
}

.recipe-dpad-btn:hover:not(:disabled) {
  border-color: var(--color-primary-500);
  color: var(--color-primary-500);
}

.recipe-dpad-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

.recipe-dpad-icon {
  width: 0.9rem;
  height: 0.9rem;
}
</style>