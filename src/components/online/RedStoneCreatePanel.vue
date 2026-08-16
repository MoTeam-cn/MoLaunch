<script setup lang="ts">
/**
 * 红石联机 - 创建房间面板
 *
 * 展示红石联机（hongshi 内核）的接入说明。红石联机通过下载内核、连接中转
 * 服务器创建隧道，与 MoLaunch 云端的 Scaffolding 联机相互独立、互不依赖。
 *
 * 功能接入按实现方案推进（见 docs/REDSTONE_ONLINE_PLAN.md）：
 * - 获取中转服务器列表（hongshi.site/newserver.json）
 * - 下载当前平台内核（hongshi.site/api/download/*）
 * - 拉起内核：hongshi.exe -server <addr> -port <mc_port> -status-file <tunnel.ini>
 * - 读取 tunnel.ini 中 status=open 的端口并展示联机地址 <server>:<port>
 */
import { defineAsyncComponent } from 'vue'
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
import { LinkIcon } from '@heroicons/vue/24/outline'
</script>

<template>
  <div class="space-y-4">
    <Card title="红石联机">
      <div class="flex flex-col items-center justify-center py-12 text-center">
        <LinkIcon class="w-12 h-12 text-gray-300 mb-4" />
        <p class="text-sm text-gray-600 font-medium mb-1">红石联机接入规划中</p>
        <p class="text-xs text-gray-400 max-w-sm">
          红石联机通过 hongshi 内核连接中转服务器创建隧道，无需虚拟网卡。
          接入方案将提供内核获取、隧道创建与联机地址展示能力，敬请期待。
        </p>
      </div>
    </Card>
  </div>
</template>