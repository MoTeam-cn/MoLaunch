<script setup lang="ts">
/**
 * 合成配方调色板 - 原版标签：按中文名/ID 搜索，每行多个标签，点击放置到槽位
 */
import { computed, ref } from 'vue'
import { RecycleScroller } from 'vue-virtual-scroller'
import 'vue-virtual-scroller/index.css'
import type { SlotValue } from '@/utils/recipe-generator/types'
import { tagLabel } from '@/utils/recipe-generator/tag-zh'

const props = defineProps<{
  tags: Record<string, string[]>
}>()

const emit = defineEmits<{
  pick: [value: SlotValue]
}>()

const query = ref('')

type TagEntry = { id: string; count: number }

const entries = computed<TagEntry[]>(() =>
  Object.entries(props.tags)
    .map(([id, values]) => ({ id, count: values.length }))
    .sort((a, b) => a.id.localeCompare(b.id)),
)

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase()
  if (!q) return entries.value
  return entries.value.filter(
    (entry) =>
      entry.id.toLowerCase().includes(q) || tagLabel(entry.id).toLowerCase().includes(q),
  )
})

/** 标签较多，每行 2 个按行虚拟滚动，避免滑动卡顿 */
const COLUMNS = 2
const ROW_HEIGHT = 40

type TagRow = { key: string; tags: TagEntry[] }

const rows = computed<TagRow[]>(() => {
  const result: TagRow[] = []
  for (let i = 0; i < filtered.value.length; i += COLUMNS) {
    result.push({ key: `row-${i}`, tags: filtered.value.slice(i, i + COLUMNS) })
  }
  return result
})

function pick(id: string) {
  emit('pick', { kind: 'vanilla_tag', id })
}
</script>

<template>
  <div class="tag-palette">
    <div>
      <input
        v-model="query"
        type="text"
        class="w-full rounded border border-gray-300 px-3 py-1.5 text-sm outline-none transition focus:border-primary-500"
        placeholder="搜索标签（中文 / ID）…"
      />
    </div>
    <div class="tag-palette-summary">共 {{ filtered.length }} 个标签</div>
    <RecycleScroller
      class="tag-palette-list"
      :items="rows"
      :item-size="ROW_HEIGHT"
      key-field="key"
    >
      <template #default="{ item }">
        <div class="tag-palette-row">
          <button
            v-for="entry in item.tags"
            :key="entry.id"
            type="button"
            class="tag-palette-entry"
            :title="`#${entry.id} · ${entry.count} 个物品`"
            @click="pick(entry.id)"
          >
            <span class="tag-palette-name">#{{ tagLabel(entry.id) }}</span>
            <span class="tag-palette-count">{{ entry.count }}</span>
          </button>
        </div>
      </template>
    </RecycleScroller>
  </div>
</template>

<style scoped>
.tag-palette {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  flex: 1;
  min-height: 0;
  padding: 0.75rem;
}

.tag-palette-summary {
  color: #86909c;
  font-size: 0.7rem;
  text-align: right;
}

.tag-palette-list {
  flex: 1;
  min-height: 0;
  padding-right: 2px;
}

.tag-palette-row {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.4rem;
  height: 40px;
}

.tag-palette-entry {
  display: flex;
  align-items: center;
  gap: 0.3rem;
  min-width: 0;
  padding: 0 0.5rem;
  border: 1px solid #e5e6eb;
  border-radius: 4px;
  background: #fff;
  cursor: pointer;
  transition: border-color 0.15s ease, background-color 0.15s ease;
}

.tag-palette-entry:hover {
  border-color: var(--color-primary-500);
  background: var(--color-primary-50);
}

.tag-palette-name {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  color: #4e5969;
  font-size: 0.72rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tag-palette-count {
  flex: none;
  padding: 0 6px;
  border-radius: 999px;
  background: #f2f3f5;
  color: #86909c;
  font-size: 0.65rem;
  line-height: 1.4rem;
}
</style>
