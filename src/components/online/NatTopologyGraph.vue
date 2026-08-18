<script setup lang="ts">
/**
 * 网络拓扑图（基于 WebRTC ICE candidate 绘制，ECharts graph 力导向布局）
 *
 * 默认折叠，经 Collapse 组件展开。展示「本机 → NAT 设备 → STUN 服务器 / 反射地址」
 * 链路；导入朋友分享后，朋友侧节点一并入图，并以 P2P 连线标注双方联机可能性。
 * 配色继承页面主题色（CSS 变量 --color-primary-*，运行时随换肤变化）。
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

echarts.use([GraphChart, TooltipComponent, LegendComponent, CanvasRenderer])

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
const stunServers = computed(() => props.result?.stunServers ?? [])

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
    '粘贴朋友分享的 NAT 内容，朋友侧节点将加入拓扑图并判断联机可能性：',
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

// ============ ECharts 拓扑图 ============
const chartEl = ref<HTMLDivElement | null>(null)
let chart: echarts.ECharts | null = null
let resizeObserver: ResizeObserver | null = null
let themeObserver: MutationObserver | null = null

/** 读取 CSS 变量（主题色），运行时随 settingsStore.primaryColor 覆盖变化 */
function cssVar(name: string, fallback: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback
}

/** 边标签配置（统一样式） */
function edgeLabel(text: string, color = '#86909c'): object {
  return { show: true, formatter: text, fontSize: 9, color, position: 'middle', offset: [0, -6] }
}

function buildOption() {
  const primary = cssVar('--color-primary-500', '#165dff')
  const primaryDark = cssVar('--color-primary-700', '#0a3aae')
  const primaryLight = cssVar('--color-primary-100', '#dde5ff')
  const friendColor = '#722ed1'
  const friendDark = '#531dab'
  const friendLight = '#f5e8ff'
  const stunColor = '#86909c'
  const textColor = '#4e5969'

  const categories = [
    { name: '本机', itemStyle: { color: primary } },
    { name: 'NAT 设备', itemStyle: { color: primaryDark } },
    { name: 'STUN 服务器', itemStyle: { color: stunColor } },
    { name: '反射地址', itemStyle: { color: primaryLight } },
    { name: '朋友设备', itemStyle: { color: friendColor } },
    { name: '朋友 NAT', itemStyle: { color: friendDark } },
    { name: '朋友反射', itemStyle: { color: friendLight } },
  ]

  const nodes: Record<string, unknown>[] = []
  const links: Record<string, unknown>[] = []

  // ---- 本机侧 ----
  nodes.push({
    id: 'local',
    name: '本机',
    category: 0,
    symbolSize: 46,
    label: { fontSize: 12, fontWeight: 500, color: textColor },
    desc: `本地出口地址：${localIp.value || '未知'}`,
  })
  nodes.push({
    id: 'nat',
    name: `NAT 设备\n(${natLabel.value})`,
    category: 1,
    symbolSize: 46,
    label: { fontSize: 12, fontWeight: 500, color: textColor },
    desc: isReachable.value
      ? `反射地址：${publicIp.value || '未知'}:${publicPort.value || '-'}`
      : 'UDP 出站受限，无法获取反射地址',
  })
  links.push({
    source: 'local',
    target: 'nat',
    label: edgeLabel('Host'),
    lineStyle: { color: primary, width: 2, type: 'solid' },
  })

  // STUN 服务器（最多 2 个）
  stunServers.value.slice(0, 2).forEach((s, i) => {
    const id = `stun-${i}`
    nodes.push({
      id,
      name: `STUN ${i + 1}`,
      category: 2,
      symbolSize: 40,
      label: { fontSize: 11, color: textColor },
      desc: `服务器：${s}`,
    })
    links.push({
      source: 'nat',
      target: id,
      label: edgeLabel('查询'),
      lineStyle: { color: stunColor, width: 1.8, type: 'dashed', curveness: i === 0 ? 0.15 : -0.15 },
    })
  })

  // 反射地址（srflx，最多 2 个）
  srflxs.value.slice(0, 2).forEach((c, i) => {
    const id = `srflx-${i}`
    nodes.push({
      id,
      name: `反射 ${i + 1}`,
      category: 3,
      symbolSize: 36,
      label: { fontSize: 10, color: textColor },
      desc: `${c.address}:${c.port}`,
    })
    links.push({
      source: 'nat',
      target: id,
      label: edgeLabel('srflx'),
      lineStyle: { color: '#a9aeb8', width: 1.5, type: 'dotted', curveness: i === 0 ? 0.1 : -0.1 },
    })
    if (stunServers.value[i]) {
      links.push({
        source: id,
        target: `stun-${i}`,
        label: { show: false },
        lineStyle: { color: '#c9cdd4', width: 1.2, type: 'solid', curveness: i === 0 ? 0.1 : -0.1 },
      })
    }
  })

  // ---- 朋友侧（导入分享后） ----
  if (friendShare.value) {
    nodes.push({
      id: 'friend-local',
      name: '朋友设备',
      category: 4,
      symbolSize: 46,
      label: { fontSize: 12, fontWeight: 500, color: textColor },
      desc: `本地出口地址：${friendShare.value.localIp || '未知'}`,
    })
    nodes.push({
      id: 'friend-nat',
      name: `朋友 NAT\n(${friendNatLabel.value})`,
      category: 5,
      symbolSize: 46,
      label: { fontSize: 12, fontWeight: 500, color: textColor },
      desc: `反射地址：${friendShare.value.publicIp || '未知'}`,
    })
    links.push({
      source: 'friend-local',
      target: 'friend-nat',
      label: edgeLabel('Host'),
      lineStyle: { color: friendColor, width: 2, type: 'solid' },
    })

    const friendSrflxs = (friendShare.value.ice ?? [])
      .filter((c) => c.kind === 'srflx')
      .slice(0, 2)
    friendSrflxs.forEach((c, i) => {
      const id = `friend-srflx-${i}`
      nodes.push({
        id,
        name: `朋友反射 ${i + 1}`,
        category: 6,
        symbolSize: 36,
        label: { fontSize: 10, color: textColor },
        desc: `${c.address}:${c.port}`,
      })
      links.push({
        source: 'friend-nat',
        target: id,
        label: edgeLabel('srflx'),
        lineStyle: { color: '#c9cdd4', width: 1.5, type: 'dotted', curveness: i === 0 ? 0.1 : -0.1 },
      })
    })

    // P2P 直连：我的反射 → 朋友反射（颜色/线型随联机可能性等级）
    if (srflxs.value.length > 0 && friendSrflxs.length > 0 && verdict.value) {
      const styleMap: Record<P2PVerdict['level'], { color: string; type: string }> = {
        high: { color: '#10b981', type: 'solid' },
        medium: { color: '#f7ba1e', type: 'dashed' },
        low: { color: '#ff7d00', type: 'dashed' },
        none: { color: '#f53f3f', type: 'dashed' },
        unknown: { color: '#86909c', type: 'dashed' },
      }
      const style = styleMap[verdict.value.level]
      links.push({
        source: 'srflx-0',
        target: 'friend-srflx-0',
        label: edgeLabel(verdict.value.label, style.color),
        lineStyle: { color: style.color, width: 2.2, type: style.type, curveness: 0.2 },
      })
    }
  }

  return {
    backgroundColor: 'transparent',
    tooltip: {
      trigger: 'item',
      formatter: (params: { dataType?: string; name?: string; data?: { desc?: string } }) => {
        if (params.dataType === 'node') {
          return `<strong>${params.name ?? ''}</strong><br/>${params.data?.desc ?? ''}`
        }
        return ''
      },
      backgroundColor: '#ffffff',
      borderColor: '#e5e6eb',
      borderWidth: 1,
      textStyle: { color: '#1d2129' },
    },
    legend: {
      show: true,
      data: categories.map((c) => c.name),
      icon: 'circle',
      orient: 'horizontal',
      left: 'center',
      top: 0,
      itemWidth: 10,
      itemHeight: 10,
      textStyle: { fontSize: 11, color: textColor },
      backgroundColor: '#f2f3f5',
      borderRadius: 20,
      padding: [3, 12],
      borderColor: '#e5e6eb',
      borderWidth: 1,
    },
    series: [
      {
        type: 'graph',
        layout: 'force',
        force: {
          repulsion: 300,
          edgeLength: [80, 200],
          gravity: 0.08,
          friction: 0.2,
          layoutAnimation: true,
        },
        data: nodes,
        links,
        categories,
        roam: true,
        draggable: true,
        edgeSymbol: ['none', 'arrow'],
        edgeSymbolSize: [0, 8],
        label: {
          show: true,
          position: 'bottom',
          fontSize: 12,
          color: textColor,
          offset: [0, 6],
        },
        edgeLabel: {
          show: true,
          fontSize: 9,
          color: '#86909c',
          position: 'middle',
          offset: [0, -6],
        },
        lineStyle: {
          color: 'source',
          curveness: 0.2,
          width: 1.8,
          opacity: 0.7,
        },
        itemStyle: {
          borderColor: '#e5e6eb',
          borderWidth: 1.5,
        },
        symbolSize: 44,
        emphasis: {
          focus: 'adjacency',
          lineStyle: { width: 2.5 },
        },
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
    // 节点被拖出画布可视区时自动释放固定位置，力导向重新布局拉回
    chart.on('dragend', handleDragEnd)
    bindManualPan()
  }
  chart.setOption(buildOption(), true)
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
    chart.setOption(buildOption(), true)
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
  </div>
</template>