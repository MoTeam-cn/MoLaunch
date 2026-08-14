<script setup lang="ts">
/**
 * 指令生成工具分类页
 *
 * 顶部子菜单切换（复用 SubTabBar），纯前端生成 Minecraft 指令：
 * - 物品编辑（/give）：物品、数量、自定义名称/Lore、附魔
 * - 告示牌商店（/setblock）：放置带文本的告示牌
 * - 召唤实体（/summon）：指定实体/坐标/名称
 *
 * 深链支持：URL `?subtab=sign-shop` 可直接切到对应子页签。
 */
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import SubTabBar from '@/components/common/SubTabBar.vue'
import { CubeIcon, DocumentTextIcon, SparklesIcon } from '@heroicons/vue/24/outline'
import ItemEditor from './ItemEditor.vue'
import SignShop from './SignShop.vue'
import SummonEntity from './SummonEntity.vue'

const subTabs = [
  { id: 'item', label: '物品编辑', icon: CubeIcon },
  { id: 'sign-shop', label: '告示牌商店', icon: DocumentTextIcon },
  { id: 'summon', label: '召唤实体', icon: SparklesIcon },
]
const activeSubTab = ref('item')

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
    <!-- 顶部子菜单：外容器不设 padding（与创建分类一致），菜单紧贴左上角，滚动时吸顶 -->
    <SubTabBar v-model="activeSubTab" :tabs="subTabs" sticky />

    <div class="p-6">
      <ItemEditor v-if="activeSubTab === 'item'" />
      <SignShop v-else-if="activeSubTab === 'sign-shop'" />
      <SummonEntity v-else />
    </div>
  </div>
</template>
