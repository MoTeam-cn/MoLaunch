<script setup lang="ts">
/**
 * 种子地图分类页
 *
 * 种子地图是高频实用工具，单独一栏便于快速访问。
 * 顶部子菜单切换（复用 SubTabBar）承载该分类下工具。
 *
 * 深链支持：URL `?subtab=seedmap` 可直接切到对应子页签。
 */
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import SubTabBar from '@/components/common/SubTabBar.vue'
import { MapIcon } from '@heroicons/vue/24/outline'
import SeedMap from '@/views/tools/data/SeedMap.vue'

const subTabs = [{ id: 'seedmap', label: '种子地图', icon: MapIcon }]
const activeSubTab = ref('seedmap')

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
        <SeedMap v-if="activeSubTab === 'seedmap'" />
      </div>
    </div>
  </div>
</template>
