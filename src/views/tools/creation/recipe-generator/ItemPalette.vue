<script setup lang="ts">
/**
 * 合成配方调色板：按名称/ID 搜索物品，点击放置到槽位
 */
import { computed, ref, defineAsyncComponent } from 'vue'
import { RecycleScroller } from 'vue-virtual-scroller'
import 'vue-virtual-scroller/index.css'
import type { AssetItem, AtlasLayout } from '@/utils/recipe-generator/resources'
import type { SlotValue } from '@/utils/recipe-generator/types'
const RecipeItemIcon = defineAsyncComponent(() => import('./RecipeItemIcon.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import { matchItem } from '@/utils/recipe-generator/itemSearch'

const props = defineProps<{
  items: AssetItem[]
  atlasUrl: string
  atlas: AtlasLayout
}>()

const emit = defineEmits<{
  pick: [value: SlotValue]
}>()

const query = ref('')

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase()
  if (!q) return props.items
  return props.items.filter((item) => matchItem(item, q))
})

/** 每行 4 个，按行做虚拟滚动，避免上千个 DOM 节点导致滑动卡顿 */
const COLUMNS = 4
const ROW_HEIGHT = 64

type PaletteRow = { key: string; items: AssetItem[] }

const rows = computed<PaletteRow[]>(() => {
  const result: PaletteRow[] = []
  for (let i = 0; i < filtered.value.length; i += COLUMNS) {
    result.push({ key: `row-${i}`, items: filtered.value.slice(i, i + COLUMNS) })
  }
  return result
})

function pick(item: AssetItem) {
  emit('pick', { kind: 'item', id: item.id })
}
</script>

<template>
  <div class="item-palette">
    <div class="item-palette-search">
      <input
        v-model="query"
        type="text"
        class="w-full rounded border border-gray-300 px-3 py-1.5 text-sm outline-none transition focus:border-primary-500"
        placeholder="搜索物品（名称 / ID / 中文 / 拼音）…"
      />
    </div>
    <div class="item-palette-count">
      共 {{ filtered.length }} 个物品
    </div>
    <RecycleScroller
      class="item-palette-grid"
      :items="rows"
      :item-size="ROW_HEIGHT"
      key-field="key"
    >
      <template #default="{ item }">
        <div class="item-palette-row">
          <template v-for="entry in item.items" :key="entry.id">
            <Tooltip :text="entry.name" block>
              <button
                type="button"
                class="item-palette-entry"
                @click="pick(entry)"
              >
                <RecipeItemIcon
                  :texture="entry.texture"
                  :atlas-url="atlasUrl"
                  :atlas="atlas"
                  :size="30"
                  :label="entry.zh || entry.name"
                />
                <Tooltip :text="`${entry.name}（${entry.zh}）`" overflowOnly block>
                  <span class="item-palette-name">
                    {{ entry.zh || entry.name }}
                  </span>
                </Tooltip>
              </button>
            </Tooltip>
          </template>
        </div>
      </template>
    </RecycleScroller>
  </div>
</template>

<style scoped>
.item-palette {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  flex: 1;
  min-height: 0;
  padding: 0.75rem;
}

.item-palette-search input {
  width: 100%;
}

.item-palette-count {
  color: #86909c;
  font-size: 0.7rem;
  text-align: right;
}

.item-palette-grid {
  flex: 1;
  min-height: 0;
  padding-right: 2px;
}

.item-palette-row {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0.4rem;
  height: 64px;
}

.item-palette-entry {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.2rem;
  min-width: 0;
  padding: 0.3rem 0.15rem;
  border: 1px solid #e5e6eb;
  border-radius: 4px;
  background: #fff;
  cursor: pointer;
  transition: border-color 0.15s ease, background-color 0.15s ease;
}

.item-palette-entry:hover {
  border-color: var(--color-primary-500);
  background: var(--color-primary-50);
}

.item-palette-name {
  width: 100%;
  overflow: hidden;
  color: #4e5969;
  font-size: 0.65rem;
  line-height: 1.1;
  text-align: center;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
