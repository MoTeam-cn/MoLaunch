<script setup lang="ts">
/**
 * 合成配方生成器：配方编辑 + JSON 预览 + 数据包导出
 *
 * 交互：点击空格子弹出抽屉选择物品/标签填入该格；点击已放置格子可清除；
 * 结果槽滚轮可调整产出数量；顶部切换版本/类型，实时校验并预览配方 JSON。
 */
import { computed, onMounted, reactive, ref, watch, defineAsyncComponent } from 'vue'
const RecipeDisplayPanel = defineAsyncComponent(() => import('./RecipeDisplayPanel.vue'))
const RecipeSettingsForm = defineAsyncComponent(() => import('./RecipeSettingsForm.vue'))
const RecipeSlotDrawer = defineAsyncComponent(() => import('./RecipeSlotDrawer.vue'))
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
  getRecipeCategoryOptions,
  getSupportedRecipeTypes,
  LATEST_JAVA_VERSION,
  RECIPE_CATEGORY_LABELS,
  RECIPE_TYPE_LABELS,
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
import { getRecipeLayout } from './recipe-layouts'
import { createDefaultRecipe, VERSION_OPTIONS } from './recipe-state'

const selectedVersion = ref<JavaVersionId>(LATEST_JAVA_VERSION)
const recipe = reactive<RecipeState>(createDefaultRecipe())

const typeOptions = computed(() =>
  getSupportedRecipeTypes(selectedVersion.value).map((type) => ({
    label: RECIPE_TYPE_LABELS[type],
    value: type,
  })),
)

const categoryOptions = computed(() =>
  (getRecipeCategoryOptions(recipe.recipeType) ?? []).map((value) => ({
    label: RECIPE_CATEGORY_LABELS[value] ?? value,
    value,
  })),
)

const items = ref<AssetItem[]>([])
const tags = ref<Record<string, string[]>>({})
const atlas = ref<AtlasLayout | null>(null)
const atlasUrl = ref('')
// 初始为 true：避免首帧渲染时 atlas 仍为 null，把空对象传给子组件触发 prop 校验警告
const loading = ref(true)

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

const gridSlots = computed<RecipeSlot[]>(() =>
  recipe.recipeType === 'crafting' ? [...CRAFTING_GRID_SLOTS] : [],
)
const inputSlots = computed<RecipeSlot[]>(() => getInputSlots(recipe))
/** 当前可编辑的输入槽位（crafting 按 2x2/3x3 裁剪，与网格显示一致） */
const editableSlots = computed<RecipeSlot[]>(() => {
  if (recipe.recipeType !== 'crafting') return inputSlots.value
  const size = recipe.crafting.twoByTwo ? 2 : 3
  return gridSlots.value.slice(0, size * size)
})
const resultSlot = computed<RecipeSlot | undefined>(() => getResultSlots(recipe)[0])
const recipeLayout = computed(() => getRecipeLayout(recipe.recipeType))
const recipeName = computed(() => (recipe.name.trim() ? recipe.name.trim() : 'recipe'))

const editingSlot = ref<RecipeSlot | null>(null)
const drawerVisible = ref(false)

watch(drawerVisible, (visible) => {
  if (!visible) editingSlot.value = null
})

function openSlotDrawer(slot: RecipeSlot) {
  editingSlot.value = slot
  drawerVisible.value = true
}

function pickValue(value: SlotValue) {
  const slot = editingSlot.value
  if (!slot) return
  recipe.slots[slot] = value
  if (slot === resultSlot.value) {
    drawerVisible.value = false
    return
  }
  // 连续放置：填完不关闭抽屉，自动定位下一个空格子；全部填满才关闭
  const slots = editableSlots.value
  const next =
    slots.slice(slots.indexOf(slot) + 1).find((s) => !recipe.slots[s]) ??
    slots.find((s) => !recipe.slots[s])
  if (next) editingSlot.value = next
  else drawerVisible.value = false
}

function updateSlot(slot: RecipeSlot, value: SlotValue | undefined) {
  if (value) recipe.slots[slot] = value
  else delete recipe.slots[slot]
}

function updateCount(slot: RecipeSlot, count: number) {
  const value = recipe.slots[slot]
  if (value && (value.kind === 'item' || value.kind === 'custom_item')) value.count = count
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
        <span class="text-xs text-gray-400">合成 · 熔炼 · 切石 · 锻造，全版本离线生成数据包</span>
      </div>
    </div>

    <div v-if="loading" class="text-center py-10 text-sm text-gray-400">正在加载版本资源…</div>
    <div v-else class="recipe-generator-body">
      <!-- 左：展示区（槽位编辑 + 校验 + JSON 预览） -->
      <RecipeDisplayPanel
        :layout="recipeLayout!"
        :values="recipe.slots"
        :context="context"
        :atlas-url="atlasUrl"
        :atlas="atlas!"
        :two-by-two="recipe.crafting.twoByTwo && recipe.recipeType === 'crafting'"
        :editing-slot="editingSlot"
        :issues="issues"
        :json-text="recipeJsonText"
        :json="recipeJson"
        :valid="isValid"
        :exporting="exporting"
        @update-slot="updateSlot"
        @update-count="updateCount"
        @edit-slot="openSlotDrawer"
        @copy="copyJson"
        @export="exportPack"
      />

      <!-- 右：功能区（配方设置） -->
      <div class="recipe-functions">
        <RecipeSettingsForm
          :recipe="recipe"
          :selected-version="selectedVersion"
          :version-options="VERSION_OPTIONS"
          :type-options="typeOptions"
          :category-options="categoryOptions"
          @update:selected-version="selectedVersion = $event"
        />
      </div>
    </div>

    <!-- 贴图选择抽屉：点击空格子展开 -->
    <RecipeSlotDrawer
      v-model:visible="drawerVisible"
      :editing-slot="editingSlot"
      :slots="editableSlots"
      :recipe-type="recipe.recipeType"
      :two-by-two="recipe.crafting.twoByTwo"
      :items="items"
      :tags="tags"
      :atlas="atlas!"
      :atlas-url="atlasUrl"
      @move-to="editingSlot = $event"
      @pick="pickValue"
    />
  </div>
</template>

<style scoped>
.recipe-generator-page {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 1rem;
}

.recipe-generator-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.recipe-generator-body {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 320px;
  gap: 1rem;
  align-items: start;
}

@media (max-width: 960px) {
  .recipe-generator-body {
    grid-template-columns: 1fr;
  }
}

.recipe-functions {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
</style>