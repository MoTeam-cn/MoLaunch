<script setup lang="ts">
/**
 * 设置 - 联机 Tab：ApiServerCard + easytier 内核/公共节点 + 设备管理
 */
import { onMounted, defineAsyncComponent } from 'vue'
import { useOnlineStore } from '@/stores/online'
const ApiServerCard = defineAsyncComponent(() => import('@/components/settings/ApiServerCard.vue'))
const EasyTierKernelCard = defineAsyncComponent(() => import('@/components/settings/EasyTierKernelCard.vue'))
const GithubProxiesEditor = defineAsyncComponent(() => import('@/components/settings/GithubProxiesEditor.vue'))
const NetworkIdentityEditor = defineAsyncComponent(() => import('@/components/settings/NetworkIdentityEditor.vue'))
const EasyTierPeersEditor = defineAsyncComponent(() => import('@/components/settings/EasyTierPeersEditor.vue'))
const DeviceManagementCard = defineAsyncComponent(() => import('@/components/settings/DeviceManagementCard.vue'))

const onlineStore = useOnlineStore()

onMounted(() => {
  void onlineStore.refreshStatus()
})
</script>

<template>
  <div class="space-y-6">
    <!-- api-server 配置（自管理加载状态） -->
    <ApiServerCard />

    <!-- easytier 内核（外部下载安装） -->
    <EasyTierKernelCard />

    <!-- GitHub 镜像源（easytier 等外部下载竞速选源） -->
    <GithubProxiesEditor />

    <!-- 虚拟网络内设备名（房客侧 easytier hostname，加入方生效） -->
    <NetworkIdentityEditor />

    <!-- easytier 公共节点（信令节点/中继节点均可，默认信令节点内置不显示） -->
    <EasyTierPeersEditor />

    <!-- 设备管理 -->
    <DeviceManagementCard />
  </div>
</template>