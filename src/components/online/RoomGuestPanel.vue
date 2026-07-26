<script setup lang="ts">
/**
 * 加入方面板（阶段二）
 *
 * 显示内容：
 * - 房间信息卡片（房间码、自己的虚拟 IP、房主 MC 版本/端口）
 * - P2P 连接状态徽章
 * - MC 版本匹配提示（如房主版本与自己不同时提示）
 * - 退出房间按钮
 *
 * 加入方无需轮询 answers（房主会主动 confirm），
 * 仅在房间状态异常时由用户主动退出。
 */

import { computed, inject, onMounted } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useWebRTC } from '@/composables/useWebRTC'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import { showConfirm } from '@/utils/modal'
import { toastError } from '@/utils/toast'
import {
  XCircleIcon,
  ClockIcon,
  ServerStackIcon,
  WifiIcon,
  ExclamationTriangleIcon,
} from '@heroicons/vue/24/outline'

const store = useOnlineStore()
const guestWebrtc = inject('guestWebRTC') as ReturnType<typeof useWebRTC>

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
        guestWebrtc.close()
        await store.guestLeaveRoom()
      } catch (e) {
        toastError(`退出失败：${e instanceof Error ? e.message : String(e)}`)
        // 即使后端调用失败也清空本地状态
        store.resetRoomState()
      }
    },
  )
}

onMounted(() => {
  // 加入方拉取一次房间信息同步元数据
  void store.refreshRoomInfo()
})
</script>

<template>
  <div class="space-y-4">
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
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <WifiIcon class="w-4 h-4 text-gray-400" />
            <span>我的虚拟 IP</span>
          </div>
          <code class="text-xs text-gray-900 bg-gray-50 px-2 py-0.5 rounded">{{ room.selfVirtualIp }}</code>
        </div>
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
            <span>剩余时间</span>
          </div>
          <span class="text-xs" :class="remainingSeconds < 300 ? 'text-red-600' : 'text-gray-900'">
            {{ remainingText }}
          </span>
        </div>
      </div>
    </Card>

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
        <ExclamationTriangleIcon class="w-3.5 h-3.5 inline mr-1" />
        连接已建立，请在 Minecraft 中使用「多人联机 → 直接连接 → {{ room.hostVirtualIp || '房主虚拟 IP' }}」加入
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
