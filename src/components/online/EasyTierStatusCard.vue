<script setup lang="ts">
/**
 * easytier 虚拟组网状态卡片（设备面板展示）
 *
 * 展示组网状态 / core 版本 / 虚拟 IP / 虚拟网络名 / 进程 PID。
 * 状态来源：全局已监听后端 `easytier-status` 事件（App.vue 注册）+ 打开页面时
 * `easytier_status` 查询兜底，统一写入 online store 的 easytier 切片。
 */
import { computed, onMounted, defineAsyncComponent } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { getEasyTierStatus } from '@/utils/api/online-manager/easytier'
import { useEasyTierInstall } from '@/composables/useEasyTierInstall'
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
const SealedOverlay = defineAsyncComponent(() => import('@/components/common/SealedOverlay.vue'))
const EasyTierStatusBadge = defineAsyncComponent(() => import('./EasyTierStatusBadge.vue'))
import {
  ServerStackIcon,
  GlobeAltIcon,
  TagIcon,
  CpuChipIcon,
} from '@heroicons/vue/24/outline'

const store = useOnlineStore()
/** easytier 内核安装状态（缺失时显示下载引导，点击前往设置页） */
const install = useEasyTierInstall()
/** 内核缺失（明确未安装才显示引导；null=未知/检查中不显示） */
const kernelMissing = computed(() => install.installed.value === false)

const version = computed(() => store.easytierRuntime.version)
const ip = computed(() => store.easytierRuntime.virtualIp)
const networkName = computed(() => store.easytierRuntime.networkName)
const pid = computed(() => store.easytierRuntime.pid)

onMounted(async () => {
  // 打开页面时查询一次兜底（emit 仅在有动作时推送，全局监听在 App.vue 已注册）
  try {
    const status = await getEasyTierStatus()
    store.setEasyTierRuntime({
      joined: status.joined,
      version: status.version ?? '',
      pid: status.pid,
      rpcPortal: status.rpcPortal ?? '',
      networkName: status.networkName ?? '',
      virtualIp: status.virtualIp ?? '',
    })
  } catch {
    // 查询失败保持现状，等待后续 emit 推送
  }
  // 内核安装状态（缺失时显示引导条；done 事件自动解除）
  void install.checkStatus()
})
</script>

<template>
  <Card title="虚拟组网（easytier）">
    <!-- 内核缺失时封存（与「联机服务不可用」一致）：虚线红框遮罩，点击弹窗引导前往设置页下载 -->
    <div class="relative">
      <SealedOverlay
        v-if="kernelMissing"
        reason="easytier 内核未安装，联机功能暂不可用，请前往 设置-联机 页面下载内核"
        @request="install.promptMissing('虚拟组网')"
      />
      <div class="divide-y divide-gray-100">
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ServerStackIcon class="w-4 h-4 text-gray-400" />
            <span>组网状态</span>
          </div>
          <EasyTierStatusBadge />
        </div>
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <TagIcon class="w-4 h-4 text-gray-400" />
            <span>core 版本</span>
          </div>
          <code class="text-xs text-gray-900 bg-gray-50 px-2 py-0.5 rounded">{{ version || '-' }}</code>
        </div>
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <GlobeAltIcon class="w-4 h-4 text-gray-400" />
            <span>虚拟网络</span>
          </div>
          <code class="text-xs text-gray-900 bg-gray-50 px-2 py-0.5 rounded max-w-[220px] truncate">{{ networkName || '-' }}</code>
        </div>
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <GlobeAltIcon class="w-4 h-4 text-gray-400" />
            <span>虚拟 IP</span>
          </div>
          <code class="text-xs text-gray-900 bg-gray-50 px-2 py-0.5 rounded font-mono">{{ ip || '-' }}</code>
        </div>
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <CpuChipIcon class="w-4 h-4 text-gray-400" />
            <span>进程 PID</span>
          </div>
          <span class="text-xs text-gray-900 font-mono">{{ pid ?? '-' }}</span>
        </div>
      </div>
    </div>
  </Card>
</template>
