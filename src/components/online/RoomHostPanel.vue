<script setup lang="ts">
/**
 * 房主面板（Scaffolding 收敛版）
 *
 * 显示房间信息（HostRoomInfoCard）+ easytier 连接状态 + 关闭房间按钮。
 *
 * 创建编排由 CreateRoomForm 完成（登记 + hostStart），此处仅消费状态；
 * 关闭流程（停联机中心/easytier → room_close）由 useRoomHost().handleCloseRoom 一站式完成。
 */
import { computed, defineAsyncComponent } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useRoomHost } from '@/composables/useRoomHost'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
const AlertV2 = defineAsyncComponent(() => import('@/components/common/AlertV2.vue'))
const HostRoomInfoCard = defineAsyncComponent(() => import('./HostRoomInfoCard.vue'))
const EasyTierStatusBadge = defineAsyncComponent(() => import('./EasyTierStatusBadge.vue'))
import { XCircleIcon, ExclamationTriangleIcon } from '@heroicons/vue/24/outline'

const store = useOnlineStore()
const roomHost = useRoomHost()

const room = computed(() => store.roomState)

function handleCloseRoom() {
  void roomHost.handleCloseRoom()
}
</script>

<template>
  <div class="space-y-4">
    <AlertV2 type="info" message="联机基于 easytier 虚拟局域网：请确认已在游戏中开启「对局域网开放」，联机中心会自动托管 MC 端口" />
    <AlertV2 type="info" message="如遇到违法违规房间，请及时向我们举报" />

    <HostRoomInfoCard />

    <!-- 连接状态 -->
    <Card title="连接状态">
      <div class="py-2 flex items-center justify-between">
        <span class="text-xs text-gray-500">easytier 虚拟网络</span>
        <EasyTierStatusBadge />
      </div>
      <div v-if="roomHost.easytier.error.value" class="mt-2 p-2 bg-red-50 rounded text-xs text-red-700 flex items-start gap-1.5">
        <ExclamationTriangleIcon class="w-3.5 h-3.5 mt-0.5 shrink-0" />
        <span>{{ roomHost.easytier.error.value }}</span>
      </div>
      <div class="mt-3 pt-3 border-t border-gray-100 text-xs text-gray-500">
        房主 MC 端口由联机中心自动托管（scaffolding-mc-server-{{ room.hostMcPort }}），加入方组网后自动探测进服地址
      </div>
    </Card>

    <!-- 关闭房间 -->
    <div class="pt-2">
      <Button type="outline" long :loading="store.roomLoading" @click="handleCloseRoom">
        <template #icon><XCircleIcon class="w-4 h-4" /></template>
        关闭房间
      </Button>
    </div>
  </div>
</template>
