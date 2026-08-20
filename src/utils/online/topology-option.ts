/**
 * 网络拓扑图 - ECharts graph 力导向布局 option 构建
 *
 * 展示「本机 → NAT 设备 → STUN 服务器 / 反射地址」链路；导入朋友分享后，
 * 朋友侧节点一并入图，并以 P2P 连线标注双方联机可能性。
 * 配色继承页面主题色（CSS 变量 --color-primary-*，运行时随换肤变化）。
 */
import type { EChartsCoreOption } from 'echarts/core'
import type { IceCandidateInfo } from '@/types/online'
import type { NatShareData, P2PVerdict } from './nat-share'

export interface TopologyOptionInput {
  localIp: string
  natLabel: string
  isReachable: boolean
  publicIp: string
  publicPort: string
  stunServers: string[]
  srflxs: IceCandidateInfo[]
  friendShare: NatShareData | null
  friendNatLabel: string
  verdict: P2PVerdict | null
}

/** 读取 CSS 变量（主题色），运行时随 settingsStore.primaryColor 覆盖变化 */
function cssVar(name: string, fallback: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback
}

/** 边标签配置（统一样式） */
function edgeLabel(text: string, color = '#86909c'): object {
  return { show: true, formatter: text, fontSize: 9, color, position: 'middle', offset: [0, -6] }
}

export function buildTopologyOption(input: TopologyOptionInput): EChartsCoreOption {
  const {
    localIp,
    natLabel,
    isReachable,
    publicIp,
    publicPort,
    stunServers,
    srflxs,
    friendShare,
    friendNatLabel,
    verdict,
  } = input
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
    desc: `本地出口地址：${localIp || '未知'}`,
  })
  nodes.push({
    id: 'nat',
    name: `NAT 设备\n(${natLabel})`,
    category: 1,
    symbolSize: 46,
    label: { fontSize: 12, fontWeight: 500, color: textColor },
    desc: isReachable
      ? `反射地址：${publicIp || '未知'}:${publicPort || '-'}`
      : 'UDP 出站受限，无法获取反射地址',
  })
  links.push({
    source: 'local',
    target: 'nat',
    label: edgeLabel('Host'),
    lineStyle: { color: primary, width: 2, type: 'solid' },
  })

  // STUN 服务器（最多 2 个）
  stunServers.slice(0, 2).forEach((s, i) => {
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
  srflxs.slice(0, 2).forEach((c, i) => {
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
    if (stunServers[i]) {
      links.push({
        source: id,
        target: `stun-${i}`,
        label: { show: false },
        lineStyle: { color: '#c9cdd4', width: 1.2, type: 'solid', curveness: i === 0 ? 0.1 : -0.1 },
      })
    }
  })

  // ---- 朋友侧（导入分享后） ----
  if (friendShare) {
    nodes.push({
      id: 'friend-local',
      name: '朋友设备',
      category: 4,
      symbolSize: 46,
      label: { fontSize: 12, fontWeight: 500, color: textColor },
      desc: `本地出口地址：${friendShare.localIp || '未知'}`,
    })
    nodes.push({
      id: 'friend-nat',
      name: `朋友 NAT\n(${friendNatLabel})`,
      category: 5,
      symbolSize: 46,
      label: { fontSize: 12, fontWeight: 500, color: textColor },
      desc: `反射地址：${friendShare.publicIp || '未知'}`,
    })
    links.push({
      source: 'friend-local',
      target: 'friend-nat',
      label: edgeLabel('Host'),
      lineStyle: { color: friendColor, width: 2, type: 'solid' },
    })

    const friendSrflxs = (friendShare.ice ?? [])
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
    if (srflxs.length > 0 && friendSrflxs.length > 0 && verdict) {
      const styleMap: Record<P2PVerdict['level'], { color: string; type: string }> = {
        high: { color: '#10b981', type: 'solid' },
        medium: { color: '#f7ba1e', type: 'dashed' },
        low: { color: '#ff7d00', type: 'dashed' },
        none: { color: '#f53f3f', type: 'dashed' },
        unknown: { color: '#86909c', type: 'dashed' },
      }
      const style = styleMap[verdict.level]
      links.push({
        source: 'srflx-0',
        target: 'friend-srflx-0',
        label: edgeLabel(verdict.label, style.color),
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