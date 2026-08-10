<script setup lang="ts">
/**
 * 加入方面板（阶段二 + 阶段三子任务 5）
 *
 * 显示内容：
 * - 房间信息卡片（房间码、自己的虚拟 IP、房主 MC 版本/端口）
 * - P2P 连接状态徽章
 * - MC 版本匹配提示（如房主版本与自己不同时提示）
 * - 退出房间按钮
 *
 * 数据分发由全局联机会话 onlineSession 统一管理（TUN 桥接 / DataChannel 绑定 /
 * 密钥注入 / 房间状态监控均常驻应用生命周期，离开联机页不断连）。
 * 加入方无需轮询 answers（房主会主动 confirm），仅房间状态异常时主动退出。
 */

import { computed, inject, onMounted } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useWebRTC } from '@/composables/useWebRTC'
import { useGuestReconnect } from '@/composables/useRoomReconnect'
import { getOnlineSession } from '@/composables/online/onlineSession'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import AlertV2 from '@/components/common/AlertV2.vue'
import { showConfirm } from '@/utils/modal'
import { toastError } from '@/utils/toast'
import { copyToClipboard } from '@/utils/clipboard'
import {
  XCircleIcon,
  ClockIcon,
  ServerStackIcon,
  ExclamationTriangleIcon,
  ClipboardDocumentIcon,
} from '@heroicons/vue/24/outline'
import VirtualIpCard from './VirtualIpCard.vue'
import ModpackRequirementCard from './ModpackRequirementCard.vue'

const store = useOnlineStore()
const guestWebrtc = inject('guestWebrtc') as ReturnType<typeof useWebRTC>

/** 全局联机会话：退出房间清理 / TUN / 密钥注入均由会话统一管理 */
const session = getOnlineSession()
// 管理员提权重启恢复：存在待重连密码时挂载后自动重连（重建 WebRTC 与房间会话，TUN 同步重启）
useGuestReconnect(guestWebrtc, session.lan)

const room = computed(() => store.roomState)
const connState = guestWebrtc.connectionState

/** 距过期剩余时间（秒） */
const remainingSeconds = computed(() => {
  if (!room.value.expiresAt) return 0
  return Math.max(0, room.value.expiresAt - Math.floor(Date.now() / 1000))
})

const remainingText = computed(() => {
  const s = remainingSeconds.value
  if (s <= 0) return '已过期'
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  if (h > 0) return `${h}小时${m}分钟`
  return `${m}分钟`
})

/** 退出房间 */
function handleLeaveRoom() {
  showConfirm(
    '退出房间',
    '退出后将断开与房主的 P2P 连接。确定退出？',
    async () => {
      try {
        await session.guestLeaveAndCleanup()
      } catch (e) {
        toastError(`退出失败：${e instanceof Error ? e.message : String(e)}`)
        // 即使后端调用失败也清空本地状态
        store.resetRoomState()
      }
    },
  )
}

/** 复制文本到剪贴板 */
async function copyText(text: string) {
  if (!text) return
  await copyToClipboard(text, { toast: true })
}

onMounted(() => {
  // 加入方拉取一次房间信息同步元数据（TUN/密钥/房间状态监控由全局会话管理）
  void store.refreshRoomInfo()
})
</script>

<template>
  <div class="space-y-4">
    <AlertV2 type="info" message="P2P联机对房主的网络质量要求较高，如遇连接不上可尝试更换房主" />
    <AlertV2 type="info" message="如遇到违法违规房间，请及时向我们举报" />
    <!-- 房间信息 -->
    <Card title="房间信息">
      <div class="divide-y divide-gray-100">
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ServerStackIcon class="w-4 h-4 text-gray-400" />
            <span>房间码</span>
          </div>
          <code class="text-base font-semibold text-primary-600 tracking-wider bg-primary-50 px-3 py-1 rounded">
            {{ room.roomCode }}
          </code>
        </div>
        <VirtualIpCard :ip="room.selfVirtualIp" label="我的虚拟 IP" />
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ServerStackIcon class="w-4 h-4 text-gray-400" />
            <span>房主 MC 版本</span>
          </div>
          <span class="text-xs text-gray-900">{{ room.hostMcVersion || '-' }}</span>
        </div>
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ServerStackIcon class="w-4 h-4 text-gray-400" />
            <span>房主端口</span>
          </div>
          <span class="text-xs text-gray-900">{{ room.hostMcPort || '-' }}</span>
        </div>
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ClockIcon class="w-4 h-4 text-gray-400" />
            <Tooltip text="房间保留时间：若在此时间内无新玩家加入，房间将自动清退；正常游玩中的房间会自动续期保留，无需担心">
              <span>剩余时间</span>
            </Tooltip>
          </div>
          <span class="text-xs" :class="remainingSeconds < 300 ? 'text-red-600' : 'text-gray-900'">
            {{ remainingText }}
          </span>
        </div>
      </div>
    </Card>

    <!-- 整合包要求（联机大厅阶段 4：房主关联整合包时显示，自动校验本地是否已装同款） -->
    <ModpackRequirementCard v-if="room.hostModpack" :modpack="room.hostModpack" />

    <!-- P2P 连接状态 -->
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
        >
          {{ connState }}
        </span>
      </div>
      <div v-if="connState === 'connected'" class="mt-2 p-2 bg-green-50 rounded text-xs text-green-700">
        <div class="flex items-start gap-1.5">
          <ExclamationTriangleIcon class="w-3.5 h-3.5 mt-0.5 shrink-0" />
          <div class="flex-1">
            <div>连接已建立，请在 Minecraft 中「多人游戏 → 直接连接」输入房主虚拟 IP 加入</div>
            <div class="mt-1 flex items-center gap-1.5">
              <code class="bg-white px-1.5 py-0.5 rounded text-green-800 border border-green-200">
                {{ room.hostVirtualIp || '（等待房主广播）' }}
              </code>
              <Tooltip text="复制房主虚拟 IP">
                <Button
                  type="ghost"
                  size="mini"
                  :disabled="!room.hostVirtualIp"
                  @click="copyText(room.hostVirtualIp)"
                >
                  <template #icon><ClipboardDocumentIcon class="w-3.5 h-3.5" /></template>
                </Button>
              </Tooltip>
            </div>
          </div>
        </div>
      </div>
      <div v-else-if="connState === 'failed'" class="mt-2 p-2 bg-red-50 rounded text-xs text-red-700">
        <ExclamationTriangleIcon class="w-3.5 h-3.5 inline mr-1" />
        连接失败，可能是 NAT 兼容性问题。请检查网络环境后重试
      </div>
    </Card>

    <!-- 退出房间按钮 -->
    <div class="pt-2">
      <Button type="outline" long :loading="store.roomLoading" @click="handleLeaveRoom">
        <template #icon><XCircleIcon class="w-4 h-4" /></template>
        退出房间
      </Button>
    </div>
  </div>
</template>
