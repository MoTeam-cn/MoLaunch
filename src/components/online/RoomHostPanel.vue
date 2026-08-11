<script setup lang="ts">
/**
 * 房主面板（阶段三 mesh 拓扑）
 *
 * 显示房间信息 + 待确认 Answer 列表 + 参与者列表 + P2P 状态 + 关闭按钮。
 *
 * 业务逻辑（信令轮询 / 自动 Offer / 确认 Answer / 踢出封禁 / 关闭房间）由
 * 全局联机会话 onlineSession 持有（App 级初始化，切页不断连），此处仅消费状态：
 * - pendingAnswers / bannedList / handleConfirm / handleKick 等来自会话
 * - WebRTC 实例通过 inject 获取（hostMesh，多 PC 管理器）
 * - TUN 桥接与数据分发由会话统一管理（onTunPacket 按角色路由）
 */

import { ref, computed, inject } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useWebRTCMesh } from '@/composables/useWebRTCMesh'
import { getOnlineSession } from '@/composables/online/onlineSession'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import Drawer from '@/components/common/Drawer.vue'
import Tag from '@/components/common/Tag.vue'
import AlertV2 from '@/components/common/AlertV2.vue'
import PendingAnswerList from './PendingAnswerList.vue'
import ParticipantList from './ParticipantList.vue'
import BannedList from './BannedList.vue'
import KickConfirmDialog from './KickConfirmDialog.vue'
import WhitelistEditor from './WhitelistEditor.vue'
import { copyToClipboard } from '@/utils/clipboard'
import {
  XCircleIcon,
  UsersIcon,
  ClockIcon,
  ServerStackIcon,
  WifiIcon,
  ClipboardDocumentIcon,
  ShieldCheckIcon,
  ArrowPathIcon,
} from '@heroicons/vue/24/outline'

const store = useOnlineStore()
const hostMesh = inject('hostMesh') as ReturnType<typeof useWebRTCMesh>

/** 房主业务逻辑来自全局联机会话（轮询/Offer/交互处理/TUN 均常驻应用生命周期） */
const {
  pendingAnswers,
  bannedList,
  banServerTime,
  handleConfirm,
  handleKick,
  handleUnban,
  refreshBans,
  handleCloseRoom,
} = getOnlineSession()

const room = computed(() => store.roomState)

/** 踢出确认弹窗状态（null=关闭，有值=正在选择封禁时长） */
const kickTarget = ref<{ participantId: string; devicePk: string; virtualIp?: string } | null>(null)

/** 列表抽屉开关：待确认申请 / 参与者 / 封禁列表（详情页仅保留入口按钮） */
const pendingDrawerOpen = ref(false)
const participantDrawerOpen = ref(false)
const banDrawerOpen = ref(false)

function onKick(participantId: string, devicePk: string) {
  const p = room.value.participants.find((x) => x.participantId === participantId)
  kickTarget.value = { participantId, devicePk, virtualIp: p?.virtualIp }
}

function onConfirmKick(banDuration: number | null) {
  const target = kickTarget.value
  kickTarget.value = null
  if (target) void handleKick(target.participantId, target.devicePk, banDuration)
}

function onCloseKick() {
  kickTarget.value = null
}
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

/** 获取参与者连接状态文本（用于 UI 显示） */
function participantStateText(participantId: string): string {
  return hostMesh.getConnState(participantId) ?? 'unknown'
}

/** 复制虚拟 IP 到剪贴板 */
async function copyVirtualIp() {
  const ip = room.value.selfVirtualIp
  if (!ip) return
  await copyToClipboard(ip, { toast: true })
}

/**
 * 当前总人数（含房主）
 */
const totalPlayers = computed(() => room.value.participants.length + 1)

/**
 * 是否接近人数上限（阶段三子任务 9 mesh 拓扑预警）
 *
 * 当总人数 >= maxPlayers - 1 时（即还差 1 人就满），显示橙色预警条，
 * 提示房主继续邀请可能导致上行带宽不足。
 * maxPlayers <= 2 时不预警（2 人房间本就最小单位）。
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

/** P2P 已联通操作指引（AlertV2 纯文本 message） */
const connectedHintMessage =
  '已联通，请在 Minecraft 内按 Esc → 「开放给局域网」开关。开放后启动器会自动捕获端口并广播给所有参与者，加入方在「多人游戏 → 直接连接」输入你的虚拟 IP 即可加入'
</script>

<template>
  <div class="space-y-4">
    <AlertV2 type="info" message="P2P联机对房主的网络质量要求较高，如遇连接不上可尝试更换房主" />
    <AlertV2 type="info" message="如遇到违法违规房间，请及时向我们举报" />
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
          <span class="text-xs text-gray-900">{{ room.hostMcVersion || '-' }} : {{ room.hostMcPort || '-' }}</span>
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
      <!-- 阶段三子任务 9：接近人数上限时显示 mesh 拓扑带宽预警 -->
      <AlertV2 v-if="nearPlayerLimit" type="warning" :message="nearPlayerLimitMessage" />
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
        <AlertV2 v-if="connectedCount > 0" type="info" :message="connectedHintMessage" />
      </div>
    </Card>

    <!-- 白名单管理（阶段三子任务 8 安全加强）：房主运行期增删 + 启用开关 -->
    <Card title="白名单管理">
      <template #extra>
        <div class="flex items-center gap-1 text-xs" :class="room.whitelistEnabled ? 'text-primary-600' : 'text-gray-400'">
          <ShieldCheckIcon class="w-3.5 h-3.5" />
          <span>{{ room.whitelistEnabled ? '已启用' : '未启用' }}</span>
        </div>
      </template>
      <WhitelistEditor mode="runtime" :room-code="room.roomCode" />
    </Card>

    <!-- 列表抽屉入口：详情页仅保留按钮，待确认申请 / 参与者 / 封禁列表全部收进抽屉 -->
    <Card title="房间管理">
      <div class="grid grid-cols-3 gap-2">
        <Button type="outline" size="small" @click="pendingDrawerOpen = true">
          <template #icon><ClockIcon class="w-3.5 h-3.5" /></template>
          <span class="flex items-center gap-1">
            加入申请
            <Tag v-if="pendingAnswers.length > 0" size="small" color="red">{{ pendingAnswers.length }}</Tag>
          </span>
        </Button>
        <Button type="outline" size="small" @click="participantDrawerOpen = true">
          <template #icon><UsersIcon class="w-3.5 h-3.5" /></template>
          <span class="flex items-center gap-1">
            参与者
            <Tag size="small">{{ room.participants.length }}</Tag>
          </span>
        </Button>
        <Button type="outline" size="small" @click="banDrawerOpen = true">
          <template #icon><ShieldCheckIcon class="w-3.5 h-3.5" /></template>
          <span class="flex items-center gap-1">
            封禁
            <Tag size="small">{{ bannedList.length }}</Tag>
          </span>
        </Button>
      </div>
    </Card>

    <!-- 待确认加入申请抽屉 -->
    <Drawer
      v-model:visible="pendingDrawerOpen"
      title="待确认加入请求"
      placement="right"
      :width="420"
      render-in-place
      popup-container="#app-content"
    >
      <PendingAnswerList :answers="pendingAnswers" @confirm="handleConfirm" />
    </Drawer>

    <!-- 参与者列表抽屉 -->
    <Drawer
      v-model:visible="participantDrawerOpen"
      title="参与者列表"
      placement="right"
      :width="420"
      render-in-place
      popup-container="#app-content"
    >
      <ParticipantList
        :participants="room.participants"
        :conn-state-text="participantStateText"
        @kick="onKick"
      />
    </Drawer>

    <!-- 封禁列表抽屉 -->
    <Drawer
      v-model:visible="banDrawerOpen"
      placement="right"
      :width="420"
      render-in-place
      popup-container="#app-content"
    >
      <template #title>
        <div class="flex items-center gap-1">
          <span>封禁列表</span>
          <Tooltip text="刷新封禁列表">
            <Button type="ghost" size="mini" @click="refreshBans">
              <template #icon><ArrowPathIcon class="w-3.5 h-3.5" /></template>
            </Button>
          </Tooltip>
        </div>
      </template>
      <BannedList :bans="bannedList" :server-time="banServerTime" @unban="handleUnban" />
    </Drawer>

    <!-- 踢出确认弹窗（选择封禁时长） -->
    <KickConfirmDialog
      v-if="kickTarget"
      :device-pk="kickTarget.devicePk"
      :virtual-ip="kickTarget.virtualIp"
      @close="onCloseKick"
      @confirm="onConfirmKick"
    />

    <div class="pt-2">
      <Button type="outline" long :loading="store.roomLoading" @click="handleCloseRoom">
        <template #icon><XCircleIcon class="w-4 h-4" /></template>
        关闭房间
      </Button>
    </div>
  </div>
</template>
