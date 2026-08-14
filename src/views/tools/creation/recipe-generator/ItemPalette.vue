<script setup lang="ts">
/**
 * 合成配方调色板：按名称/ID 搜索物品，点击放置到槽位
 */
import { computed, ref } from 'vue'
import type { AssetItem, AtlasLayout } from '@/utils/recipe-generator/resources'
import type { SlotValue } from '@/utils/recipe-generator/types'
import RecipeItemIcon from './RecipeItemIcon.vue'

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
  return props.items.filter(
    (item) =>
      item.id.toLowerCase().includes(q) ||
      item.name.toLowerCase().includes(q) ||
      item.zh.toLowerCase().includes(q),
  )
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
        placeholder="搜索物品（名称 / ID / 中文）…"
      />
    </div>
    <div class="item-palette-count">
      共 {{ filtered.length }} 个物品
    </div>
    <div class="item-palette-grid">
      <button
        v-for="item in filtered"
        :key="item.id"
        type="button"
        class="item-palette-entry"
        :title="item.name"
        @click="pick(item)"
      >
        <RecipeItemIcon
          :texture="item.texture"
          :atlas-url="atlasUrl"
          :atlas="atlas"
          :size="30"
          :label="item.name"
        />
        <span class="item-palette-name" :title="`${item.name}（${item.zh}）`">
          {{ item.name }}
        </span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.item-palette {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
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
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0.4rem;
  overflow-y: auto;
  max-height: 28rem;
  padding-right: 2px;
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
  border-color: #165dff;
  background: #f2f6ff;
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
