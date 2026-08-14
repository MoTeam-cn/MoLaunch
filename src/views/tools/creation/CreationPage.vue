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
  <div class="h-full flex flex-col overflow-hidden">
    <!-- 顶部子菜单：外容器对 creation 分类不设 padding（与设置页 about 页签一致），菜单紧贴左上角 -->
    <SubTabBar v-model="activeSubTab" :tabs="subTabs" sticky />

    <!-- 内容区：recipe 页签占满剩余高度（内部滚动），渐变文字页签保持 p-6 + 自身滚动 -->
    <div class="flex-1 min-h-0 flex flex-col overflow-hidden">
      <div v-if="activeSubTab === 'gradient-text'" id="tool-gradient-text" class="flex-1 min-h-0 overflow-y-auto p-6">
        <GradientTextPage />
      </div>
      <div
        v-else
        id="tool-recipe-generator"
        class="flex-1 min-h-0 flex flex-col overflow-hidden"
        data-toc-card="tool-recipe-generator"
        data-toc-title="合成配方"
      >
        <RecipeGeneratorPage />
      </div>
    </div>
  </div>
</template>
