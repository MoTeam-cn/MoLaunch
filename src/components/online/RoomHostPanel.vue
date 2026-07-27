<script setup lang="ts">
/**
 * 房主面板（阶段三 mesh 拓扑）
 *
 * 显示房间信息 + 待确认 Answer 列表 + 参与者列表 + P2P 状态 + 关闭按钮。
 *
 * 信令轮询（mesh 拓扑三路并行）：
 * - 5s 拉 participants：发现 `status='joined' && !hostOfferReady` 的参与者
 *   → 调用 hostMesh.createOfferFor → uploadParticipantOffer（per-participant Offer 生成）
 * - 5s 拉 answers：发现待确认 Answer → confirmParticipant(accepted=true) → hostMesh.setRemoteAnswer
 * - 5min keepalive
 *
 * WebRTC 实例通过 inject 从父级 RoomManager 获取（hostMesh，多 PC 管理器）。
 */

import { computed, inject, onMounted, onUnmounted, ref } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useWebRTCMesh } from '@/composables/useWebRTCMesh'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import {
  listAnswers,
  confirmParticipant,
  kickParticipant,
  uploadParticipantOffer,
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
const hostMesh = inject('hostMesh') as ReturnType<typeof useWebRTCMesh>

const pendingAnswers = ref<PendingAnswer[]>([])
const polling = ref(false)
// 正在为参与者生成 Offer 的集合，防止重复生成（key=participantId）
const offerGenerating = ref<Set<string>>(new Set())
// 轮询失败时 toast 防刷屏：记录上次 toast 时间，30s 内不重复弹
const lastAnswerErrorToastAt = ref(0)
const lastOfferErrorToastAt = ref(0)
const POLL_ERROR_TOAST_INTERVAL = 30_000

const room = computed(() => store.roomState)
const stunServers = computed(() => store.roomState.stunServers ?? [])
/** 已联通参与者数（channel open） */
const connectedCount = computed(() => hostMesh.connectedCount())
/** 已确认参与者数（status='confirmed'） */
const confirmedCount = computed(
  () => store.roomState.participants.filter((p) => p.status === 'confirmed').length,
)

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

/**
 * 为单个参与者生成 SDP Offer 并上传到后端
 *
 * 流程：hostMesh.createOfferFor → uploadParticipantOffer
 * 失败时 toast 提示但不阻塞其他参与者。
 */
async function generateOfferForParticipant(participantId: string) {
  if (offerGenerating.value.has(participantId)) return
  offerGenerating.value.add(participantId)
  try {
    const { sdp, iceCandidates } = await hostMesh.createOfferFor(participantId, stunServers.value)
    const result = await uploadParticipantOffer(
      store.roomState.roomCode,
      participantId,
      sdp,
      iceCandidates,
    )
    if (result.code !== 1) {
      throw new Error(result.msg || '上传 SDP Offer 失败')
    }
  } catch (e) {
    console.warn(`[Online] 为参与者 ${participantId} 生成 Offer 失败:`, e)
    maybeToastOfferError(
      `生成 SDP Offer 失败：${e instanceof Error ? e.message : String(e)}`,
    )
    // 生成失败时清理 PC，避免下次轮询跳过
    hostMesh.closeParticipant(participantId)
  } finally {
    offerGenerating.value.delete(participantId)
  }
}

/**
 * 扫描参与者列表，为 status='joined' && !hostOfferReady 的参与者生成 Offer
 *
 * 由 pollParticipants 调用，每次刷新参与者列表后触发。
 */
async function scanAndGenerateOffers() {
  const roomCode = store.roomState.roomCode
  if (!roomCode || store.roomState.role !== 'host') return
  const needOffer = store.roomState.participants.filter(
    (p) => p.status === 'joined' && !p.hostOfferReady,
  )
  // 并发生成（每个参与者独立 PC，互不干扰）
  await Promise.all(needOffer.map((p) => generateOfferForParticipant(p.participantId)))
}

/** 轮询参与者列表 + 触发 Offer 生成 */
async function pollParticipants() {
  if (store.roomState.role !== 'host' || !store.roomState.roomCode || polling.value) return
  polling.value = true
  try {
    await store.refreshParticipants()
    await scanAndGenerateOffers()
  } catch (e) {
    console.warn('[Online] pollParticipants 异常:', e)
  } finally {
    polling.value = false
  }
}

/** 轮询待确认 Answer */
async function pollAnswers() {
  if (store.roomState.role !== 'host' || !store.roomState.roomCode) return
  try {
    const result = await listAnswers(store.roomState.roomCode)
    if (result.code === 1 && result.data) {
      pendingAnswers.value = result.data.answers ?? []
      lastAnswerErrorToastAt.value = 0
    } else {
      console.warn(
        `[Online] pollAnswers 业务失败: code=${result.code}, msg=${result.msg}, req_id=${result.req_id}`,
      )
      maybeToastAnswerError(`获取待确认 Answer 失败：${result.msg}`)
    }
  } catch (e) {
    console.warn('[Online] pollAnswers 异常:', e)
    maybeToastAnswerError(
      `获取待确认 Answer 异常：${e instanceof Error ? e.message : String(e)}`,
    )
  }
}

/** 30s 防刷屏 toast：避免轮询连续失败时刷屏 */
function maybeToastAnswerError(msg: string) {
  const now = Date.now()
  if (now - lastAnswerErrorToastAt.value < POLL_ERROR_TOAST_INTERVAL) return
  lastAnswerErrorToastAt.value = now
  toastError(msg)
}
function maybeToastOfferError(msg: string) {
  const now = Date.now()
  if (now - lastOfferErrorToastAt.value < POLL_ERROR_TOAST_INTERVAL) return
  lastOfferErrorToastAt.value = now
  toastError(msg)
}

/**
 * 确认/拒绝参与者连接
 *
 * - 接受：confirmParticipant(true) → hostMesh.setRemoteAnswer(participantId, ...)
 * - 拒绝：confirmParticipant(false) → hostMesh.closeParticipant(participantId)
 */
async function handleConfirm(answer: PendingAnswer, accepted: boolean) {
  try {
    const result = await confirmParticipant(
      store.roomState.roomCode,
      answer.participantId,
      accepted,
    )
    if (result.code !== 1) throw new Error(result.msg || '确认操作失败')
    if (accepted) {
      await hostMesh.setRemoteAnswer(
        answer.participantId,
        answer.sdpAnswer,
        answer.iceCandidates ?? [],
      )
    } else {
      // 拒绝连接：关闭对应 PC 释放资源
      hostMesh.closeParticipant(answer.participantId)
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
      // 关闭对应 PC
      hostMesh.closeParticipant(participantId)
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
      hostMesh.close()
      await store.hostCloseRoom()
    } catch (e) {
      toastError(`关闭失败：${e instanceof Error ? e.message : String(e)}`)
    }
  })
}

/** 获取参与者连接状态文本（用于 UI 显示） */
function participantStateText(participantId: string): string {
  return hostMesh.getConnState(participantId) ?? 'unknown'
}

// 定时器句柄
let answerTimer: ReturnType<typeof setInterval> | null = null
let keepaliveTimer: ReturnType<typeof setInterval> | null = null
let participantsTimer: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  void pollParticipants()
  void pollAnswers()
  void doKeepalive()
  // 参与者轮询 5s（同时触发 Offer 生成）
  participantsTimer = setInterval(() => void pollParticipants(), 5000)
  // Answer 轮询 5s
  answerTimer = setInterval(() => void pollAnswers(), 5000)
  // 保活 5min
  keepaliveTimer = setInterval(() => void doKeepalive(), 5 * 60 * 1000)
})

onUnmounted(() => {
  if (answerTimer) clearInterval(answerTimer)
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
      <div class="py-2 space-y-2">
        <div class="flex items-center justify-between">
          <span class="text-xs text-gray-500">已联通 / 已确认</span>
          <span class="text-xs text-gray-900">{{ connectedCount }} / {{ confirmedCount }}</span>
        </div>
        <div class="flex items-center justify-between">
          <span class="text-xs text-gray-500">总参与者数</span>
          <span class="text-xs text-gray-900">{{ room.participants.length }}</span>
        </div>
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
            <div class="text-xs text-gray-500">{{ p.virtualIp }} · {{ p.status }} · {{ participantStateText(p.participantId) }}</div>
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
