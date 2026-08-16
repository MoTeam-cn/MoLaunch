<script setup lang="ts">
/**
 * 地址测速
 *
 * 对地址做 TCP 握手（tcping）/ UDP 探针 / ICMP ping 测延迟。
 * 每行一个目标：host 或 host:port（端口缺省 80），可加名称前缀 名称|host。
 */
import { ref, computed, defineAsyncComponent } from 'vue'
import {
  SignalIcon,
  BoltIcon,
  CheckCircleIcon,
  XCircleIcon,
} from '@heroicons/vue/24/outline'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
import { toastSuccess, toastError } from '@/utils/toast'
import { addressLatencyTest } from '@/utils/api/tools'
import type { AddressLatencyItem, AddressTarget } from '@/utils/api/tools'

const PROTOCOL_OPTIONS = [
  { label: 'TCP 握手', value: 'tcp' },
  { label: 'UDP 探针', value: 'udp' },
  { label: 'Ping', value: 'ping' },
] as const

const text = ref('')
const protocol = ref<'tcp' | 'udp' | 'ping'>('tcp')
const testing = ref(false)
const results = ref<AddressLatencyItem[]>([])

interface ParsedTarget {
  name?: string
  host: string
  port: number
}

/** 未携带端口时的默认端口 */
const DEFAULT_PORT = 80

function parseLine(line: string): ParsedTarget | null {
  const trimmed = line.trim()
  if (!trimmed) return null
  let name: string | undefined
  let addr = trimmed
  const sep = trimmed.indexOf('|')
  if (sep > 0) {
    name = trimmed.slice(0, sep).trim()
    addr = trimmed.slice(sep + 1).trim()
  }
  let host: string
  let port: number
  const idx = addr.lastIndexOf(':')
  if (idx <= 0) {
    // 未携带端口：host 或 名称|host，默认使用 80
    host = addr.trim()
    port = DEFAULT_PORT
  } else {
    host = addr.slice(0, idx).trim()
    const portStr = addr.slice(idx + 1).trim()
    if (!/^\d+$/.test(portStr)) return null
    port = Number(portStr)
    if (port < 1 || port > 65535) return null
  }
  if (!host) return null
  return { name: name || undefined, host, port }
}

const targets = computed<AddressTarget[]>(() =>
  text.value
    .split('\n')
    .map((line) => parseLine(line))
    .filter((t): t is ParsedTarget => t !== null)
    .map((t) => ({ ...t, protocol: protocol.value })),
)

const invalidLines = computed(() =>
  text.value
    .split('\n')
    .map((l) => l.trim())
    .filter((l) => l.length > 0 && !parseLine(l)),
)

const canTest = computed(
  () => targets.value.length > 0 && !testing.value && invalidLines.value.length === 0,
)

function latencyColor(ms: number): string {
  if (ms < 200) return 'text-green-500'
  if (ms < 500) return 'text-yellow-500'
  return 'text-orange-500'
}

async function doTest() {
  if (!canTest.value) return
  testing.value = true
  results.value = []
  try {
    const res = await addressLatencyTest(targets.value)
    results.value = res.results
    const ok = res.results.filter((r) => r.reachable).length
    toastSuccess('测试完成：' + ok + '/' + res.results.length + ' 可达')
  } catch (e) {
    toastError('测试失败: ' + (e instanceof Error ? e.message : String(e)))
  } finally {
    testing.value = false
  }
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <SignalIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">地址测速</h3>
    </div>
    <div class="px-5 pb-5 space-y-3">
      <p class="text-xs text-gray-500">
        对地址做 TCP 握手 / UDP 探针 / ICMP ping 测延迟。
      </p>

      <!-- 目标输入 -->
      <Input
        v-model="text"
        textarea
        :rows="5"
        placeholder="每行一个目标：host 或 host:port，可加名称前缀「名称|host」，端口缺省 80，如 主服|1.2.3.4"
      />
      <div v-if="invalidLines.length > 0" class="text-xs text-red-400">
        以下行格式无效：{{ invalidLines.join('；') }}
      </div>

      <!-- 协议选择 -->
      <div class="flex items-center gap-2 flex-wrap">
        <Button
          v-for="opt in PROTOCOL_OPTIONS"
          :key="opt.value"
          :type="protocol === opt.value ? 'primary' : 'outline'"
          size="small"
          @click="protocol = opt.value"
        >{{ opt.label }}</Button>
      </div>

      <!-- 操作按钮 -->
      <div class="flex items-center gap-3">
        <Button type="primary" :loading="testing" :disabled="!canTest" @click="doTest">
          <template #icon><BoltIcon class="h-4 w-4" /></template>
          {{ testing ? '测试中...' : '开始测试' }}
        </Button>
        <span v-if="targets.length > 0" class="text-xs text-gray-400">{{ targets.length }} 个目标</span>
      </div>

      <!-- 测试结果 -->
      <div v-if="results.length > 0" class="rounded-lg border border-gray-200 divide-y divide-gray-100">
        <div
          v-for="(item, idx) in results"
          :key="idx"
          class="flex items-center gap-3 px-3 py-2.5"
        >
          <CheckCircleIcon v-if="item.reachable" class="h-4 w-4 flex-none text-green-400" />
          <XCircleIcon v-else class="h-4 w-4 flex-none text-red-400" />
          <div class="flex-1 min-w-0">
            <div class="truncate text-sm text-gray-800">
              {{ item.name ? item.name + ' · ' : '' }}{{ item.host }}{{ item.protocol === 'ping' ? '' : ':' + item.port }}
            </div>
            <div v-if="item.error" class="text-xs text-red-400">{{ item.error }}</div>
            <div v-else class="text-xs text-gray-400">{{ item.protocol }} 探测成功</div>
          </div>
          <span
            v-if="item.reachable"
            class="flex-none text-sm font-medium"
            :class="latencyColor(item.latency_ms)"
          >{{ item.latency_ms }} ms</span>
          <span v-else class="flex-none text-sm font-medium text-red-400">失败</span>
        </div>
      </div>
    </div>
  </section>
</template>