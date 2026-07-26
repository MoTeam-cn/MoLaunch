<script setup lang="ts">
/**
 * 房主面板（阶段二）
 *
 * 显示房间信息 + 待确认 Answer 列表 + 参与者列表 + P2P 状态 + 关闭按钮。
 * 信令轮询：5s 拉 answers / 10s 拉参与者 / 5min keepalive。
 * WebRTC 实例通过 inject 从父级 RoomManager 获取（共享同一 PC）。
 */

import { computed, inject, onMounted, onUnmounted, ref } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useWebRTC } from '@/composables/useWebRTC'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import {
  listAnswers,
  confirmParticipant,
  kickParticipant,
} from '@/utils/api/online-manager'
import type { PendingAnswer } from '@/types/online'
import { showConfirm } from '@/utils/modal'
import { toastSuccess, toastError } from '@/utils/toast'
import { formatTimestamp } from '@/utils/format'
import {
  XCircleIcon,
  CheckCircleIcon,
  UsersIcon,
  ClockIcon,
  ServerStackIcon,
  WifiIcon,
} from '@heroicons/vue/24/outline'

const store = useOnlineStore()
const hostWebrtc = inject('hostWebRTC') as ReturnType<typeof useWebRTC>

const pendingAnswers = ref<PendingAnswer[]>([])
const polling = ref(false)
// 轮询失败时 toast 防刷屏：记录上次 toast 时间，30s 内不重复弹
const lastAnswerErrorToastAt = ref(0)
const ANSWER_ERROR_TOAST_INTERVAL = 30_000

const room = computed(() => store.roomState)
const connState = hostWebrtc.connectionState

const remainingSeconds = computed(() => {
  if (!room.value.expiresAt) return 0
  return Math.max(0, room.value.expiresAt - Math.floor(Date.now() / 1000))
})

const remainingText = computed(() => {
  const s = remainingSeconds.value
  if (s <= 0) return '已过期'
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  return h > 0 ? `${h}小时${m}分钟` : `${m}分钟`
})

/** 轮询待确认 Answer */
async function pollAnswers() {
  if (store.roomState.role !== 'host' || !store.roomState.roomCode || polling.value) return
  polling.value = true
  try {
    const result = await listAnswers(store.roomState.roomCode)
    if (result.code === 1 && result.data) {
      pendingAnswers.value = result.data.answers ?? []
      // 成功时重置防刷屏计时，下次失败可立即弹 toast
      lastAnswerErrorToastAt.value = 0
    } else {
      // 业务失败（如 1002 房间不存在 / 1004 仅房主可执行此操作）
      console.warn(
        `[Online] pollAnswers 业务失败: code=${result.code}, msg=${result.msg}, req_id=${result.req_id}`,
      )
      maybeToastError(`获取待确认 Answer 失败：${result.msg}`)
    }
  } catch (e) {
    console.warn('[Online] pollAnswers 异常:', e)
    maybeToastError(
      `获取待确认 Answer 异常：${e instanceof Error ? e.message : String(e)}`,
    )
  } finally {
    polling.value = false
  }
}

/** 30s 防刷屏 toast：避免 5s 轮询连续失败时刷屏 */
function maybeToastError(msg: string) {
  const now = Date.now()
  if (now - lastAnswerErrorToastAt.value < ANSWER_ERROR_TOAST_INTERVAL) return
  lastAnswerErrorToastAt.value = now
  toastError(msg)
}

/** 确认/拒绝参与者连接 */
async function handleConfirm(answer: PendingAnswer, accepted: boolean) {
  try {
    const result = await confirmParticipant(
      store.roomState.roomCode,
      answer.participantId,
      accepted,
    )
    if (result.code !== 1) throw new Error(result.msg || '确认操作失败')
    if (accepted) {
      await hostWebrtc.setRemoteAnswer(answer.sdpAnswer, answer.iceCandidates ?? [])
    }
    pendingAnswers.value = pendingAnswers.value.filter(
      (a) => a.participantId !== answer.participantId,
    )
    toastSuccess(accepted ? '已接受连接' : '已拒绝连接')
    await store.refreshParticipants()
  } catch (e) {
    toastError(`确认失败：${e instanceof Error ? e.message : String(e)}`)
  }
}

/** 踢出参与者（不封禁） */
function handleKick(participantId: string, devicePk: string) {
  showConfirm('踢出参与者', `确定踢出 ${devicePk.slice(0, 8)}...？`, async () => {
    try {
      const result = await kickParticipant(store.roomState.roomCode, participantId, null)
      if (result.code !== 1) throw new Error(result.msg || '踢出失败')
      toastSuccess('已踢出')
      await store.refreshParticipants()
    } catch (e) {
      toastError(`踢出失败：${e instanceof Error ? e.message : String(e)}`)
    }
  })
}

/** 房主保活 */
async function doKeepalive() {
  try {
    await store.keepalive()
  } catch (e) {
    console.warn('[Online] keepalive 失败:', e)
  }
}

/** 关闭房间 */
function handleCloseRoom() {
  showConfirm('关闭房间', '关闭后所有加入方将被断开连接，且无法恢复。确定关闭？', async () => {
    try {
      hostWebrtc.close()
      await store.hostCloseRoom()
    } catch (e) {
      toastError(`关闭失败：${e instanceof Error ? e.message : String(e)}`)
    }
  })
}

// 定时器句柄
let pollTimer: ReturnType<typeof setInterval> | null = null
let keepaliveTimer: ReturnType<typeof setInterval> | null = null
let participantsTimer: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  void pollAnswers()
  void store.refreshParticipants()
  void doKeepalive()
  pollTimer = setInterval(() => void pollAnswers(), 5000)
  keepaliveTimer = setInterval(() => void doKeepalive(), 5 * 60 * 1000)
  participantsTimer = setInterval(() => void store.refreshParticipants(), 10000)
})

onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer)
  if (keepaliveTimer) clearInterval(keepaliveTimer)
  if (participantsTimer) clearInterval(participantsTimer)
})
</script>

<template>
  <div class="space-y-4">
    <Card title="房间信息">
      <div class="divide-y divide-gray-100">
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ServerStackIcon class="w-4 h-4 text-gray-400" /><span>房间码</span>
          </div>
          <code class="text-base font-semibold text-primary-600 tracking-wider bg-primary-50 px-3 py-1 rounded">
            {{ room.roomCode }}
          </code>
        </div>
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <WifiIcon class="w-4 h-4 text-gray-400" /><span>虚拟 IP</span>
          </div>
          <code class="text-xs text-gray-900 bg-gray-50 px-2 py-0.5 rounded">{{ room.selfVirtualIp }}</code>
        </div>
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ServerStackIcon class="w-4 h-4 text-gray-400" /><span>MC 版本 / 端口</span>
          </div>
          <span class="text-xs text-gray-900">{{ room.hostMcVersion || '-' }} : {{ room.hostMcPort || '-' }}</span>
        </div>
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ClockIcon class="w-4 h-4 text-gray-400" /><span>剩余时间</span>
          </div>
          <span class="text-xs" :class="remainingSeconds < 300 ? 'text-red-600' : 'text-gray-900'">
            {{ remainingText }}
          </span>
        </div>
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <UsersIcon class="w-4 h-4 text-gray-400" /><span>人数</span>
          </div>
          <span class="text-xs text-gray-900">{{ room.participants.length + 1 }} / {{ room.maxPlayers }}</span>
        </div>
      </div>
    </Card>

    <Card title="P2P 连接">
      <div class="py-2 flex items-center justify-between">
        <span class="text-xs text-gray-500">WebRTC 状态</span>
        <span
          class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium"
          :class="{
            'bg-green-50 text-green-700': connState === 'connected',
            'bg-blue-50 text-blue-700': connState === 'connecting' || connState === 'new',
            'bg-yellow-50 text-yellow-700': connState === 'disconnected',
            'bg-red-50 text-red-700': connState === 'failed' || connState === 'closed',
          }"
        >{{ connState }}</span>
      </div>
    </Card>

    <Card v-if="pendingAnswers.length > 0" title="待确认加入请求">
      <div class="space-y-2 py-1">
        <div v-for="ans in pendingAnswers" :key="ans.participantId" class="p-3 bg-gray-50 rounded-lg">
          <div class="flex items-center justify-between mb-2">
            <div>
              <div class="text-xs font-medium text-gray-900">{{ ans.devicePk.slice(0, 12) }}...</div>
              <div class="text-xs text-gray-500">虚拟 IP: {{ ans.playerVirtualIp }}</div>
            </div>
            <div class="flex items-center gap-1">
              <Tooltip text="接受连接">
                <Button type="primary" size="mini" @click="handleConfirm(ans, true)">
                  <template #icon><CheckCircleIcon class="w-3.5 h-3.5" /></template>
                </Button>
              </Tooltip>
              <Tooltip text="拒绝连接">
                <Button type="ghost" size="mini" @click="handleConfirm(ans, false)">
                  <template #icon><XCircleIcon class="w-3.5 h-3.5" /></template>
                </Button>
              </Tooltip>
            </div>
          </div>
          <div class="text-xs text-gray-400">加入时间: {{ formatTimestamp(ans.joinedAt) }}</div>
        </div>
      </div>
    </Card>

    <Card v-if="room.participants.length > 0" title="参与者">
      <div class="divide-y divide-gray-100">
        <div v-for="p in room.participants" :key="p.participantId" class="px-1 py-2.5 flex items-center justify-between">
          <div>
            <div class="text-xs font-medium text-gray-900">{{ p.devicePk.slice(0, 12) }}...</div>
            <div class="text-xs text-gray-500">{{ p.virtualIp }} · {{ p.status }}</div>
          </div>
          <Tooltip text="踢出">
            <Button type="ghost" size="mini" @click="handleKick(p.participantId, p.devicePk)">
              <template #icon><XCircleIcon class="w-3.5 h-3.5" /></template>
            </Button>
          </Tooltip>
        </div>
      </div>
    </Card>

    <div class="pt-2">
      <Button type="outline" long :loading="store.roomLoading" @click="handleCloseRoom">
        <template #icon><XCircleIcon class="w-4 h-4" /></template>
        关闭房间
      </Button>
    </div>
  </div>
</template>
