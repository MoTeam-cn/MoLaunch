<script setup lang="ts">
/**
 * 网络拓扑图（基于 WebRTC ICE candidate 绘制，ECharts graph 力导向布局）
 *
 * 默认折叠，经 Collapse 组件展开。展示「本机 → NAT 设备 → STUN 服务器 / 反射地址」
 * 链路；导入朋友分享后，朋友侧节点一并入图，并以 P2P 连线标注双方联机可能性。
 * 数据源：NatDetectionResult.ice / localIp / publicIp / stunServers。
 */

import { computed, defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import * as echarts from 'echarts/core'
import { GraphChart } from 'echarts/charts'
import { TooltipComponent, LegendComponent } from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'
import type { NatDetectionResult } from '@/types/online'
import { NAT_TYPE_META } from '@/utils/online/nat'
import {
  serializeNatShare,
  judgeP2PFeasibility,
  type NatShareData,
  type P2PVerdict,
} from '@/utils/online/nat-share'
import { buildTopologyOption } from '@/utils/online/topology-option'
import { copyToClipboard } from '@/utils/clipboard'
import { toastSuccess, toastWarning } from '@/utils/toast'
import {
  ShareIcon,
  ChevronDownIcon,
  ClipboardDocumentIcon,
  ArrowDownTrayIcon,
} from '@heroicons/vue/24/outline'

echarts.use([GraphChart, TooltipComponent, LegendComponent, CanvasRenderer])

const Collapse = defineAsyncComponent(() => import('@/components/common/Collapse.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const NatShareImportDialog = defineAsyncComponent(() => import('./NatShareImportDialog.vue'))

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
const stunServers = computed(() => props.result?.stunServers ?? [])

// ============ NAT 分享 ============
const friendShare = ref<NatShareData | null>(null)
const verdict = ref<P2PVerdict | null>(null)
/** 导入 NAT 分享抽屉开关 */
const importOpen = ref(false)

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
  if (!props.result) {
    toastWarning('请先检测 NAT 类型')
    return
  }
  importOpen.value = true
}

/** 导入成功：更新朋友侧节点并判断联机可能性 */
function onImported(data: NatShareData) {
  if (!props.result) return
  friendShare.value = data
  verdict.value = judgeP2PFeasibility(props.result.type, data.type)
}

// ============ ECharts 拓扑图 ============
const chartEl = ref<HTMLDivElement | null>(null)
let chart: echarts.ECharts | null = null
let resizeObserver: ResizeObserver | null = null
let themeObserver: MutationObserver | null = null

function applyOption() {
  if (!chart) return
  chart.setOption(buildTopologyOption({
    localIp: localIp.value,
    natLabel: natLabel.value,
    isReachable: isReachable.value,
    publicIp: publicIp.value,
    publicPort: publicPort.value,
    stunServers: stunServers.value,
    srflxs: srflxs.value,
    friendShare: friendShare.value,
    friendNatLabel: friendNatLabel.value,
    verdict: verdict.value,
  }), true)
}

function ensureChart() {
  if (!chartEl.value) return
  if (!chart) {
    chart = echarts.init(chartEl.value)
    resizeObserver = new ResizeObserver(() => chart?.resize())
    resizeObserver.observe(chartEl.value)
    // 节点被拖出画布可视区时自动释放固定位置，力导向重新布局拉回
    chart.on('dragend', handleDragEnd)
    bindManualPan()
  }
  applyOption()
}

/**
 * 手动平移兜底：ECharts graph 的 roam pan 仅在图内容包围盒（节点外接矩形）内有效，
 * 包围盒外的空白区域无法平移。此处监听 zrender 底层事件，在包围盒外按下拖动时
 * 通过 graphRoam action 平移画布，实现全域可拖拽。
 */
function bindManualPan() {
  if (!chart) return
  const zr = chart.getZr()
  let panning = false
  let lastX = 0
  let lastY = 0
  zr.on('mousedown', (e: { offsetX: number; offsetY: number }) => {
    // 包围盒内交给 ECharts（节点拖拽 / 内置 pan），包围盒外手动平移
    if (chart?.containPixel({ seriesIndex: 0 }, [e.offsetX, e.offsetY])) {
      panning = false
      return
    }
    panning = true
    lastX = e.offsetX
    lastY = e.offsetY
  })
  zr.on('mousemove', (e: { offsetX: number; offsetY: number }) => {
    if (!panning || !chart) return
    const dx = e.offsetX - lastX
    const dy = e.offsetY - lastY
    lastX = e.offsetX
    lastY = e.offsetY
    chart.dispatchAction({ type: 'graphRoam', dx, dy })
  })
  zr.on('mouseup', () => {
    panning = false
  })
  zr.on('globalout', () => {
    panning = false
  })
}

/** 节点被拖出画布可视区（节点中心越界）时重建图表，释放 fx/fy 让力导向拉回 */
function handleDragEnd(params: unknown) {
  if (!chart) return
  const p = params as { dataType?: string; data?: { id?: string; x?: number; y?: number } | null }
  if (p.dataType !== 'node' || !p.data?.id) return
  const { x, y } = p.data
  if (x == null || y == null) return
  const pixel = chart.convertToPixel({ seriesIndex: 0 }, [x, y])
  if (!Array.isArray(pixel)) return
  const [px, py] = pixel
  const margin = 24
  if (
    px < -margin ||
    px > chart.getWidth() + margin ||
    py < -margin ||
    py > chart.getHeight() + margin
  ) {
    applyOption()
  }
}

watch(
  () => [props.result, open.value, hasData.value, friendShare.value, verdict.value],
  async () => {
    await nextTick()
    if (hasData.value && open.value) ensureChart()
  },
  { immediate: true },
)

onMounted(() => {
  // 主题色变化（settingsStore.primaryColor 覆盖 CSS 变量）时刷新图表配色
  themeObserver = new MutationObserver(() => {
    applyOption()
  })
  themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ['style'] })
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  themeObserver?.disconnect()
  chart?.dispose()
  chart = null
})
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
          <div
            ref="chartEl"
            class="w-full"
            :style="{ height: friendShare ? '340px' : '280px' }"
          ></div>
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

    <NatShareImportDialog
      v-if="importOpen && props.result"
      @imported="onImported"
      @close="importOpen = false"
    />
  </div>
</template>