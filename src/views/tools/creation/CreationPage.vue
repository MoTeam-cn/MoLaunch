<script setup lang="ts">
/**
 * 创作工具分类页
 *
 * 顶部子菜单切换（复用设置页 SubTabBar），避免多个工具叠加过长：
 * - 渐变文字生成器
 * - 合成配方生成器
 *
 * 深链支持：URL `?subtab=recipe-generator` 可直接切到「合成配方」子页签。
 */
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import SubTabBar from '@/components/common/SubTabBar.vue'
import { CubeIcon, PencilSquareIcon } from '@heroicons/vue/24/outline'
import GradientTextPage from './GradientTextPage.vue'
import RecipeGeneratorPage from './recipe-generator/RecipeGeneratorPage.vue'

const subTabs = [
  { id: 'gradient-text', label: '渐变文字', icon: PencilSquareIcon },
  { id: 'recipe-generator', label: '合成配方', icon: CubeIcon },
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
  <div class="space-y-4">
    <!-- 顶部子菜单（sticky 固定，滚动时吸顶） -->
    <SubTabBar v-model="activeSubTab" :tabs="subTabs" sticky />

    <!-- 内容区：按当前子页签渲染（渐变文字根节点自带 data-toc-card） -->
    <div v-if="activeSubTab === 'gradient-text'" id="tool-gradient-text">
      <GradientTextPage />
    </div>
    <div
      v-else
      id="tool-recipe-generator"
      data-toc-card="tool-recipe-generator"
      data-toc-title="合成配方"
    >
      <RecipeGeneratorPage />
    </div>
  </div>
</template>
