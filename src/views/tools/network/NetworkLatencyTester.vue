<script setup lang="ts">
/**
 * 网络延迟测试
 *
 * 输入多个 URL（每行一个），并发测试 HTTP 延迟。
 * 提供官方源 / BMCLAPI 预设按钮，一键填充。
 */
import { ref, computed } from 'vue'
import {
  SignalIcon,
  BoltIcon,
  CheckCircleIcon,
  XCircleIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import { toastSuccess, toastError, toastInfo } from '@/utils/toast'
import { networkLatencyTest } from '@/utils/api/tools'
import type { LatencyItem } from '@/utils/api/tools'

const urlText = ref('')
const results = ref<LatencyItem[]>([])
const testing = ref(false)

const urls = computed(() =>
  urlText.value
    .split('\n')
    .map((u) => u.trim())
    .filter((u) => u.length > 0),
)

const canTest = computed(() => urls.value.length > 0 && !testing.value)

const PRESETS: Record<string, string[]> = {
  '官方源': [
    'https://piston-meta.mojang.com',
    'https://launchermeta.mojang.com',
    'https://libraries.minecraft.net',
  ],
  'BMCLAPI': [
    'https://bmclapi2.bangbang93.com',
    'https://bmclapi2.bangbang93.com/mc/game/version_manifest.json',
  ],
  'MCBBS': [
    'https://download.mcbbs.net',
    'https://download.mcbbs.net/mc/game/version_manifest.json',
  ],
}

function loadPreset(name: string) {
  const list = PRESETS[name]
  if (!list) return
  urlText.value = list.join('\n')
  toastInfo('已载入' + name + '预设（' + list.length + ' 个 URL）')
}

function latencyColor(ms: number | null): string {
  if (ms === null) return 'text-red-500'
  if (ms < 200) return 'text-green-500'
  if (ms < 500) return 'text-yellow-500'
  return 'text-orange-500'
}

async function doTest() {
  if (!canTest.value) return
  testing.value = true
  results.value = []
  try {
    const res = await networkLatencyTest(urls.value)
    results.value = res.results
    const ok = res.results.filter((r) => r.error === '').length
    toastSuccess('测试完成：' + ok + '/' + res.results.length + ' 成功')
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
      <h3 class="text-sm font-semibold text-gray-900">网络延迟测试</h3>
    </div>
    <div class="px-5 pb-5 space-y-3">
      <p class="text-xs text-gray-500">
        并发测试多个 URL 的 HTTP 延迟，用于选择最快的下载源。
      </p>

      <!-- 预设按钮 -->
      <div class="flex items-center gap-2">
        <span class="text-xs text-gray-500">预设：</span>
        <Button
          v-for="name in Object.keys(PRESETS)"
          :key="name"
          type="outline"
          size="small"
          @click="loadPreset(name)"
        >{{ name }}</Button>
      </div>

      <!-- URL 输入 -->
      <Input
        v-model="urlText"
        textarea
        :rows="5"
        placeholder="每行输入一个 URL，如 https://piston-meta.mojang.com"
      />

      <!-- 测试按钮 -->
      <div class="flex items-center gap-3">
        <Button type="primary" :loading="testing" :disabled="!canTest" @click="doTest">
          <template #icon><BoltIcon class="h-4 w-4" /></template>
          {{ testing ? '测试中...' : '开始测试' }}
        </Button>
        <span v-if="urls.length > 0" class="text-xs text-gray-400">{{ urls.length }} 个 URL</span>
      </div>

      <!-- 测试结果 -->
      <div v-if="results.length > 0" class="rounded-lg border border-gray-200 divide-y divide-gray-100">
        <div
          v-for="(item, idx) in results"
          :key="idx"
          class="flex items-center gap-3 px-3 py-2.5"
        >
          <CheckCircleIcon v-if="item.error === ''" class="h-4 w-4 flex-none text-green-400" />
          <XCircleIcon v-else class="h-4 w-4 flex-none text-red-400" />
          <div class="flex-1 min-w-0">
            <div class="truncate text-sm text-gray-800">{{ item.url }}</div>
            <div v-if="item.error" class="text-xs text-red-400">{{ item.error }}</div>
            <div v-else class="text-xs text-gray-400">HTTP {{ item.status_code }}</div>
          </div>
          <span
            v-if="item.latency_ms !== null"
            class="flex-none text-sm font-medium"
            :class="latencyColor(item.latency_ms)"
          >{{ item.latency_ms }} ms</span>
          <span v-else class="flex-none text-sm font-medium text-red-400">失败</span>
        </div>
      </div>
    </div>
  </section>
</template>
