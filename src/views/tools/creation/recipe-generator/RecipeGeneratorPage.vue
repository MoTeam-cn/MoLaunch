<script setup lang="ts">
/**
 * 合成配方生成器：配方编辑 + JSON 预览 + 数据包导出
 *
 * 交互：从右侧调色板点选物品自动填入第一个空格；点击已放置格子可清除；
 * 结果槽滚轮可调整产出数量；顶部切换版本/类型，实时校验并预览配方 JSON。
 */
import { computed, onMounted, reactive, ref, watch } from 'vue'
import Button from '@/components/common/Button.vue'
import Select from '@/components/common/Select.vue'
import Input from '@/components/common/Input.vue'
import Checkbox from '@/components/common/Checkbox.vue'
import { toastError, toastInfo, toastSuccess } from '@/utils/toast'
import {
  buildSlotContext,
  getAtlasLayout,
  getAtlasPngUrl,
  loadVersionItems,
  loadVersionTags,
  type AssetItem,
  type AtlasLayout,
} from '@/utils/recipe-generator/resources'
import {
  coerceRecipeTypeForVersion,
  DEFAULT_COOKING_TIME,
  getRecipeCategoryOptions,
  getSupportedRecipeTypes,
  isRecipeTypeAvailable,
  LATEST_JAVA_VERSION,
  supportsRecipeCategory,
  supportsShowNotification,
  supportsSmithingTrimPattern,
} from '@/utils/recipe-generator/versions'
import {
  CRAFTING_GRID_SLOTS,
  generateRecipeJson,
} from '@/utils/recipe-generator/recipe-engine'
import {
  getInputSlots,
  getResultSlots,
} from '@/utils/recipe-generator/formatter'
import { validateRecipe } from '@/utils/recipe-generator/validation'
import { exportRecipePack } from '@/utils/recipe-generator/exporter'
import { sanitizeRecipeName } from '@/utils/recipe-generator/datapack'
import type { JavaVersionId, RecipeSlot, RecipeSlotContext, RecipeState, SlotValue } from '@/utils/recipe-generator/types'
import RecipeSlotsEditor from './RecipeSlotsEditor.vue'
import ItemPalette from './ItemPalette.vue'
import TagPalette from './TagPalette.vue'

let uidCounter = 0
function createUid(prefix: string): string {
  uidCounter += 1
  return `${prefix}-${Date.now().toString(36)}-${uidCounter}`
}

function createDefaultRecipe(): RecipeState {
  return {
    id: createUid('recipe'),
    recipeType: 'crafting',
    group: '',
    category: 'misc',
    showNotification: true,
    nameMode: 'manual',
    name: 'my_recipe',
    slots: {},
    crafting: { shapeless: false, keepWhitespace: false, twoByTwo: false },
    cooking: { time: null, experience: 0 },
    smithing: { trimPattern: '' },
  }
}

const selectedVersion = ref<JavaVersionId>(LATEST_JAVA_VERSION)
const recipe = reactive<RecipeState>(createDefaultRecipe())

const versionOptions = [
  { label: '1.12', value: '1.12' },
  { label: '1.13', value: '1.13' },
  { label: '1.14', value: '1.14' },
  { label: '1.15', value: '1.15' },
  { label: '1.16', value: '1.16' },
  { label: '1.17', value: '1.17' },
  { label: '1.18', value: '1.18' },
  { label: '1.19', value: '1.19' },
  { label: '1.20', value: '1.20' },
  { label: '1.21', value: '1.21' },
  { label: '1.21.2', value: '1.21.2' },
  { label: '1.21.4', value: '1.21.4' },
  { label: '1.21.5', value: '1.21.5' },
  { label: '1.21.6', value: '1.21.6' },
  { label: '1.21.7', value: '1.21.7' },
  { label: '1.21.9', value: '1.21.9' },
  { label: '1.21.11', value: '1.21.11' },
  { label: '26.1', value: '26.1' },
  { label: '26.2', value: '26.2' },
]

const typeOptions = computed(() =>
  getSupportedRecipeTypes(selectedVersion.value).map((type) => ({
    label: type,
    value: type,
  })),
)

const items = ref<AssetItem[]>([])
const tags = ref<Record<string, string[]>>({})
const atlas = ref<AtlasLayout | null>(null)
const atlasUrl = ref('')
const loading = ref(false)
const activeTab = ref<'items' | 'tags'>('items')

const context = computed<RecipeSlotContext>(() => buildSlotContext(items.value, tags.value))

async function loadResources(version: JavaVersionId) {
  loading.value = true
  try {
    const [loadedItems, loadedTags, loadedAtlas, pngUrl] = await Promise.all([
      loadVersionItems(version),
      loadVersionTags(version),
      getAtlasLayout(),
      Promise.resolve(getAtlasPngUrl()),
    ])
    items.value = loadedItems
    tags.value = loadedTags
    atlas.value = loadedAtlas
    atlasUrl.value = pngUrl
  } finally {
    loading.value = false
  }
}

onMounted(() => loadResources(selectedVersion.value))

watch(selectedVersion, (version) => {
  recipe.recipeType = coerceRecipeTypeForVersion(recipe.recipeType, version)
  loadResources(version)
})

const issues = computed(() => validateRecipe(recipe, selectedVersion.value, context.value))
const isValid = computed(() => issues.value.length === 0)

const recipeJson = computed(() => {
  if (!isValid.value) return null
  try {
    return generateRecipeJson(recipe, selectedVersion.value, context.value)
  } catch {
    return null
  }
})

const recipeJsonText = computed(() => (recipeJson.value ? JSON.stringify(recipeJson.value, null, 2) : ''))

const gridSlots = computed<RecipeSlot[]>(() => [...CRAFTING_GRID_SLOTS])
const inputSlots = computed<RecipeSlot[]>(() => getInputSlots(recipe))
const resultSlot = computed<RecipeSlot | undefined>(() => getResultSlots(recipe)[0])
const recipeName = computed(() => (recipe.name.trim() ? recipe.name.trim() : 'recipe'))

function pickValue(value: SlotValue) {
  const target = inputSlots.value.find((slot) => !recipe.slots[slot])
  if (target) recipe.slots[target] = value
  else toastInfo('合成格已填满，请先清除一些格子')
}

function updateSlot(slot: RecipeSlot, value: SlotValue | undefined) {
  if (value) recipe.slots[slot] = value
  else delete recipe.slots[slot]
}

function updateCount(slot: RecipeSlot, count: number) {
  const value = recipe.slots[slot]
  if (value && (value.kind === 'item' || value.kind === 'custom_item')) value.count = count
}

function onCookingTimeChange(value: string | number) {
  const str = String(value).trim()
  recipe.cooking.time = str === '' ? null : Number(str)
}

async function copyJson() {
  if (!recipeJsonText.value) return
  try {
    await navigator.clipboard.writeText(recipeJsonText.value)
    toastSuccess('配方 JSON 已复制到剪贴板')
  } catch {
    toastError('复制失败，请手动选择复制')
  }
}

const exporting = ref(false)
async function exportPack() {
  if (!isValid.value || !recipeJson.value) {
    toastInfo('请先修复校验问题')
    return
  }
  exporting.value = true
  try {
    const result = await exportRecipePack({
      version: selectedVersion.value,
      recipes: [{ name: recipeName.value, json: recipeJson.value }],
      tags: [],
      defaultFileName: sanitizeRecipeName(recipeName.value, 'recipe'),
    })
    if (result.ok) toastSuccess(`已导出到 ${result.path}`)
    else if (result.reason === 'cancelled') toastInfo('已取消导出')
    else if (result.reason === 'unsupported') toastInfo('该版本不支持数据包，已改为单配方 JSON')
    else toastError(`导出失败：${result.reason}`)
  } finally {
    exporting.value = false
  }
}
</script>

<template>
  <div class="recipe-generator-page">
    <div class="recipe-generator-header">
      <div class="flex items-center gap-2">
        <span class="text-base font-semibold text-gray-800">合成配方生成器</span>
        <span class="text-xs text-gray-400">Shaped / Shapeless / Cooking / Stonecutter / Smithing</span>
      </div>
    </div>

    <div v-if="loading" class="text-center py-10 text-sm text-gray-400">正在加载版本资源…</div>
    <div v-else class="recipe-generator-body">
      <!-- 左：设置 -->
      <section class="recipe-panel recipe-settings">
        <h3 class="recipe-panel-title">配方设置</h3>
        <div class="recipe-form">
          <label class="recipe-field">
            <span class="recipe-field-label">目标版本</span>
            <Select v-model="selectedVersion" :options="versionOptions" size="small" />
          </label>
          <label class="recipe-field">
            <span class="recipe-field-label">配方类型</span>
            <Select v-model="recipe.recipeType" :options="typeOptions" size="small" />
          </label>
          <label class="recipe-field">
            <span class="recipe-field-label">配方名称</span>
            <Input v-model="recipe.name" size="small" placeholder="export_name" />
          </label>
          <label v-if="recipe.recipeType !== 'smithing' && recipe.recipeType !== 'smithing_trim' && recipe.recipeType !== 'smithing_transform'" class="recipe-field">
            <span class="recipe-field-label">分组 group</span>
            <Input v-model="recipe.group" size="small" placeholder="可空" />
          </label>
          <label v-if="supportsRecipeCategory(selectedVersion, recipe.recipeType) && getRecipeCategoryOptions(recipe.recipeType)" class="recipe-field">
            <span class="recipe-field-label">分类 category</span>
            <Select
              v-model="recipe.category"
              :options="getRecipeCategoryOptions(recipe.recipeType)!.map((value) => ({ label: value, value }))"
              size="small"
            />
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

      <!-- 中：编辑 + 预览 -->
      <section class="recipe-panel recipe-editor">
        <RecipeSlotsEditor
          :slots="inputSlots"
          :grid-slots="gridSlots"
          :result-slot="resultSlot"
          :values="recipe.slots"
          :context="context"
          :atlas-url="atlasUrl"
          :atlas="atlas!"
          :two-by-two="recipe.crafting.twoByTwo && recipe.recipeType === 'crafting'"
          @update-slot="updateSlot"
          @update-count="updateCount"
        />
        <p class="text-xs text-gray-400">
          从右侧选择物品自动填入第一个空格，点击格子可清除，结果槽滚轮调整数量
        </p>

        <div v-if="issues.length" class="recipe-issues">
          <div v-for="item in issues" :key="item.code" class="recipe-issue">{{ item.message }}</div>
        </div>

        <div class="recipe-preview">
          <div class="recipe-preview-toolbar">
            <span class="text-sm font-medium text-gray-700">配方 JSON</span>
            <div class="flex gap-2">
              <Button size="small" :disabled="!recipeJson" @click="copyJson">复制</Button>
              <Button type="primary" size="small" :disabled="!isValid" :loading="exporting" @click="exportPack">
                导出数据包
              </Button>
            </div>
          </div>
          <pre v-if="recipeJsonText" class="recipe-preview-code">{{ recipeJsonText }}</pre>
          <div v-else class="recipe-preview-empty">填写配方后此处显示生成的 JSON</div>
        </div>
      </section>

      <!-- 右：调色板 -->
      <aside class="recipe-panel recipe-palette">
        <div class="recipe-palette-tabs">
          <span
            class="recipe-palette-tab"
            :class="{ active: activeTab === 'items' }"
            @click="activeTab = 'items'"
          >物品</span>
          <span
            class="recipe-palette-tab"
            :class="{ active: activeTab === 'tags' }"
            @click="activeTab = 'tags'"
          >标签</span>
        </div>
        <ItemPalette
          v-if="activeTab === 'items'"
          :items="items"
          :atlas-url="atlasUrl"
          :atlas="atlas!"
          @pick="pickValue"
        />
        <TagPalette v-else :tags="tags" @pick="pickValue" />
      </aside>
    </div>
  </div>
</template>

<style scoped>
.recipe-generator-page {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem;
  min-height: 100%;
}

.recipe-generator-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.recipe-generator-body {
  display: grid;
  grid-template-columns: 280px minmax(0, 1fr) 320px;
  gap: 1rem;
  align-items: start;
}

@media (max-width: 1280px) {
  .recipe-generator-body {
    grid-template-columns: 1fr;
  }
}

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

.recipe-editor {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 1rem;
}

.recipe-issues {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  padding: 0.5rem 0.75rem;
  border: 1px solid #ffd8cc;
  border-radius: 6px;
  background: #fff7e8;
}

.recipe-issue {
  color: #e8590c;
  font-size: 0.75rem;
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

.recipe-palette-tabs {
  display: flex;
  border-bottom: 1px solid #f0f1f3;
}

.recipe-palette-tab {
  flex: 1;
  padding: 0.6rem 0;
  color: #86909c;
  font-size: 0.8rem;
  text-align: center;
  cursor: pointer;
}

.recipe-palette-tab.active {
  color: #165dff;
  font-weight: 600;
  border-bottom: 2px solid #165dff;
}
</style>
