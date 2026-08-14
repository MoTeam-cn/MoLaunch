<script setup lang="ts">
/**
 * P2P 组网失败诊断卡片（加入方 / 房主共用）
 *
 * 组网失败时替代僵硬错误提示：以友好文案说明原因，绘制双方 NAT 类型；
 * 服务端 TURN 中继无可用资源时给出第三方 FRP / 虚拟组网等替代联机方案。
 */
import { computed, defineAsyncComponent } from 'vue'
import { resolveNatMeta, getNatFeasibilityColorClass } from '@/utils/online/nat-type'
import { useOnlineStore } from '@/stores/online'
import { ExclamationTriangleIcon } from '@heroicons/vue/24/outline'
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))

export interface PeerNatEntry {
  /** 对端标识（如「房主」或设备短 ID） */
  label: string
  /** 对端 NAT 类型（未收到时为 null） */
  natType: string | null
}

withDefaults(
  defineProps<{
    /** 自己的 NAT 类型（未检测时为 null） */
    selfNatType?: string | null
    /** 对端 NAT 类型列表 */
    peers?: PeerNatEntry[]
  }>(),
  {
    selfNatType: null,
    peers: () => [],
  },
)

const store = useOnlineStore()

/** 服务端 TURN 中继资源是否不可用（未拉取到或返回空列表均视为无资源） */
const turnUnavailable = computed(() => {
  const turn = store.systemTurnServers
  return !turn || turn.servers.length === 0
})

/** NAT 徽章配色 class */
function natBadgeClass(type: string | null) {
  const meta = resolveNatMeta(type)
  return meta ? getNatFeasibilityColorClass(meta.feasibility) : 'bg-gray-100 text-gray-600'
}

/** NAT 徽章文案 */
function natLabel(type: string | null) {
  return resolveNatMeta(type)?.label ?? (type || '未获取')
}
</script>

<template>
  <div class="rounded-lg border border-red-200 bg-red-50 px-4 py-3">
    <div class="flex items-start gap-2.5">
      <ExclamationTriangleIcon class="w-5 h-5 text-red-500 shrink-0 mt-0.5" />
      <div class="flex-1 min-w-0">
        <div class="text-sm font-medium text-red-700">P2P 组网没能打通</div>
        <p class="mt-1 text-xs leading-relaxed text-red-600">
          两台设备没能直接建立点对点连接。这通常是因为双方的网络都处在较严格的 NAT 后面，STUN
          打洞没能穿透，属于常见现象，换种方式一般都能连上。
        </p>

        <!-- 双方 NAT 类型 -->
        <div class="mt-3 grid grid-cols-2 gap-2">
          <div class="rounded bg-white/70 border border-red-100 px-2.5 py-2">
            <div class="text-xs text-red-500">我的网络</div>
            <div class="mt-1.5">
              <Tooltip :text="resolveNatMeta(selfNatType)?.tooltip ?? '未获取到 NAT 类型'">
                <span
                  class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium cursor-help"
                  :class="natBadgeClass(selfNatType)"
                >
                  {{ natLabel(selfNatType) }}
                </span>
              </Tooltip>
            </div>
          </div>
          <div class="rounded bg-white/70 border border-red-100 px-2.5 py-2">
            <div class="text-xs text-red-500">对方网络</div>
            <div class="mt-1.5 space-y-1">
              <template v-if="peers.length > 0">
                <span v-for="peer in peers" :key="peer.label" class="flex items-center gap-1.5">
                  <span class="text-xs text-gray-500 shrink-0">{{ peer.label }}</span>
                  <Tooltip :text="resolveNatMeta(peer.natType)?.tooltip ?? '对方尚未上报 NAT 类型'">
                    <span
                      class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium cursor-help"
                      :class="natBadgeClass(peer.natType)"
                    >
                      {{ natLabel(peer.natType) }}
                    </span>
                  </Tooltip>
                </span>
              </template>
              <span
                v-else
                class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-600"
              >
                未获取
              </span>
            </div>
          </div>
        </div>

        <!-- TURN 无资源：第三方 FRP / 虚拟组网等替代方案 -->
        <template v-if="turnUnavailable">
          <div class="mt-3 pt-3 border-t border-red-200">
            <div class="text-xs text-red-600">
              另外，联机服务器目前没有可用的中继（TURN）资源，无法自动补位转接。可以试试这些替代联机方案：
            </div>
            <ul class="mt-2 space-y-1.5 text-xs text-red-700 list-disc pl-4 leading-relaxed">
              <li>
                使用第三方内网穿透 / FRP 服务（如 SakuraFrp、花生壳等），把房主的游戏端口映射到公网
              </li>
              <li>
                使用虚拟组网工具（如 ZeroTier、Radmin VPN、Tailscale 等），把双方拉进同一虚拟局域网
              </li>
              <li>
                房主在路由器上开启 UPnP、DMZ 或端口映射后再试；双方也可更换网络（如手机热点）重新进房
              </li>
            </ul>
            <div class="mt-2 text-xs text-red-500">应用会持续自动重试连接，网络调整好后无需手动操作。</div>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>
