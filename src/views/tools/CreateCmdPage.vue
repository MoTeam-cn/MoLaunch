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
 *
 * 深链支持：URL `?subtab=recipe-generator` 可直接切到对应子页签。
 */
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import SubTabBar from '@/components/common/SubTabBar.vue'
import { CubeIcon, DocumentTextIcon, GiftIcon, PencilSquareIcon, SparklesIcon } from '@heroicons/vue/24/outline'
import GradientTextPage from './creation/GradientTextPage.vue'
import RecipeGeneratorPage from './creation/recipe-generator/RecipeGeneratorPage.vue'
import ItemEditor from './command/ItemEditor.vue'
import SignShop from './command/SignShop.vue'
import SummonEntity from './command/SummonEntity.vue'

const subTabs = [
  { id: 'gradient-text', label: '渐变文字', icon: PencilSquareIcon },
  { id: 'recipe-generator', label: '合成配方', icon: CubeIcon },
  { id: 'item', label: '物品编辑', icon: GiftIcon },
  { id: 'sign-shop', label: '告示牌商店', icon: DocumentTextIcon },
  { id: 'summon', label: '召唤实体', icon: SparklesIcon },
]
const activeSubTab = ref('gradient-text')

const route = useRoute()

onMounted(() => {
  const subtab = route.query.subtab as string | undefined
  if (subtab && subTabs.some((t) => t.id === subtab)) {
    activeSubTab.value = subtab
  }
})
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
        <SummonEntity v-else />
      </div>
    </div>
  </div>
</template>
