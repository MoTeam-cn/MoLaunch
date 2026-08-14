<script setup lang="ts">
/**
 * 计算工具分类页
 *
 * 顶部子菜单切换（复用 SubTabBar），承载该分类下所有工具：
 * - 坐标距离计算
 * - 游戏内调色板
 *
 * 深链支持：URL `?subtab=palette` 可直接切到对应子页签。
 */
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import SubTabBar from '@/components/common/SubTabBar.vue'
import { MapPinIcon, SwatchIcon } from '@heroicons/vue/24/outline'
import CoordCalculator from '@/views/tools/calc/CoordCalculator.vue'
import ColorPalette from '@/views/tools/calc/ColorPalette.vue'

const subTabs = [
  { id: 'coord', label: '坐标计算', icon: MapPinIcon },
  { id: 'palette', label: '调色板', icon: SwatchIcon },
]
const activeSubTab = ref('coord')

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
        <CoordCalculator v-if="activeSubTab === 'coord'" />
        <ColorPalette v-else />
      </div>
    </div>
  </div>
</template>
