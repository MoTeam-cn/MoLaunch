<script setup lang="ts">
/**
 * 常用工具分类页（归并原「趣味工具」「便捷工具」「计算工具」）
 *
 * 顶部子菜单切换（复用 SubTabBar）：
 * - 今日人品
 * - 便捷工具（清理垃圾/内存优化/协议诊断等）
 * - 坐标距离计算
 * - 游戏内调色板
 * - 版本 JSON 编辑
 *
 * 深链支持：URL `?subtab=palette` 可直接切到对应子页签。
 */
import { onMounted, ref, defineAsyncComponent } from 'vue'
import { useRoute } from 'vue-router'
const SubTabBar = defineAsyncComponent(() => import('@/components/common/SubTabBar.vue'))
import { CodeBracketIcon, FaceSmileIcon, MapPinIcon, SwatchIcon, WrenchScrewdriverIcon } from '@heroicons/vue/24/outline'
const LuckyTool = defineAsyncComponent(() => import('../quick-tools/LuckyTool.vue'))
const QuickTools = defineAsyncComponent(() => import('../QuickTools.vue'))
const CoordCalculator = defineAsyncComponent(() => import('./calc/CoordCalculator.vue'))
const ColorPalette = defineAsyncComponent(() => import('./calc/ColorPalette.vue'))
const VersionJsonEditor = defineAsyncComponent(() => import('./data/VersionJsonEditor.vue'))

const subTabs = [
  { id: 'luck', label: '今日人品', icon: FaceSmileIcon },
  { id: 'quick', label: '便捷工具', icon: WrenchScrewdriverIcon },
  { id: 'coord', label: '坐标计算', icon: MapPinIcon },
  { id: 'palette', label: '调色板', icon: SwatchIcon },
  { id: 'version-json', label: '版本 JSON', icon: CodeBracketIcon },
]
const activeSubTab = ref('luck')

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
        <LuckyTool v-if="activeSubTab === 'luck'" />
        <QuickTools v-else-if="activeSubTab === 'quick'" />
        <CoordCalculator v-else-if="activeSubTab === 'coord'" />
        <ColorPalette v-else-if="activeSubTab === 'palette'" />
        <VersionJsonEditor v-else />
      </div>
    </div>
  </div>
</template>
