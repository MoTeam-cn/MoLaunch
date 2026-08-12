<script setup lang="ts">
/**
 * 联机房间管理（阶段三 mesh 拓扑主控制器）
 *
 * 由父组件 [Online.vue](src/views/Online.vue) 通过 `mode` prop 指定当前模式：
 * - `mode='create'`：显示创建房间表单（role=null 时）或房主面板（role=host 时）
 * - `mode='join'`：显示加入房间表单（role=null 时）或加入方面板（role=guest 时）
 *
 * 已进入房间时（role=host/guest），无论 mode 如何都显示对应面板，
 * 保证用户在房间内切换子菜单不会丢失连接。
 *
 * WebRTC 实例归属（mesh 拓扑，房间挂起改造）：
 * - `hostMesh` / `guestWebrtc` 实例由 [Online.vue](src/views/Online.vue) 在页面级创建并 provide，
 *   本组件通过 inject 获取引用，**不在本地创建**。
 * - 切换侧边栏菜单（device ↔ create ↔ join）时 RoomManager 被 v-if 卸载，
 *   但 WebRTC 实例生命周期绑定在 Online.vue，不会触发 onUnmounted → close()，
 *   房间连接保持不断。仅离开联机页面时才销毁。
 * - 子面板 RoomHostPanel / RoomGuestPanel 同样 inject 获取，链路一致。
 */

import { ref, inject, computed } from 'vue'
import { useOnlineStore } from '@/stores/online'
import type { useWebRTC } from '@/composables/useWebRTC'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import Input from '@/components/common/Input.vue'
import AlertV2 from '@/components/common/AlertV2.vue'
import RoomHostPanel from './RoomHostPanel.vue'
import RoomGuestPanel from './RoomGuestPanel.vue'
import CreateRoomForm from './CreateRoomForm.vue'
import {
  ArrowRightOnRectangleIcon,
} from '@heroicons/vue/24/outline'
import { submitAnswer } from '@/utils/api/online-manager'
import { rememberJoinPassword } from '@/utils/relaunchSnapshot'
import { toastError } from '@/utils/toast'

defineProps<{
  /** 当前模式：'create' = 创建房间，'join' = 加入房间 */
  mode: 'create' | 'join'
}>()

const store = useOnlineStore()
// WebRTC 实例由 Online.vue 页面级 provide，本组件 inject 获取加入方实例引用。
// 切换侧边栏菜单时本组件卸载不会触发实例 close()，房间连接保持。
// 房主侧 hostMesh 由 RoomHostPanel 自行 inject，此处无需获取。
const guestWebrtc = inject<ReturnType<typeof useWebRTC>>('guestWebrtc')!

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

/** 加入方加入房间（mesh 拓扑：轮询房主为自己生成的 Offer → 创建 Answer → 提交） */
async function handleJoinRoom() {
  const code = joinForm.value.roomCode.trim().toUpperCase()
  if (code.length !== 6) {
    toastError('房间码格式错误：请输入 6 位房间码')
    return
  }
  try {
    const joinResp = await store.guestJoinRoom(code, joinForm.value.password)
    // 记住加入密码：提权重启后自动重连同一房间需要重新 join
    rememberJoinPassword(joinForm.value.password)
    // mesh 拓扑：房主为本参与者单独生成 Offer，需要轮询拉取
    // 首次连接仅用房间内 ICE 服务器（STUN + 自定义 TURN）尝试 P2P 直连，
    // 系统 TURN 留到直连失败（iceconnectionstatechange=failed）时再懒加载
    const iceServers = store.roomState.iceServers
    const { sdp, iceCandidates } = await guestWebrtc.fetchOfferAndAnswer(
      code,
      joinResp.participantId,
      iceServers,
    )
    const result = await submitAnswer(code, joinResp.participantId, sdp, iceCandidates)
    if (result.code !== 1) {
      throw new Error(result.msg || '提交 Answer 失败')
    }
  } catch (e) {
    toastError(`加入房间失败：${e instanceof Error ? e.message : String(e)}`)
    // 通知服务端删除参与者记录，避免僵尸数据导致大厅显示 2/4
    // API 失败不阻塞本地退出（服务端定期清理兜底）
    await store.guestLeaveRoom().catch(() => toastError('离开房间失败'))
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
    <CreateRoomForm v-else-if="mode === 'create'" />

    <!-- 加入房间表单（mode=join 且未在房间） -->
    <div v-else class="space-y-4">
      <AlertV2 type="info" message="P2P联机对房主的网络质量要求较高，如遇连接不上可尝试更换房主" />
      <AlertV2 type="info" message="如遇到违法违规房间，请及时向我们举报" />
      <Card title="加入房间">
      <div class="space-y-4 py-1">
        <div class="flex items-start gap-3">
          <label class="w-24 text-xs text-gray-600 pt-2 shrink-0">房间码</label>
          <Input
            v-model="joinForm.roomCode"
            placeholder="6 位房间码"
            :maxlength="6"
            :hint="roomCodeHint"
            :hint-type="roomCodeHintType"
          />
        </div>
        <div class="flex items-center gap-3">
          <label class="w-24 text-xs text-gray-600 shrink-0">房间密码</label>
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
  </div>
</template>
