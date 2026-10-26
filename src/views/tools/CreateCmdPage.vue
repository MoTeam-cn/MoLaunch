<script setup lang="ts">
/**
 * 创作指令分类页（归并原「创作工具」「指令生成」）
 *
 * 顶部子菜单切换（复用 SubTabBar）：
 * - 渐变文字
 * - 合成配方
 * - 物品编辑（/give）
 * - 告示牌商店（/setblock）
 * - 召唤实体（/summon）
 * - 成就生成器（导出 PNG）
 * - 皮肤编辑（像素画布 + 3D 预览）
 *
 * 深链支持：URL `?subtab=recipe-generator` 可直接切到对应子页签（由 SubTabBar 内置持久化恢复）。
 */
import { ref, defineAsyncComponent } from 'vue'
const SubTabBar = defineAsyncComponent(() => import('@/components/common/SubTabBar.vue'))
import {
  CubeIcon,
  FaceSmileIcon,
  DocumentTextIcon,
  GiftIcon,
  PencilSquareIcon,
  SparklesIcon,
  TrophyIcon,
} from '@heroicons/vue/24/outline'
const GradientTextPage = defineAsyncComponent(() => import('./creation/GradientTextPage.vue'))
const RecipeGeneratorPage = defineAsyncComponent(() => import('./creation/recipe-generator/RecipeGeneratorPage.vue'))
const AchievementGenerator = defineAsyncComponent(() => import('./creation/AchievementGenerator.vue'))
const SkinEditorPage = defineAsyncComponent(() => import('./creation/skin-editor/SkinEditorPage.vue'))
const ItemEditor = defineAsyncComponent(() => import('./command/ItemEditor.vue'))
const SignShop = defineAsyncComponent(() => import('./command/SignShop.vue'))
const SummonEntity = defineAsyncComponent(() => import('./command/SummonEntity.vue'))

const subTabs = [
  { id: 'gradient-text', label: '渐变文字', icon: PencilSquareIcon },
  { id: 'recipe-generator', label: '合成配方', icon: CubeIcon },
  { id: 'item', label: '物品编辑', icon: GiftIcon },
  { id: 'sign-shop', label: '告示牌商店', icon: DocumentTextIcon },
  { id: 'summon', label: '召唤实体', icon: SparklesIcon },
  { id: 'achievement', label: '成就生成器', icon: TrophyIcon },
  { id: 'skin-editor', label: '皮肤编辑', icon: FaceSmileIcon },
]
const activeSubTab = ref('gradient-text')
</script>

<template>
  <div>
    <SubTabBar v-model="activeSubTab" :tabs="subTabs" sticky />

    <div class="p-6">
      <div class="mx-auto max-w-3xl">
        <GradientTextPage v-if="activeSubTab === 'gradient-text'" />
        <RecipeGeneratorPage v-else-if="activeSubTab === 'recipe-generator'" />
        <ItemEditor v-else-if="activeSubTab === 'item'" />
        <SignShop v-else-if="activeSubTab === 'sign-shop'" />
        <SummonEntity v-else-if="activeSubTab === 'summon'" />
        <SkinEditorPage v-else-if="activeSubTab === 'skin-editor'" />
        <AchievementGenerator v-else />
      </div>
    </div>
  </div>
</template>
