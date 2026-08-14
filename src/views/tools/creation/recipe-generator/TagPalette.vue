<script setup lang="ts">
/**
 * 合成配方调色板 - 原版标签：搜索标签，点击放置到槽位
 */
import { computed, ref } from 'vue'
import { RecycleScroller } from 'vue-virtual-scroller'
import 'vue-virtual-scroller/index.css'
import type { SlotValue } from '@/utils/recipe-generator/types'

const props = defineProps<{
  tags: Record<string, string[]>
}>()

const emit = defineEmits<{
  pick: [value: SlotValue]
}>()

const query = ref('')

const entries = computed(() =>
  Object.entries(props.tags)
    .map(([id, values]) => ({ id, count: values.length }))
    .sort((a, b) => a.id.localeCompare(b.id)),
)

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase()
  if (!q) return entries.value
  return entries.value.filter((entry) => entry.id.toLowerCase().includes(q))
})

/** 标签较多，用虚拟滚动只渲染可视区，避免滑动卡顿 */
const ITEM_HEIGHT = 42

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
        placeholder="搜索标签…"
      />
    </div>
    <RecycleScroller
      class="tag-palette-list"
      :items="filtered"
      :item-size="ITEM_HEIGHT"
      key-field="id"
    >
      <template #default="{ item }">
        <button type="button" class="tag-palette-entry" @click="pick(item.id)">
          <span class="tag-palette-name">#{{ item.id }}</span>
          <span class="tag-palette-count">{{ item.count }}</span>
        </button>
      </template>
    </RecycleScroller>
  </div>
</template>

<style scoped>
.tag-palette {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  padding: 0.75rem;
}

.tag-palette-list {
  height: 28rem;
  padding-right: 2px;
}

.tag-palette-entry {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  height: 42px;
  padding: 0.4rem 0.6rem;
  border: 1px solid #e5e6eb;
  border-radius: 4px;
  background: #fff;
  cursor: pointer;
  text-align: left;
  transition: border-color 0.15s ease, background-color 0.15s ease;
}

.tag-palette-entry:hover {
  border-color: #165dff;
  background: #f2f6ff;
}

.tag-palette-name {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  color: #4e5969;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
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
