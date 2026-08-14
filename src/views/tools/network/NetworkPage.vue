<script setup lang="ts">
/**
 * 网络工具分类页
 *
 * 顶部子菜单切换（复用 SubTabBar），承载该分类下所有工具：
 * - 服务器状态检测
 * - 网络延迟测试
 *
 * 深链支持：URL `?subtab=latency` 可直接切到对应子页签。
 */
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import SubTabBar from '@/components/common/SubTabBar.vue'
import { BoltIcon, ServerIcon } from '@heroicons/vue/24/outline'
import ServerPinger from '@/views/tools/network/ServerPinger.vue'
import NetworkLatencyTester from '@/views/tools/network/NetworkLatencyTester.vue'

const subTabs = [
  { id: 'server', label: '服务器检测', icon: ServerIcon },
  { id: 'latency', label: '延迟测试', icon: BoltIcon },
]
const activeSubTab = ref('server')

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
        <ServerPinger v-if="activeSubTab === 'server'" />
        <NetworkLatencyTester v-else />
      </div>
    </div>
  </div>
</template>
