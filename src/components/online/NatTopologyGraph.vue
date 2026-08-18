<script setup lang="ts">
/**
 * 网络拓扑图（基于 WebRTC ICE candidate 绘制）
 *
 * 默认折叠，经 Collapse 组件展开。以 SVG 直观展示
 * 「本机 → NAT 设备 → 公网（STUN 服务器 / 反射地址）」链路，
 * 与 NAT 类型 Tag + Tooltip 文字说明互为补充。
 * 数据源：NatDetectionResult.ice / localIp / publicIp / stunServers。
 */

import { computed, ref, defineAsyncComponent } from 'vue'
import type { NatDetectionResult } from '@/types/online'
import { NAT_TYPE_META } from '@/utils/online/nat'
import { ShareIcon, ChevronDownIcon } from '@heroicons/vue/24/outline'
const Collapse = defineAsyncComponent(() => import('@/components/common/Collapse.vue'))

const props = defineProps<{ result: NatDetectionResult | null }>()

/** 折叠状态（默认收起） */
const open = ref(false)

const hosts = computed(() => (props.result?.ice ?? []).filter((c) => c.kind === 'host'))
const srflxs = computed(() => (props.result?.ice ?? []).filter((c) => c.kind === 'srflx'))
const hasData = computed(
  () => !!props.result && (hosts.value.length > 0 || srflxs.value.length > 0),
)

const localIp = computed(() => props.result?.localIp ?? hosts.value[0]?.address ?? '')
const extraHostCount = computed(() => Math.max(0, hosts.value.length - 1))
const natLabel = computed(() =>
  props.result ? (NAT_TYPE_META[props.result.type]?.label ?? '未知') : '',
)
const isReachable = computed(
  () => !!props.result && !['Blocked', 'Unknown'].includes(props.result.type),
)
const publicIp = computed(() => props.result?.publicIp ?? srflxs.value[0]?.address ?? '')
const publicPort = computed(() => (srflxs.value[0] ? String(srflxs.value[0].port) : ''))
const stunServer = computed(() =>
  (props.result?.stunServers ?? []).map((s) => s.replace(/^stun:/, '')).join('、') || '',
)

/** 长地址截断显示 */
function truncate(s: string, n: number): string {
  return s.length > n ? `${s.slice(0, n)}…` : s
}
</script>

<template>
  <div class="border-t border-gray-100 mt-3 pt-3">
    <!-- 标题栏（折叠开关，箭头随状态旋转） -->
    <button
      class="w-full flex items-center justify-between text-sm text-gray-600 hover:text-gray-900 transition-colors"
      @click="open = !open"
    >
      <span class="flex items-center gap-1.5">
        <ShareIcon class="w-4 h-4 text-gray-400" />
        <span class="font-medium">网络拓扑</span>
        <span class="text-xs text-gray-400">基于 ICE candidate 绘制</span>
      </span>
      <ChevronDownIcon
        class="w-4 h-4 text-gray-400 transition-transform duration-300"
        :class="open ? 'rotate-180' : ''"
      />
    </button>

    <Collapse :open="open">
      <div class="pt-3">
        <p v-if="!hasData" class="py-6 text-center text-xs text-gray-400">
          暂无 ICE candidate 数据，点击「重新检测」获取后展示
        </p>
        <svg v-else viewBox="0 0 550 148" class="w-full">
          <defs>
            <marker id="nat-arrow-blue" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto">
              <path d="M0,0 L7,3 L0,6 Z" fill="#3b82f6" />
            </marker>
            <marker id="nat-arrow-green" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto">
              <path d="M0,0 L7,3 L0,6 Z" fill="#10b981" />
            </marker>
            <marker id="nat-arrow-red" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto">
              <path d="M0,0 L7,3 L0,6 Z" fill="#ef4444" />
            </marker>
          </defs>

          <!-- 连线：本机 → NAT（局域网） -->
          <line
            x1="165" y1="96" x2="197" y2="96"
            stroke="#3b82f6" stroke-width="1.5" marker-end="url(#nat-arrow-blue)"
          />
          <text x="152" y="84" font-size="10" fill="#94a3b8" text-anchor="middle">局域网</text>

          <!-- 连线：NAT → 公网（公网；不可达时红色虚线） -->
          <line
            x1="350" y1="96" x2="382" y2="96"
            :stroke="isReachable ? '#10b981' : '#ef4444'"
            :stroke-dasharray="isReachable ? undefined : '4 3'"
            stroke-width="1.5"
            :marker-end="isReachable ? 'url(#nat-arrow-green)' : 'url(#nat-arrow-red)'"
          />
          <text x="342" y="84" font-size="10" fill="#94a3b8" text-anchor="middle">公网</text>

          <!-- 节点：本机（局域网） -->
          <rect x="15" y="50" width="150" height="90" rx="8" fill="#eff6ff" stroke="#93c5fd" stroke-width="1.2" />
          <rect x="15" y="50" width="150" height="4" rx="2" fill="#3b82f6" />
          <text x="90" y="74" font-size="12" font-weight="600" fill="#1e40af" text-anchor="middle">本机（局域网）</text>
          <text x="90" y="96" font-size="11" fill="#334155" text-anchor="middle">{{ truncate(localIp || '未知', 20) }}</text>
          <text x="90" y="114" font-size="10" fill="#94a3b8" text-anchor="middle">
            {{ extraHostCount > 0 ? `共 ${hosts.length} 个本地地址` : '本地出口地址' }}
          </text>

          <!-- 节点：NAT 设备 -->
          <rect x="200" y="50" width="150" height="90" rx="8" fill="#f5f3ff" stroke="#c4b5fd" stroke-width="1.2" />
          <rect x="200" y="50" width="150" height="4" rx="2" fill="#8b5cf6" />
          <text x="275" y="74" font-size="12" font-weight="600" fill="#6d28d9" text-anchor="middle">NAT 设备</text>
          <text x="275" y="96" font-size="11" font-weight="500" fill="#334155" text-anchor="middle">{{ natLabel }}</text>
          <text x="275" y="114" font-size="10" fill="#94a3b8" text-anchor="middle">
            {{ isReachable ? `反射: ${truncate(publicIp, 12)}:${publicPort}` : 'UDP 出站受限' }}
          </text>

          <!-- 节点：公网（STUN） -->
          <rect
            x="385" y="50" width="150" height="90" rx="8"
            :fill="isReachable ? '#ecfdf5' : '#fef2f2'"
            :stroke="isReachable ? '#6ee7b7' : '#fca5a5'"
            stroke-width="1.2"
          />
          <rect
            x="385" y="50" width="150" height="4" rx="2"
            :fill="isReachable ? '#10b981' : '#ef4444'"
          />
          <text x="460" y="74" font-size="12" font-weight="600" :fill="isReachable ? '#065f46' : '#991b1b'" text-anchor="middle">公网（STUN）</text>
          <text x="460" y="96" font-size="10" :fill="isReachable ? '#334155' : '#b91c1c'" text-anchor="middle">{{ truncate(stunServer || 'STUN 服务器', 24) }}</text>
          <text x="460" y="114" font-size="10" fill="#94a3b8" text-anchor="middle">
            {{ isReachable ? `公网 IP: ${truncate(publicIp, 14)}` : '无法获取反射地址' }}
          </text>
        </svg>
      </div>
    </Collapse>
  </div>
</template>
