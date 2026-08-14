<script setup lang="ts">
/**
 * 游戏资源分类页
 *
 * 顶部子菜单切换（复用 SubTabBar），承载该分类下所有工具：
 * - 截图批量管理
 * - 资源包转换
 *
 * 深链支持：URL `?subtab=resourcepack` 可直接切到对应子页签。
 */
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import SubTabBar from '@/components/common/SubTabBar.vue'
import { CameraIcon, PhotoIcon } from '@heroicons/vue/24/outline'
import ScreenshotManager from '@/views/tools/data/ScreenshotManager.vue'
import ResourcePackConverter from '@/views/tools/data/ResourcePackConverter.vue'

const subTabs = [
  { id: 'screenshot', label: '截图管理', icon: CameraIcon },
  { id: 'resourcepack', label: '资源包转换', icon: PhotoIcon },
]
const activeSubTab = ref('screenshot')

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
        <ScreenshotManager v-if="activeSubTab === 'screenshot'" />
        <ResourcePackConverter v-else />
      </div>
    </div>
  </div>
</template>
