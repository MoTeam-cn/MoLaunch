<script setup lang="ts">
/**
 * 网络拓扑图（基于 WebRTC ICE candidate 绘制，ECharts graph）
 *
 * 默认折叠，经 Collapse 组件展开。以 ECharts 图（线条互连）直观展示
 * 「本机 → NAT 设备 → 公网（STUN 服务器 / 反射地址）」链路，
 * 节点与连线颜色继承页面主题色（CSS 变量 --color-primary-*，运行时随换肤变化）。
 * 附带 NAT 分享算法：复制分享内容 / 导入朋友分享，判断双方联机可能性。
 * 数据源：NatDetectionResult.ice / localIp / publicIp / stunServers。
 */

import { computed, defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import * as echarts from 'echarts/core'
import { GraphChart } from 'echarts/charts'
import { TooltipComponent } from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import type { NatDetectionResult } from '@/types/online'
import { NAT_TYPE_META } from '@/utils/online/nat'
import {
  serializeNatShare,
  parseNatShare,
  judgeP2PFeasibility,
  type NatShareData,
  type P2PVerdict,
} from '@/utils/online/nat-share'
import { copyToClipboard } from '@/utils/clipboard'
import { showPrompt } from '@/utils/modal'
import { toastSuccess, toastError, toastWarning } from '@/utils/toast'
import {
  ShareIcon,
  ChevronDownIcon,
  ClipboardDocumentIcon,
  ArrowDownTrayIcon,
} from '@heroicons/vue/24/outline'

echarts.use([GraphChart, TooltipComponent, CanvasRenderer])

const Collapse = defineAsyncComponent(() => import('@/components/common/Collapse.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))

const props = defineProps<{ result: NatDetectionResult | null }>()

/** 折叠状态（默认收起） */
const open = ref(false)

const hosts = computed(() => (props.result?.ice ?? []).filter((c) => c.kind === 'host'))
const srflxs = computed(() => (props.result?.ice ?? []).filter((c) => c.kind === 'srflx'))
const hasData = computed(
  () => !!props.result && (hosts.value.length > 0 || srflxs.value.length > 0),
)

const localIp = computed(() => props.result?.localIp ?? hosts.value[0]?.address ?? '')
const natLabel = computed(() =>
  props.result ? (NAT_TYPE_META[props.result.type]?.label ?? '未知') : '',
)
const isReachable = computed(
  () => !!props.result && !['Blocked', 'Unknown'].includes(props.result.type),
)
const publicIp = computed(() => props.result?.publicIp ?? srflxs.value[0]?.address ?? '')
const publicPort = computed(() => (srflxs.value[0] ? String(srflxs.value[0].port) : ''))
const stunServer = computed(
  () => (props.result?.stunServers ?? []).map((s) => s.replace(/^stun:/, '')).join('、') || '',
)

/** 长地址截断显示 */
function truncate(s: string, n: number): string {
  return s.length > n ? `${s.slice(0, n)}…` : s
}

// ============ ECharts 拓扑图 ============
const chartEl = ref<HTMLDivElement | null>(null)
let chart: echarts.ECharts | null = null
let resizeObserver: ResizeObserver | null = null
let themeObserver: MutationObserver | null = null

/** 读取 CSS 变量（主题色），运行时随 settingsStore.primaryColor 覆盖变化 */
function cssVar(name: string, fallback: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback
}

function buildOption() {
  const primary = cssVar('--color-primary-500', '#165dff')
  const primaryDark = cssVar('--color-primary-700', '#0a3aae')
  const textLight = '#86909c'
  const reachable = isReachable.value
  const edgeColor = reachable ? '#10b981' : '#ef4444'
  return {
    backgroundColor: 'transparent',
    tooltip: {
      show: true,
      trigger: 'item',
      formatter: (params: { dataType?: string; data?: { name?: string; desc?: string } }) => {
        if (params.dataType === 'node' && params.data) {
          return `<b>${params.data.name ?? ''}</b><br/>${params.data.desc ?? ''}`
        }
        return ''
      },
    },
    series: [
      {
        type: 'graph',
        layout: 'none',
        roam: false,
        draggable: false,
        edgeSymbol: ['none', 'arrow'],
        edgeSymbolSize: 8,
        label: {
          show: true,
          position: 'inside',
          color: '#ffffff',
          fontSize: 12,
          fontWeight: 600,
          lineHeight: 18,
        },
        itemStyle: { borderRadius: 8 },
        data: [
          {
            name: '本机',
            x: 90,
            y: 100,
            symbol: 'roundRect',
            symbolSize: [120, 60],
            itemStyle: { color: primary },
            label: { formatter: `本机\n${truncate(localIp.value || '未知', 16)}` },
            desc: `本地出口地址：${localIp.value || '未知'}`,
          },
          {
            name: 'NAT 设备',
            x: 300,
            y: 100,
            symbol: 'roundRect',
            symbolSize: [120, 60],
            itemStyle: { color: primaryDark },
            label: { formatter: `NAT 设备\n${natLabel.value}` },
            desc: reachable
              ? `反射地址：${publicIp.value || '未知'}:${publicPort.value || '-'}`
              : 'UDP 出站受限，无法获取反射地址',
          },
          {
            name: '公网',
            x: 510,
            y: 100,
            symbol: 'roundRect',
            symbolSize: [120, 60],
            itemStyle: { color: reachable ? '#10b981' : '#ef4444' },
            label: { formatter: `公网 STUN\n${truncate(stunServer.value || 'STUN 服务器', 14)}` },
            desc: reachable
              ? `STUN：${stunServer.value || '未知'}\n公网 IP：${publicIp.value || '未知'}`
              : '无法获取反射地址',
          },
        ],
        links: [
          {
            source: '本机',
            target: 'NAT 设备',
            lineStyle: { color: primary, width: 1.5 },
            label: {
              show: true,
              formatter: '局域网',
              position: 'middle',
              color: textLight,
              fontSize: 11,
            },
          },
          {
            source: 'NAT 设备',
            target: '公网',
            lineStyle: {
              color: edgeColor,
              width: 1.5,
              type: reachable ? 'solid' : 'dashed',
            },
            label: {
              show: true,
              formatter: '公网',
              position: 'middle',
              color: textLight,
              fontSize: 11,
            },
          },
        ],
      },
    ],
  }
}

function ensureChart() {
  if (!chartEl.value) return
  if (!chart) {
    chart = echarts.init(chartEl.value)
    resizeObserver = new ResizeObserver(() => chart?.resize())
    resizeObserver.observe(chartEl.value)
  }
  chart.setOption(buildOption(), true)
}

watch(
  () => [props.result, open.value, hasData.value],
  async () => {
    await nextTick()
    if (hasData.value && open.value) ensureChart()
  },
  { immediate: true },
)

onMounted(() => {
  // 主题色变化（settingsStore.primaryColor 覆盖 CSS 变量）时刷新图表配色
  themeObserver = new MutationObserver(() => {
    if (chart) chart.setOption(buildOption(), true)
  })
  themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ['style'] })
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  themeObserver?.disconnect()
  chart?.dispose()
  chart = null
})

// ============ NAT 分享 ============
const friendShare = ref<NatShareData | null>(null)
const verdict = ref<P2PVerdict | null>(null)

const myNatLabel = computed(() =>
  props.result ? (NAT_TYPE_META[props.result.type]?.label ?? '未知') : '',
)
const friendNatLabel = computed(() =>
  friendShare.value ? (NAT_TYPE_META[friendShare.value.type]?.label ?? '未知') : '',
)

/** 联机可能性等级 → Tag 预设色 */
const VERDICT_TAG_COLOR: Record<P2PVerdict['level'], string> = {
  high: 'green',
  medium: 'blue',
  low: 'gold',
  none: 'red',
  unknown: 'gray',
}

async function handleShare() {
  if (!props.result) {
    toastWarning('请先检测 NAT 类型')
    return
  }
  const ok = await copyToClipboard(serializeNatShare(props.result))
  if (ok) toastSuccess('分享内容已复制，发送给朋友即可')
}

function handleImport() {
  const result = props.result
  if (!result) {
    toastWarning('请先检测 NAT 类型')
    return
  }
  showPrompt(
    '导入 NAT 分享',
    '粘贴朋友分享的 NAT 内容，判断双方联机可能性：',
    (value) => {
      const data = parseNatShare(value)
      if (!data) {
        toastError('分享内容无效，请确认完整复制')
        return
      }
      friendShare.value = data
      verdict.value = judgeP2PFeasibility(result.type, data.type)
    },
    { placeholder: 'MoLaunchNATv1|...' },
  )
}
</script>

<template>
  <div class="border-t border-gray-100 mt-3 pt-3">
    <!-- 标题栏：折叠开关 + 分享/导入操作 -->
    <div class="flex items-center justify-between">
      <button
        class="flex items-center gap-1.5 text-sm text-gray-600 hover:text-gray-900 transition-colors"
        @click="open = !open"
      >
        <ShareIcon class="w-4 h-4 text-gray-400" />
        <span class="font-medium">网络拓扑</span>
        <span class="text-xs text-gray-400">基于 ICE candidate 绘制</span>
        <ChevronDownIcon
          class="w-4 h-4 text-gray-400 transition-transform duration-300"
          :class="open ? 'rotate-180' : ''"
        />
      </button>
      <div class="flex items-center gap-1">
        <Tooltip text="复制分享内容">
          <Button type="ghost" size="mini" :disabled="!hasData" @click="handleShare">
            <template #icon><ClipboardDocumentIcon class="w-3.5 h-3.5" /></template>
          </Button>
        </Tooltip>
        <Tooltip text="导入朋友的分享内容">
          <Button type="ghost" size="mini" @click="handleImport">
            <template #icon><ArrowDownTrayIcon class="w-3.5 h-3.5" /></template>
          </Button>
        </Tooltip>
      </div>
    </div>

    <Collapse :open="open">
      <div class="pt-3">
        <p v-if="!hasData" class="py-6 text-center text-xs text-gray-400">
          暂无 ICE candidate 数据，点击「重新检测」获取后展示
        </p>
        <template v-else>
          <div ref="chartEl" class="w-full" style="height: 180px"></div>
          <!-- 与朋友的联机可能性 -->
          <div v-if="verdict && friendShare" class="mt-3 rounded-lg border border-gray-200 p-3">
            <div class="flex items-center justify-between">
              <span class="text-xs text-gray-500">与朋友的联机可能性</span>
              <Tag size="small" :color="VERDICT_TAG_COLOR[verdict.level]">
                {{ verdict.label }}
              </Tag>
            </div>
            <div class="mt-2 text-xs text-gray-600">
              我（{{ myNatLabel }}） × 朋友（{{ friendNatLabel }}）
            </div>
            <p class="mt-1 text-xs text-gray-500">{{ verdict.detail }}</p>
          </div>
        </template>
      </div>
    </Collapse>
  </div>
</template>