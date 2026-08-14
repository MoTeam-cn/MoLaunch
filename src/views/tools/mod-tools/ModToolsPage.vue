<script setup lang="ts">
/**
 * Mod 工具分类页
 *
 * 顶部子菜单切换（复用 SubTabBar），承载该分类下所有工具：
 * - Mod 依赖检测
 * - Mod 文件去重
 *
 * 深链支持：URL `?subtab=dedup` 可直接切到对应子页签。
 */
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import SubTabBar from '@/components/common/SubTabBar.vue'
import { ScissorsIcon, ShieldCheckIcon } from '@heroicons/vue/24/outline'
import ModDependencyChecker from '@/views/tools/mod-tools/ModDependencyChecker.vue'
import ModDedupScanner from '@/views/tools/mod-tools/ModDedupScanner.vue'

const subTabs = [
  { id: 'dependency', label: '依赖检测', icon: ShieldCheckIcon },
  { id: 'dedup', label: 'Mod 去重', icon: ScissorsIcon },
]
const activeSubTab = ref('dependency')

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
        <ModDependencyChecker v-if="activeSubTab === 'dependency'" />
        <ModDedupScanner v-else />
      </div>
    </div>
  </div>
</template>
