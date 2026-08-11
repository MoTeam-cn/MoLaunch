<script setup lang="ts">
/**
 * 房主房间信息卡（房间码 / 虚拟 IP / MC 版本端口 / 剩余时间 / 人数）
 *
 * 端口区使用 HostMcPortEditor：自动捕获显示 + 手动指定（最高可信度），
 * 手动值经会话 setManualMcPort 广播给所有参与者。
 */
import { computed } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { getOnlineSession } from '@/composables/online/onlineSession'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import AlertV2 from '@/components/common/AlertV2.vue'
import HostMcPortEditor from './HostMcPortEditor.vue'
import { copyToClipboard } from '@/utils/clipboard'
import {
  ServerStackIcon,
  WifiIcon,
  ClockIcon,
  UsersIcon,
  ClipboardDocumentIcon,
} from '@heroicons/vue/24/outline'

const store = useOnlineStore()
const { setManualMcPort, clearManualMcPort } = getOnlineSession()
const room = computed(() => store.roomState)

/** 当前总人数（含房主） */
const totalPlayers = computed(() => room.value.participants.length + 1)

/**
 * 是否接近人数上限（mesh 拓扑预警）
 *
 * 总人数 >= maxPlayers - 1（还差 1 人就满）时显示橙色预警条；
 * maxPlayers <= 2 不预警（2 人房间本就最小单位）。
 */
const nearPlayerLimit = computed(
  () =>
    room.value.maxPlayers > 2 &&
    totalPlayers.value >= room.value.maxPlayers - 1 &&
    room.value.participants.length > 0,
)

/** 接近人数上限预警文案（AlertV2 纯文本 message） */
const nearPlayerLimitMessage = computed(
  () =>
    `接近人数上限（${totalPlayers.value}/${room.value.maxPlayers}），mesh 拓扑下房主上行带宽随人数线性增长，继续邀请可能出现卡顿，建议改用专业服务器`,
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

/** 复制虚拟 IP 到剪贴板 */
async function copyVirtualIp() {
  const ip = room.value.selfVirtualIp
  if (!ip) return
  await copyToClipboard(ip, { toast: true })
}
</script>

<template>
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
        <div class="flex items-center gap-1">
          <code class="text-xs text-gray-900 bg-gray-50 px-2 py-0.5 rounded">{{ room.selfVirtualIp }}</code>
          <Tooltip text="复制虚拟 IP">
            <Button type="ghost" size="mini" @click="copyVirtualIp">
              <template #icon><ClipboardDocumentIcon class="w-3.5 h-3.5" /></template>
            </Button>
          </Tooltip>
        </div>
      </div>
      <div class="px-1 py-3 flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm text-gray-600">
          <ServerStackIcon class="w-4 h-4 text-gray-400" /><span>MC 版本 / 端口</span>
        </div>
        <div class="flex items-center gap-1.5">
          <span class="text-xs text-gray-900">{{ room.hostMcVersion || '-' }}</span>
          <span class="text-gray-300">:</span>
          <HostMcPortEditor
            :value="room.hostMcPort"
            :manual="room.hostMcPortManual"
            @confirm="setManualMcPort"
            @clear="clearManualMcPort"
          />
        </div>
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
      <div class="px-1 py-3 flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm text-gray-600">
          <UsersIcon class="w-4 h-4 text-gray-400" /><span>人数</span>
        </div>
        <span class="text-xs text-gray-900">{{ totalPlayers }} / {{ room.maxPlayers }}</span>
      </div>
    </div>
    <AlertV2 v-if="nearPlayerLimit" type="warning" :message="nearPlayerLimitMessage" />
  </Card>
</template>
