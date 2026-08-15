<script setup lang="ts">
/**
 * 存档资源分类页（归并原「存档管理」「游戏资源」「种子地图」）
 *
 * 顶部子菜单切换（复用 SubTabBar）：
 * - 存档管理（备份/恢复/导出）
 * - NBT 编辑器（level.dat / playerdata / region .mca）
 * - 截图批量管理
 * - 资源包转换
 * - 种子地图
 *
 * 深链支持：URL `?subtab=screenshot` 可直接切到对应子页签。
 */
import { onMounted, ref, defineAsyncComponent } from 'vue'
import { useRoute } from 'vue-router'
const SubTabBar = defineAsyncComponent(() => import('@/components/common/SubTabBar.vue'))
import { CameraIcon, FolderIcon, MapIcon, PhotoIcon, TableCellsIcon } from '@heroicons/vue/24/outline'
const ArchiveManager = defineAsyncComponent(() => import('./archive/ArchiveManager.vue'))
const NbtViewer = defineAsyncComponent(() => import('./data/NbtViewer.vue'))
const ScreenshotManager = defineAsyncComponent(() => import('./data/ScreenshotManager.vue'))
const ResourcePackConverter = defineAsyncComponent(() => import('./data/ResourcePackConverter.vue'))
const SeedMap = defineAsyncComponent(() => import('./data/SeedMap.vue'))

const subTabs = [
  { id: 'archive', label: '存档管理', icon: FolderIcon },
  { id: 'nbt', label: 'NBT 编辑器', icon: TableCellsIcon },
  { id: 'screenshot', label: '截图管理', icon: CameraIcon },
  { id: 'resourcepack', label: '资源包转换', icon: PhotoIcon },
  { id: 'seedmap', label: '种子地图', icon: MapIcon },
]
const activeSubTab = ref('archive')

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
        <ArchiveManager v-if="activeSubTab === 'archive'" />
        <NbtViewer v-else-if="activeSubTab === 'nbt'" />
        <ScreenshotManager v-else-if="activeSubTab === 'screenshot'" />
        <ResourcePackConverter v-else-if="activeSubTab === 'resourcepack'" />
        <SeedMap v-else />
      </div>
    </div>
  </div>
</template>
