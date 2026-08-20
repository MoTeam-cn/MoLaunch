<script setup lang="ts">
/**
 * 合成配方生成器 - 配方设置表单（版本/类型/名称/分类/选项）
 */
import { defineAsyncComponent } from 'vue'
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Checkbox = defineAsyncComponent(() => import('@/components/common/Checkbox.vue'))
import {
  DEFAULT_COOKING_TIME,
  isRecipeTypeAvailable,
  supportsRecipeCategory,
  supportsShowNotification,
  supportsSmithingTrimPattern,
} from '@/utils/recipe-generator/versions'
import type { JavaVersionId, RecipeState } from '@/utils/recipe-generator/types'

const props = defineProps<{
  recipe: RecipeState
  selectedVersion: JavaVersionId
  versionOptions: { label: string; value: string }[]
  typeOptions: { label: string; value: string }[]
  categoryOptions: { label: string; value: string }[]
}>()

const emit = defineEmits<{
  'update:selected-version': [version: JavaVersionId]
}>()

function onCookingTimeChange(value: string | number) {
  const str = String(value).trim()
  props.recipe.cooking.time = str === '' ? null : Number(str)
}
</script>

<template>
  <section class="recipe-panel recipe-settings">
    <h3 class="recipe-panel-title">配方设置</h3>
    <div class="recipe-form">
      <label class="recipe-field">
        <span class="recipe-field-label">目标版本</span>
        <Select
          :model-value="selectedVersion"
          :options="versionOptions"
          size="small"
          @update:model-value="emit('update:selected-version', $event as JavaVersionId)"
        />
      </label>
      <label class="recipe-field">
        <span class="recipe-field-label">配方类型</span>
        <Select v-model="recipe.recipeType" :options="typeOptions" size="small" />
      </label>
      <label class="recipe-field">
        <span class="recipe-field-label">配方名称</span>
        <Input v-model="recipe.name" size="small" placeholder="文件名（自动清理非法字符）" />
      </label>
      <label v-if="recipe.recipeType !== 'smithing' && recipe.recipeType !== 'smithing_trim' && recipe.recipeType !== 'smithing_transform'" class="recipe-field">
        <span class="recipe-field-label">分组</span>
        <Input v-model="recipe.group" size="small" placeholder="可空" />
      </label>
      <label v-if="supportsRecipeCategory(selectedVersion, recipe.recipeType) && categoryOptions.length" class="recipe-field">
        <span class="recipe-field-label">分类</span>
        <Select v-model="recipe.category" :options="categoryOptions" size="small" />
      </label>

      <div v-if="recipe.recipeType === 'crafting'" class="recipe-checkbox-group">
        <Checkbox v-model="recipe.crafting.shapeless">无序合成</Checkbox>
        <Checkbox v-model="recipe.crafting.twoByTwo">2×2 网格</Checkbox>
        <Checkbox v-model="recipe.crafting.keepWhitespace">保留空格</Checkbox>
      </div>
      <div v-if="isRecipeTypeAvailable(selectedVersion, recipe.recipeType) && (recipe.recipeType === 'smelting' || recipe.recipeType === 'blasting' || recipe.recipeType === 'smoking' || recipe.recipeType === 'campfire_cooking')" class="recipe-checkbox-group">
        <label class="recipe-field-inline">
          <span>经验</span>
          <Input v-model.number="recipe.cooking.experience" type="number" size="small" min="0" step="0.1" />
        </label>
        <label class="recipe-field-inline">
          <span>时长</span>
          <Input
            :model-value="recipe.cooking.time ?? ''"
            type="number"
            size="small"
            min="1"
            :placeholder="String(DEFAULT_COOKING_TIME[recipe.recipeType])"
            @update:model-value="onCookingTimeChange"
          />
        </label>
      </div>
      <div v-if="recipe.recipeType === 'smithing_trim' && supportsSmithingTrimPattern(selectedVersion)" class="recipe-field">
        <span class="recipe-field-label">纹饰图案</span>
        <Input v-model="recipe.smithing.trimPattern" size="small" placeholder="minecraft:silence_armor_trim_smithing_template" />
      </div>
      <div v-if="supportsShowNotification(selectedVersion, recipe.recipeType, recipe.crafting.shapeless)" class="recipe-checkbox-group">
        <Checkbox v-model="recipe.showNotification">显示完成通知</Checkbox>
      </div>
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

.recipe-panel-title {
  margin: 0;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid #f0f1f3;
  font-size: 0.85rem;
  font-weight: 600;
  color: #1d2129;
}

.recipe-form {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  padding: 0.75rem 1rem 1rem;
}

.recipe-field {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.recipe-field-label {
  color: #4e5969;
  font-size: 0.75rem;
}

.recipe-field-inline {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: #4e5969;
  font-size: 0.75rem;
}

.recipe-checkbox-group {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  font-size: 0.8rem;
}
</style>