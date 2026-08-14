<script setup lang="ts">
/**
 * Java 管理分类页
 *
 * Java 运行时是启动游戏的核心依赖，作为重要工具单独一栏。
 * 顶部子菜单切换（复用 SubTabBar）：
 * - Java 下载器
 * - 已安装版本 Java 环境检测
 * - Java 运行时列表
 *
 * 深链支持：URL `?subtab=manager` 可直接切到对应子页签。
 */
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import SubTabBar from '@/components/common/SubTabBar.vue'
import { ArrowDownTrayIcon, MagnifyingGlassIcon, ServerStackIcon } from '@heroicons/vue/24/outline'
import JavaManager from '@/views/tools/data/JavaManager.vue'
import JavaEnvCheck from '@/views/tools/java/JavaEnvCheck.vue'
import JavaDownloader from '@/views/tools/java/JavaDownloader.vue'

const subTabs = [
  { id: 'downloader', label: 'Java 下载器', icon: ArrowDownTrayIcon },
  { id: 'env-check', label: '环境检测', icon: MagnifyingGlassIcon },
  { id: 'manager', label: '运行时列表', icon: ServerStackIcon },
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
        <JavaManager v-else />
      </div>
    </div>
  </div>
</template>
