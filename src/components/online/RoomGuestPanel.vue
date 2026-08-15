<script setup lang="ts">
/**
 * 加入方面板（Scaffolding 收敛版）
 *
 * 进房后显示：
 * - 房间信息（N 段公开标识、房主 MC 版本/端口、备注）
 * - easytier 连接状态徽章（组网中 / 已组网 / 失败）
 * - 进服地址（房主虚拟 IP:MC 端口，可复制）+ 重新探测按钮
 * - 整合包要求（房主关联整合包时自动校验本地）
 * - 退出房间按钮（停 easytier + 清空本地状态）
 *
 * 挂载后自动探测进服地址（scaffolding_client_probe，join 闸门已拿完整码）。
 */
import { computed, onMounted, defineAsyncComponent } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { getOnlineSession } from '@/composables/online/onlineSession'
import { useEasyTier, type EasyTierStatus } from '@/composables/useEasyTier'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const AlertV2 = defineAsyncComponent(() => import('@/components/common/AlertV2.vue'))
const ModpackRequirementCard = defineAsyncComponent(() => import('./ModpackRequirementCard.vue'))
import { showConfirm } from '@/utils/modal'
import { toastError } from '@/utils/toast'
import { copyToClipboard } from '@/utils/clipboard'
import {
  XCircleIcon,
  ServerStackIcon,
  ExclamationTriangleIcon,
  ClipboardDocumentIcon,
  ArrowPathIcon,
} from '@heroicons/vue/24/outline'

const store = useOnlineStore()
const easytier = useEasyTier()
const session = getOnlineSession()

const room = computed(() => store.roomState)
/** 探测结果（mcIp/mcPort 由 scaffolding.probe 写入 store.easytierRuntime） */
const entry = computed(() => store.easytierRuntime)

const connStateText = computed(() => {
  switch (easytier.status.value) {
    case 'joined': return '已组网'
    case 'joining': return '组网中…'
    case 'error': return '组网失败'
    case 'stopping': return '断开中…'
    default: return '未组网'
  }
})

const connStateClass = computed(() => {
  switch (easytier.status.value as EasyTierStatus) {
    case 'joined': return 'bg-green-50 text-green-700'
    case 'joining': return 'bg-blue-50 text-blue-700'
    case 'error': return 'bg-red-50 text-red-700'
    case 'stopping': return 'bg-yellow-50 text-yellow-700'
    default: return 'bg-gray-50 text-gray-500'
  }
})

/** 进服地址（mcIp:mcPort） */
const entryAddress = computed(() => {
  if (!entry.value.mcIp || !entry.value.mcPort) return ''
  return `${entry.value.mcIp}:${entry.value.mcPort}`
})

/** 重新探测进服地址（scaffolding_client_probe） */
async function reProbe() {
  const res = await session.reconnect.reconnect()
  if (!res.ok) {
    toastError(`探测失败：${res.error ?? '未知错误'}`)
  }
}

/** 退出房间：停 easytier + 清空本地状态 */
function handleLeaveRoom() {
  showConfirm(
    '退出房间',
    '退出后将断开虚拟局域网连接。确定退出？',
    async () => {
      try {
        await easytier.stop()
      } catch (e) {
        toastError(`退出失败：${e instanceof Error ? e.message : String(e)}`)
      } finally {
        store.resetRoomState()
      }
    },
  )
}

async function copyText(text: string) {
  if (!text) return
  await copyToClipboard(text, { toast: true })
}

onMounted(() => {
  if (room.value.role === 'guest' && room.value.roomCode) {
    void reProbe()
  }
})
</script>

<template>
  <div class="space-y-4">
    <AlertV2 type="info" message="联机基于 easytier 虚拟局域网：组网成功后，在 Minecraft「多人游戏 → 直接连接」输入下方进服地址即可进入房主房间" />
    <AlertV2 type="info" message="如遇到违法违规房间，请及时向我们举报" />

    <!-- 房间信息 -->
    <Card title="房间信息">
      <div class="divide-y divide-gray-100">
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ServerStackIcon class="w-4 h-4 text-gray-400" />
            <span>房间标识</span>
          </div>
          <code class="text-sm font-semibold text-primary-600 tracking-wider bg-primary-50 px-2 py-1 rounded">
            {{ room.publicIdentifier }}
          </code>
        </div>
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ServerStackIcon class="w-4 h-4 text-gray-400" />
            <span>房主 MC 版本</span>
          </div>
          <span class="text-xs text-gray-900">{{ room.hostMcVersion || '-' }}</span>
        </div>
        <div v-if="room.remark" class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ServerStackIcon class="w-4 h-4 text-gray-400" />
            <span>房主备注</span>
          </div>
          <span class="text-xs text-gray-900 truncate max-w-[50%]">{{ room.remark }}</span>
        </div>
      </div>
    </Card>

    <!-- 整合包要求（房主关联整合包时显示，自动校验本地是否已装同款） -->
    <ModpackRequirementCard v-if="room.hostModpack" :modpack="room.hostModpack" />

    <!-- 连接状态 -->
    <Card title="连接状态">
      <div class="py-2 flex items-center justify-between">
        <span class="text-xs text-gray-500">easytier 虚拟网络</span>
        <span
          class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium"
          :class="connStateClass"
        >
          {{ connStateText }}
        </span>
      </div>
      <div v-if="easytier.error" class="mt-2 p-2 bg-red-50 rounded text-xs text-red-700 flex items-start gap-1.5">
        <ExclamationTriangleIcon class="w-3.5 h-3.5 mt-0.5 shrink-0" />
        <span>{{ easytier.error }}</span>
      </div>
    </Card>

    <!-- 进服地址 -->
    <Card title="进服地址">
      <div v-if="entryAddress" class="py-1 flex items-center justify-between gap-2">
        <div class="flex-1 min-w-0">
          <code class="text-base font-mono font-semibold text-green-700 bg-green-50 px-3 py-1.5 rounded block truncate">
            {{ entryAddress }}
          </code>
        </div>
        <Tooltip text="复制进服地址">
          <Button type="ghost" size="small" @click="copyText(entryAddress)">
            <template #icon><ClipboardDocumentIcon class="w-3.5 h-3.5" /></template>
            复制
          </Button>
        </Tooltip>
      </div>
      <div v-else class="py-2 text-xs text-gray-500">
        <div class="flex items-center gap-1.5">
          <ExclamationTriangleIcon class="w-3.5 h-3.5 text-yellow-500 shrink-0" />
          <span>尚未探测到房主 MC 服务，请确认房主已在游戏中开启「对局域网开放」后重新探测</span>
        </div>
      </div>
      <div class="mt-3 pt-3 border-t border-gray-100">
        <Button type="outline" long size="small" :loading="session.scaffolding.probing" @click="reProbe">
          <template #icon><ArrowPathIcon class="w-3.5 h-3.5" /></template>
          重新探测进服地址
        </Button>
      </div>
    </Card>

    <!-- 退出房间 -->
    <div class="pt-2">
      <Button type="outline" long :loading="store.roomLoading" @click="handleLeaveRoom">
        <template #icon><XCircleIcon class="w-4 h-4" /></template>
        退出房间
      </Button>
    </div>
  </div>
</template>
