<script setup lang="ts">
/**
 * 服务器状态检测
 *
 * 输入 host + port，通过 SLP 协议（1.7+）获取服务器 MOTD / 在线人数 / 版本 / 延迟 / Favicon。
 * 纯 Rust TCP 实现，无需第三方 API。
 */
import { ref, defineAsyncComponent } from 'vue'
import {
  ServerStackIcon,
  BoltIcon,
  CheckCircleIcon,
  XCircleIcon,
  UsersIcon,
  ClockIcon,
  TagIcon,
  DocumentTextIcon,
  PaintBrushIcon,
} from '@heroicons/vue/24/outline'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
import { toastSuccess, toastError } from '@/utils/toast'
import { serverPing } from '@/utils/api/tools'
import type { ServerPingResult } from '@/utils/api/tools'
import { parseMcMotd } from '@/utils/motd'

const host = ref('')
const port = ref(25565)
const pinging = ref(false)
const result = ref<ServerPingResult | null>(null)
/** MOTD 显示模式：true=彩色（§ 解析），false=纯文本 */
const motdColored = ref(true)

/** 拆分 `host:port`（含 IPv6 方括号形式 `[::1]:25565`），非法端口返回 null */
function splitHostPort(v: string): { host: string; port: number } | null {
  const m = v.match(/^\[([^\]]+)\]:(\d{1,5})$/) || v.match(/^([^:]+):(\d{1,5})$/)
  if (!m) return null
  const port = parseInt(m[2] ?? m[4], 10)
  if (port < 1 || port > 65535) return null
  return { host: m[1] ?? m[3], port }
}

/** 粘贴 `host:port` 时自动拆分，端口填入端口框 */
function onHostPaste(e: ClipboardEvent) {
  const text = e.clipboardData?.getData('text') ?? ''
  const split = splitHostPort(text.trim())
  if (split) {
    e.preventDefault()
    host.value = split.host
    port.value = split.port
  }
}

async function doPing() {
  let h = host.value.trim()
  let p = port.value || 25565
  // 兜底：地址框内直接带端口（如手动输入 host:port）时同样拆分
  const split = splitHostPort(h)
  if (split) {
    h = split.host
    p = split.port
  }
  if (!h) return
  pinging.value = true
  result.value = null
  try {
    const res = await serverPing(h, p)
    result.value = res
    if (res.error) {
      toastError('检测失败：' + res.error)
    } else {
      toastSuccess('检测完成，延迟 ' + res.latency_ms + ' ms')
    }
  } catch (e) {
    toastError('检测失败: ' + (e instanceof Error ? e.message : String(e)))
  } finally {
    pinging.value = false
  }
}

function latencyColor(ms: number): string {
  if (ms < 50) return 'text-green-500'
  if (ms < 150) return 'text-yellow-500'
  if (ms < 300) return 'text-orange-500'
  return 'text-red-500'
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <ServerStackIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">服务器状态检测</h3>
    </div>
    <div class="px-5 pb-5 space-y-3">
      <p class="text-xs text-gray-500">
        通过 SLP 协议检测 Minecraft 服务器状态（MOTD、在线人数、版本、延迟、Favicon）。
      </p>

      <!-- 输入区 -->
      <div class="flex items-end gap-3">
        <div class="flex-1">
          <label class="mb-1 block text-xs font-medium text-gray-700">服务器地址</label>
          <Input v-model="host" placeholder="如 mc.hypixel.net" clearable @paste="onHostPaste" />
        </div>
        <div class="w-28">
          <label class="mb-1 block text-xs font-medium text-gray-700">端口</label>
          <Input
            :model-value="String(port)"
            type="number"
            placeholder="25565"
            @update:model-value="(v: string) => port = parseInt(v) || 25565"
          />
        </div>
        <Button type="primary" :loading="pinging" :disabled="!host.trim()" @click="doPing">
          <template #icon><BoltIcon class="h-4 w-4" /></template>
          {{ pinging ? '检测中...' : '检测' }}
        </Button>
      </div>

      <!-- 结果区 -->
      <div v-if="result" class="rounded-lg border border-gray-200 p-4 space-y-3">
        <!-- 成功/失败标识 -->
        <div class="flex items-center gap-2">
          <CheckCircleIcon v-if="!result.error" class="h-5 w-5 text-green-500" />
          <XCircleIcon v-else class="h-5 w-5 text-red-500" />
          <span v-if="!result.error" class="text-sm font-medium text-green-700">服务器在线</span>
          <span v-else class="text-sm font-medium text-red-600">连接失败</span>
        </div>

        <!-- 错误信息 -->
        <div v-if="result.error" class="rounded-lg bg-red-50 px-3 py-2 text-xs text-red-600">
          {{ result.error }}
        </div>

        <!-- 服务器信息 -->
        <template v-if="!result.error">
          <!-- Favicon + MOTD -->
          <div v-if="result.favicon || result.motd" class="rounded-lg bg-gray-50 px-3 py-2.5">
            <div class="flex items-start gap-3">
              <img
                v-if="result.favicon"
                :src="result.favicon"
                alt="favicon"
                class="h-12 w-12 flex-none rounded"
              />
              <div class="flex-1 min-w-0">
                <!-- eslint-disable vue/no-v-html -- parseMcMotd 先 HTML 转义再按 COLOR_MAP 白名单拼色值 -->
                <div
                  v-if="motdColored && result.motd_raw"
                  class="text-sm whitespace-pre-wrap break-words leading-relaxed"
                  v-html="parseMcMotd(result.motd_raw)"
                />
                <!-- eslint-enable vue/no-v-html -->
                <div v-else class="text-sm text-gray-800 whitespace-pre-wrap break-words leading-relaxed">
                  {{ result.motd || '（无 MOTD）' }}
                </div>
              </div>
              <!-- 彩色/纯文本切换 -->
              <Tooltip
                v-if="result.motd_raw && result.motd !== result.motd_raw"
                :text="motdColored ? '切换为纯文本' : '切换为彩色'"
              >
                <Button type="ghost" size="small" @click="motdColored = !motdColored">
                  <template #icon>
                    <component :is="motdColored ? DocumentTextIcon : PaintBrushIcon" class="h-4 w-4" />
                  </template>
                </Button>
              </Tooltip>
            </div>
          </div>

          <!-- 信息栏 -->
          <div class="grid grid-cols-3 gap-3">
            <div class="rounded-lg bg-gray-50 px-3 py-2">
              <div class="flex items-center gap-1.5 text-xs text-gray-400">
                <UsersIcon class="h-3.5 w-3.5" />在线人数
              </div>
              <div class="mt-0.5 text-sm font-medium text-gray-800">
                {{ result.online }} / {{ result.max }}
              </div>
            </div>
            <div class="rounded-lg bg-gray-50 px-3 py-2">
              <div class="flex items-center gap-1.5 text-xs text-gray-400">
                <TagIcon class="h-3.5 w-3.5" />版本
              </div>
              <div class="mt-0.5 text-sm font-medium text-gray-800 truncate">{{ result.version || '未知' }}</div>
            </div>
            <div class="rounded-lg bg-gray-50 px-3 py-2">
              <div class="flex items-center gap-1.5 text-xs text-gray-400">
                <ClockIcon class="h-3.5 w-3.5" />延迟
              </div>
              <div class="mt-0.5 text-sm font-medium" :class="latencyColor(result.latency_ms)">
                {{ result.latency_ms }} ms
              </div>
            </div>
          </div>
        </template>
      </div>
    </div>
  </section>
</template>
