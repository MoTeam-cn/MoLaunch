<script setup lang="ts">
/**
 * Mod 网络分类页（归并原「Mod 工具」「网络工具」）
 *
 * 顶部子菜单切换（复用 SubTabBar）：
 * - Mod 依赖检测
 * - Mod 文件去重
 * - 服务器状态检测
 * - 网络延迟测试
 * - 地址测速
 *
 * 深链支持：URL `?subtab=latency` 可直接切到对应子页签。
 */
import { onMounted, ref, defineAsyncComponent } from 'vue'
import { useRoute } from 'vue-router'
const SubTabBar = defineAsyncComponent(() => import('@/components/common/SubTabBar.vue'))
import { BoltIcon, ScissorsIcon, ServerIcon, ShieldCheckIcon, SignalIcon } from '@heroicons/vue/24/outline'
const ModDependencyChecker = defineAsyncComponent(() => import('./mod-tools/ModDependencyChecker.vue'))
const ModDedupScanner = defineAsyncComponent(() => import('./mod-tools/ModDedupScanner.vue'))
const ServerPinger = defineAsyncComponent(() => import('./network/ServerPinger.vue'))
const NetworkLatencyTester = defineAsyncComponent(() => import('./network/NetworkLatencyTester.vue'))
const AddressLatencyTester = defineAsyncComponent(() => import('./network/AddressLatencyTester.vue'))

const subTabs = [
  { id: 'dependency', label: '依赖检测', icon: ShieldCheckIcon },
  { id: 'dedup', label: 'Mod 去重', icon: ScissorsIcon },
  { id: 'server', label: '服务器检测', icon: ServerIcon },
  { id: 'latency', label: '延迟测试', icon: BoltIcon },
  { id: 'addr', label: '地址测速', icon: SignalIcon },
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
        <ModDedupScanner v-else-if="activeSubTab === 'dedup'" />
        <ServerPinger v-else-if="activeSubTab === 'server'" />
        <NetworkLatencyTester v-else-if="activeSubTab === 'latency'" />
        <AddressLatencyTester v-else />
      </div>
    </div>
  </div>
</template>
