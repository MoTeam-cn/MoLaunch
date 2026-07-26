<script setup lang="ts">
/**
 * 联机房间管理（阶段二主控制器）
 *
 * 由父组件 [Online.vue](src/views/Online.vue) 通过 `mode` prop 指定当前模式：
 * - `mode='create'`：显示创建房间表单（role=null 时）或房主面板（role=host 时）
 * - `mode='join'`：显示加入房间表单（role=null 时）或加入方面板（role=guest 时）
 *
 * 已进入房间时（role=host/guest），无论 mode 如何都显示对应面板，
 * 保证用户在房间内切换子菜单不会丢失连接。
 *
 * WebRTC 实例归属：
 * - `hostWebrtc` 与 `guestWebrtc` 均在 setup 阶段创建，确保 onUnmounted 生效
 * - 大厅阶段调用 `hostWebrtc.createOffer` 或 `guestWebrtc.setRemoteOfferAndCreateAnswer`
 * - 进入房间后通过 `provide` 传递给对应面板复用（继续处理 setRemoteAnswer / 状态监听）
 */

import { ref, provide, computed } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useWebRTC } from '@/composables/useWebRTC'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import Input from '@/components/common/Input.vue'
import RoomHostPanel from './RoomHostPanel.vue'
import RoomGuestPanel from './RoomGuestPanel.vue'
import {
  PlusIcon,
  ArrowRightOnRectangleIcon,
  ArrowPathIcon,
  CheckCircleIcon,
} from '@heroicons/vue/24/outline'
import { submitAnswer } from '@/utils/api/online-manager'
import { toastError } from '@/utils/toast'

defineProps<{
  /** 当前模式：'create' = 创建房间，'join' = 加入房间 */
  mode: 'create' | 'join'
}>()

/** provide key：房主 / 加入方 WebRTC 实例（子面板 inject 复用同一 PC） */
const HOST_WEBRTC_KEY = 'hostWebRTC'
const GUEST_WEBRTC_KEY = 'guestWebRTC'

const store = useOnlineStore()
// 房主与加入方各一份独立 PC，避免角色切换时 PC 状态污染
const hostWebrtc = useWebRTC('host')
const guestWebrtc = useWebRTC('guest')

provide(HOST_WEBRTC_KEY, hostWebrtc)
provide(GUEST_WEBRTC_KEY, guestWebrtc)

/** 创建房间表单 */
const createForm = ref({
  maxPlayers: 4,
  password: '',
  mcVersion: '',
  mcPort: 25565,
})
/** 加入房间表单 */
const joinForm = ref({
  roomCode: '',
  password: '',
})

/** 房间码输入框提示：根据输入长度动态切换 default/error/success 三态 */
const roomCodeHint = computed(() => {
  const raw = joinForm.value.roomCode.trim()
  if (raw.length === 0) return '请输入 6 位房间码（数字 + 大写字母）'
  if (raw.length < 6) return `还需输入 ${6 - raw.length} 位`
  if (raw.length === 6) return '房间码格式正确'
  return '房间码不能超过 6 位'
})
const roomCodeHintType = computed<'default' | 'error' | 'success'>(() => {
  const len = joinForm.value.roomCode.trim().length
  if (len === 6) return 'success'
  if (len === 0) return 'default'
  return 'error'
})

/**
 * 创建房间步骤指示器
 *
 * - `stun`：获取 STUN 服务器列表
 * - `offer`：生成本地 SDP Offer + 收集 ICE 候选（最长 5 秒）
 * - `create`：调用后端创建房间
 *
 * UI 根据当前 step 高亮当前步骤、打勾已完成步骤、灰化未完成步骤。
 */
const createSteps = [
  { key: 'stun' as const, label: '获取 STUN 服务器' },
  { key: 'offer' as const, label: '生成本地连接信息' },
  { key: 'create' as const, label: '创建房间' },
]
const stepOrder = ['stun', 'offer', 'create'] as const
const currentStepIndex = computed(() => {
  if (!store.roomCreateStep) return -1
  return stepOrder.indexOf(store.roomCreateStep)
})

/** 房主创建房间 */
async function handleCreateRoom() {
  if (!createForm.value.mcVersion) {
    toastError('请填写 MC 版本：创建房间前需指明房主的 Minecraft 版本')
    return
  }
  if (createForm.value.mcPort <= 0 || createForm.value.mcPort > 65535) {
    toastError('MC 端口无效：端口范围 1-65535')
    return
  }
  if (createForm.value.maxPlayers < 2 || createForm.value.maxPlayers > 20) {
    toastError('人数无效：最大人数范围 2-20')
    return
  }

  try {
    store.roomCreateStep = 'stun'
    const stun = await store.fetchStunServers()

    store.roomCreateStep = 'offer'
    const { sdp, iceCandidates } = await hostWebrtc.createOffer(stun)

    store.roomCreateStep = 'create'
    await store.hostCreateRoom(
      sdp,
      iceCandidates,
      createForm.value.maxPlayers,
      createForm.value.password,
      createForm.value.mcVersion,
      createForm.value.mcPort,
      stun,
    )
  } catch (e) {
    toastError(`创建房间失败：${e instanceof Error ? e.message : String(e)}`)
  } finally {
    store.roomCreateStep = null
  }
}

/** 加入方加入房间 */
async function handleJoinRoom() {
  const code = joinForm.value.roomCode.trim().toUpperCase()
  if (code.length !== 6) {
    toastError('房间码格式错误：请输入 6 位房间码')
    return
  }
  try {
    const joinResp = await store.guestJoinRoom(code, joinForm.value.password)
    const { sdp, iceCandidates } = await guestWebrtc.setRemoteOfferAndCreateAnswer(
      joinResp.stunServers ?? [],
      joinResp.hostSdpOffer,
      joinResp.hostIceCandidates ?? [],
    )
    const result = await submitAnswer(code, joinResp.participantId, sdp, iceCandidates)
    if (result.code !== 1) {
      throw new Error(result.msg || '提交 Answer 失败')
    }
  } catch (e) {
    toastError(`加入房间失败：${e instanceof Error ? e.message : String(e)}`)
    store.resetRoomState()
    guestWebrtc.close()
  }
}
</script>

<template>
  <div class="space-y-4">
    <!-- 房主面板（role=host 时无论 mode 都显示） -->
    <RoomHostPanel v-if="store.roomState.role === 'host'" />

    <!-- 加入方面板（role=guest 时无论 mode 都显示） -->
    <RoomGuestPanel v-else-if="store.roomState.role === 'guest'" />

    <!-- 创建房间表单（mode=create 且未在房间） -->
    <Card v-else-if="mode === 'create'" title="创建房间">
      <div class="space-y-3 py-1">
        <div class="flex items-center gap-3">
          <label class="w-20 text-xs text-gray-600">MC 版本</label>
          <Input v-model="createForm.mcVersion" placeholder="如 1.20.1" />
        </div>
        <div class="flex items-center gap-3">
          <label class="w-20 text-xs text-gray-600">MC 端口</label>
          <Input v-model="createForm.mcPort" type="number" placeholder="25565" />
        </div>
        <div class="flex items-center gap-3">
          <label class="w-20 text-xs text-gray-600">最大人数</label>
          <Input v-model="createForm.maxPlayers" type="number" placeholder="4" />
        </div>
        <div class="flex items-center gap-3">
          <label class="w-20 text-xs text-gray-600">房间密码</label>
          <Input v-model="createForm.password" placeholder="留空表示无密码" />
        </div>
        <div class="pt-2">
          <Button type="primary" long :loading="store.roomLoading" @click="handleCreateRoom">
            <template #icon><PlusIcon class="w-4 h-4" /></template>
            创建房间
          </Button>
          <!-- 创建进度反馈：三步指示器（当前步高亮 + spinner，已完成打勾，未完成灰化） -->
          <div
            v-if="store.roomCreateStep"
            class="mt-2 space-y-1.5 px-3 py-2.5 bg-primary-50/50 rounded-lg border border-primary-100"
          >
            <div
              v-for="(step, idx) in createSteps"
              :key="step.key"
              class="flex items-center gap-2 text-xs transition-colors"
              :class="[
                idx === currentStepIndex
                  ? 'text-primary-700 font-medium'
                  : idx < currentStepIndex
                    ? 'text-green-600'
                    : 'text-gray-400',
              ]"
            >
              <ArrowPathIcon
                v-if="idx === currentStepIndex"
                class="w-3.5 h-3.5 animate-spin"
              />
              <CheckCircleIcon
                v-else-if="idx < currentStepIndex"
                class="w-3.5 h-3.5"
              />
              <span
                v-else
                class="w-3.5 h-3.5 rounded-full border border-current opacity-40"
              />
              <span>{{ step.label }}</span>
            </div>
          </div>
        </div>
      </div>
    </Card>

    <!-- 加入房间表单（mode=join 且未在房间） -->
    <Card v-else title="加入房间">
      <div class="space-y-3 py-1">
        <div class="flex items-start gap-3">
          <label class="w-20 text-xs text-gray-600 pt-2 shrink-0">房间码</label>
          <Input
            v-model="joinForm.roomCode"
            placeholder="6 位房间码"
            :maxlength="6"
            :hint="roomCodeHint"
            :hint-type="roomCodeHintType"
          />
        </div>
        <div class="flex items-center gap-3">
          <label class="w-20 text-xs text-gray-600">房间密码</label>
          <Input v-model="joinForm.password" placeholder="无密码留空" />
        </div>
        <div class="pt-2">
          <Button type="primary" long :loading="store.roomLoading" @click="handleJoinRoom">
            <template #icon><ArrowRightOnRectangleIcon class="w-4 h-4" /></template>
            加入房间
          </Button>
        </div>
      </div>
    </Card>
  </div>
</template>
