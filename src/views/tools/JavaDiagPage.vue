<script setup lang="ts">
/**
 * Java 诊断分类页（归并原「Java 管理」「诊断工具」）
 *
 * Java 运行时是启动游戏的核心依赖，诊断工具用于排查启动问题，二者合并便于排障。
 * 顶部子菜单切换（复用 SubTabBar）：
 * - Java 下载器
 * - 已安装版本 Java 环境检测
 * - Java 运行时列表
 * - 版本 JSON 编辑
 *
 * 深链支持：URL `?subtab=manager` 可直接切到对应子页签。
 */
import { onMounted, ref, defineAsyncComponent } from 'vue'
import { useRoute } from 'vue-router'
const SubTabBar = defineAsyncComponent(() => import('@/components/common/SubTabBar.vue'))
import { ArrowDownTrayIcon, CodeBracketIcon, MagnifyingGlassIcon, ServerStackIcon } from '@heroicons/vue/24/outline'
const JavaDownloader = defineAsyncComponent(() => import('./java/JavaDownloader.vue'))
const JavaEnvCheck = defineAsyncComponent(() => import('./java/JavaEnvCheck.vue'))
const JavaManager = defineAsyncComponent(() => import('./data/JavaManager.vue'))
const VersionJsonEditor = defineAsyncComponent(() => import('./data/VersionJsonEditor.vue'))

const subTabs = [
  { id: 'downloader', label: 'Java 下载器', icon: ArrowDownTrayIcon },
  { id: 'env-check', label: '环境检测', icon: MagnifyingGlassIcon },
  { id: 'manager', label: '运行时列表', icon: ServerStackIcon },
  { id: 'version-json', label: '版本 JSON', icon: CodeBracketIcon },
]
const activeSubTab = ref('downloader')

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
        <JavaDownloader v-if="activeSubTab === 'downloader'" />
        <JavaEnvCheck v-else-if="activeSubTab === 'env-check'" />
        <JavaManager v-else-if="activeSubTab === 'manager'" />
        <VersionJsonEditor v-else-if="activeSubTab === 'version-json'" />
      </div>
    </div>
  </div>
</template>
