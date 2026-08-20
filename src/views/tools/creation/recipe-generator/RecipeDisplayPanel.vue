<script setup lang="ts">
/**
 * 合成配方生成器 - 展示区：槽位编辑 + 校验提示 + JSON 预览
 */
import { defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Alert = defineAsyncComponent(() => import('@/components/common/Alert.vue'))
const RecipeSlotsEditor = defineAsyncComponent(() => import('./RecipeSlotsEditor.vue'))
import type { RecipeIssue } from '@/utils/recipe-generator/validation'
import type { RecipeSlot, RecipeSlotContext, SlotValue } from '@/utils/recipe-generator/types'
import type { AtlasLayout } from '@/utils/recipe-generator/resources'
import type { RecipeLayout } from './recipe-layouts'

defineProps<{
  layout: RecipeLayout
  values: Partial<Record<RecipeSlot, SlotValue>>
  context: RecipeSlotContext
  atlasUrl: string
  atlas: AtlasLayout
  twoByTwo: boolean
  editingSlot: RecipeSlot | null
  issues: RecipeIssue[]
  jsonText: string
  json: object | null
  valid: boolean
  exporting: boolean
}>()

const emit = defineEmits<{
  'update-slot': [slot: RecipeSlot, value: SlotValue | undefined]
  'update-count': [slot: RecipeSlot, count: number]
  'edit-slot': [slot: RecipeSlot]
  copy: []
  export: []
}>()
</script>

<template>
  <section class="recipe-panel recipe-display">
    <RecipeSlotsEditor
      :layout="layout"
      :values="values"
      :context="context"
      :atlas-url="atlasUrl"
      :atlas="atlas"
      :two-by-two="twoByTwo"
      :editing-slot="editingSlot"
      @update-slot="(slot, value) => emit('update-slot', slot, value)"
      @update-count="(slot, count) => emit('update-count', slot, count)"
      @edit-slot="(slot) => emit('edit-slot', slot)"
    />
    <p class="text-xs text-gray-400">
      点击空格子从抽屉选择物品/标签，点击已放置格子可清除，滚轮调整数量
    </p>

    <div v-if="issues.length" class="recipe-issues">
      <Alert
        v-for="item in issues"
        :key="item.code"
        type="warning"
        :message="item.message"
        :truncate="false"
      />
    </div>

    <div class="recipe-preview">
      <div class="recipe-preview-toolbar">
        <span class="text-sm font-medium text-gray-700">配方 JSON</span>
        <div class="flex gap-2">
          <Button size="small" :disabled="!json" @click="emit('copy')">复制</Button>
          <Button type="primary" size="small" :disabled="!valid" :loading="exporting" @click="emit('export')">
            导出数据包
          </Button>
        </div>
      </div>
      <pre v-if="jsonText" class="recipe-preview-code">{{ jsonText }}</pre>
      <div v-else class="recipe-preview-empty">填写配方后此处显示生成的 JSON</div>
    </div>
  </section>
</template>

<style scoped>
.recipe-panel {
  border: 1px solid #e5e6eb;
  border-radius: 8px;
  background: #fff;
  overflow: hidden;
}

.recipe-display {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 1rem;
}

.recipe-issues {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.recipe-preview {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.recipe-preview-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.recipe-preview-code {
  max-height: 20rem;
  margin: 0;
  padding: 0.75rem;
  overflow: auto;
  border: 1px solid #e5e6eb;
  border-radius: 6px;
  background: #f7f8fa;
  color: #4e5969;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 0.72rem;
  line-height: 1.5;
  white-space: pre;
}

.recipe-preview-empty {
  padding: 2rem 0;
  border: 1px dashed #e5e6eb;
  border-radius: 6px;
  color: #86909c;
  font-size: 0.75rem;
  text-align: center;
}
</style>