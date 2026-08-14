<script setup lang="ts">
/**
 * 存档管理分类页
 *
 * 顶部子菜单切换（复用 SubTabBar），承载该分类下所有工具：
 * - 存档管理（备份/恢复/导出）
 *
 * 深链支持：URL `?subtab=archive` 可直接切到对应子页签。
 */
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import SubTabBar from '@/components/common/SubTabBar.vue'
import { FolderIcon } from '@heroicons/vue/24/outline'
import ArchiveManager from '@/views/tools/archive/ArchiveManager.vue'

const subTabs = [{ id: 'archive', label: '存档管理', icon: FolderIcon }]
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
      </div>
    </div>
  </div>
</template>
