<script setup lang="ts">
/**
 * 连接传输方式状态行（P2P 直连 / TURN 中继 + SVG 国旗）
 *
 * 轮询 getStats 检测实际传输方式：已选 candidate-pair 的 localCandidate 为
 * relay 时走 TURN 中继（显示节点国旗与名称），否则为 P2P 直连。
 * 房主/加入方共用：`pcs` 传入需检测的 PC 数组（房主为全部参与者 PC，加入方为自身 PC）。
 */

import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { hasFlag } from 'country-flag-icons'
import * as flags from 'country-flag-icons/string/3x2'
import { detectTransportInfo, type TransportInfo } from '@/utils/online/transport-info'
import type { IceServerEntry } from '@/types/online'

const props = defineProps<{
  /** 需检测的 PeerConnection 数组（房主=全部参与者，加入方=自身） */
  pcs: RTCPeerConnection[]
  /** ICE 服务器条目（用于匹配中继 URL 对应的 regionCode/name） */
  iceServers: IceServerEntry[]
}>()

/** 轮询间隔（ms） */
const POLL_INTERVAL_MS = 5_000

const transport = ref<TransportInfo | null>(null)
let timer: number | undefined

/** 中继节点 SVG 国旗（按 ISO 3166 alpha-2 映射，无效代码返回空） */
const flagSvg = computed(() => {
  const regionCode = transport.value?.regionCode
  if (!regionCode) return ''
  const key = regionCode.toUpperCase()
  if (!hasFlag(key)) return ''
  return (flags as Record<string, string>)[key] ?? ''
})

async function refresh() {
  const list = props.pcs
  if (list.length === 0) {
    transport.value = null
    return
  }
  const results = await Promise.all(
    list.map((pc) => detectTransportInfo(pc, props.iceServers)),
  )
  const relay = results.find((r) => r?.mode === 'relay')
  if (relay) {
    transport.value = relay
  } else if (results.some((r) => r?.mode === 'direct')) {
    transport.value = { mode: 'direct' }
  } else {
    transport.value = null
  }
}

watch(() => props.pcs, () => void refresh())

onMounted(() => {
  void refresh()
  timer = window.setInterval(() => void refresh(), POLL_INTERVAL_MS)
})

onBeforeUnmount(() => {
  if (timer) window.clearInterval(timer)
})
</script>

<template>
  <div v-if="transport" class="flex items-center justify-between">
    <span class="text-xs text-gray-500">传输方式</span>
    <span class="inline-flex items-center gap-1.5 text-xs">
      <template v-if="transport.mode === 'relay'">
        <!-- 国旗 SVG 来自 country-flag-icons 静态包且经 hasFlag 校验，无注入风险 -->
        <!-- eslint-disable-next-line vue/no-v-html -->
        <span class="inline-block w-4 h-3 overflow-hidden rounded-sm shrink-0 ring-1 ring-gray-200" v-html="flagSvg" />
        <span class="font-medium text-primary-600">TURN 中转</span>
        <span v-if="transport.name" class="text-gray-500">{{ transport.name }}</span>
      </template>
      <template v-else>
        <span class="font-medium text-green-700">P2P 直连</span>
      </template>
    </span>
  </div>
</template>
