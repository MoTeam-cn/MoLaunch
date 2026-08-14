<script setup lang="ts">
/**
 * 诊断工具分类页
 *
 * 顶部子菜单切换（复用 SubTabBar），承载该分类下所有工具：
 * - 版本 JSON 编辑
 * - NBT 数据查看
 *
 * 注：崩溃日志分析已迁移至「实验性」页面的日志分析分类（仅实验性页面可用）。
 * 深链支持：URL `?subtab=nbt` 可直接切到对应子页签。
 */
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import SubTabBar from '@/components/common/SubTabBar.vue'
import { CodeBracketIcon, TableCellsIcon } from '@heroicons/vue/24/outline'
import VersionJsonEditor from '@/views/tools/data/VersionJsonEditor.vue'
import NbtViewer from '@/views/tools/data/NbtViewer.vue'

const subTabs = [
  { id: 'version-json', label: '版本 JSON', icon: CodeBracketIcon },
  { id: 'nbt', label: 'NBT 查看', icon: TableCellsIcon },
]
const activeSubTab = ref('version-json')

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
        <VersionJsonEditor v-if="activeSubTab === 'version-json'" />
        <NbtViewer v-else />
      </div>
    </div>
  </div>
</template>
