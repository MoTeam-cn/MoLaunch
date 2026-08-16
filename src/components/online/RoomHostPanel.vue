<script setup lang="ts">
/**
 * 房主面板（Scaffolding 收敛版）
 *
 * 显示房间信息（HostRoomInfoCard）+ easytier 连接状态 + 关闭房间按钮。
 *
 * 创建编排由 CreateRoomForm 完成（登记 + hostStart），此处仅消费状态；
 * 关闭流程（停联机中心/easytier → room_close）由 useRoomHost().handleCloseRoom 一站式完成。
 */
import { onMounted, ref, defineAsyncComponent } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useRoomHost } from '@/composables/useRoomHost'
import { useTauriEvent } from '@/composables/useTauriEvent'
import { toastError, toastSuccess } from '@/utils/toast'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
const AlertV2 = defineAsyncComponent(() => import('@/components/common/AlertV2.vue'))
const HostRoomInfoCard = defineAsyncComponent(() => import('./HostRoomInfoCard.vue'))
const EasyTierStatusBadge = defineAsyncComponent(() => import('./EasyTierStatusBadge.vue'))
const RoomToolsDrawer = defineAsyncComponent(() => import('./RoomToolsDrawer.vue'))
import { XCircleIcon, ExclamationTriangleIcon, Cog6ToothIcon } from '@heroicons/vue/24/outline'

const store = useOnlineStore()
const roomHost = useRoomHost()

/** 房间工具抽屉开关（检查 MC 服务 / 网络连通性 / 端口自动检测） */
const toolsDrawerOpen = ref(false)

/** 手动指定端口输入与生效标记（最高权重，自动探测不再覆盖） */
const manualPortInput = ref('')
const manualActive = ref(false)

function handleCloseRoom() {
  void roomHost.handleCloseRoom()
}

async function applyManualPort() {
  const port = Number(manualPortInput.value)
  if (!Number.isInteger(port) || port <= 0 || port > 65535) {
    toastError('请输入有效的端口号（1-65535）')
    return
  }
  try {
    await roomHost.scaffolding.setMcPort(port)
    manualActive.value = true
    toastSuccess(`已手动指定端口 ${port}（最高权重，自动探测不再覆盖）`)
  } catch (e) {
    toastError(`设置失败：${e instanceof Error ? e.message : String(e)}`)
  }
}

async function clearManualPort() {
  try {
    await roomHost.scaffolding.setMcPort(null)
    manualActive.value = false
    manualPortInput.value = ''
    toastSuccess('已恢复端口自动探测')
  } catch (e) {
    toastError(`恢复失败：${e instanceof Error ? e.message : String(e)}`)
  }
}

/** 后端自动关房事件（MC 服务 30s 不可达）：清理房间登记 */
const autoCloseListener = useTauriEvent<{ reason: string; mcPort?: number }>(
  'scaffolding-host-auto-close',
  () => {
    toastError('MC 服务持续不可达，房间已自动关闭')
    handleCloseRoom()
  },
)

/** 后端 MC 端口变更事件（后台监视发现新端口） */
const portChangeListener = useTauriEvent<{ mcPort: number }>('scaffolding-mc-port-change', (p) => {
  if (p.mcPort && p.mcPort !== store.easytierRuntime.mcPort) {
    store.setEasyTierRuntime({ mcPort: p.mcPort })
    toastSuccess(`MC 端口已自动更新为 ${p.mcPort}`)
  }
})

onMounted(() => {
  void autoCloseListener.start()
  void portChangeListener.start()
})
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
        联机中心自动托管 MC 端口：游戏开放局域网后自动发现并更新进服端口，加入方组网后同步探测
      </div>
    </Card>

    <!-- MC 端口（手动最高权重，自动探测不覆盖） -->
    <Card title="MC 端口">
      <div class="py-1 flex items-center justify-between">
        <span class="text-xs text-gray-500">当前端口（自动探测）</span>
        <code class="text-sm font-mono font-semibold text-green-700 bg-green-50 px-2 py-0.5 rounded">
          {{ store.easytierRuntime.mcPort || '未探测到' }}
        </code>
      </div>
      <div v-if="manualActive" class="mt-1 flex items-center justify-between text-xs">
        <span class="text-gray-500">已手动指定端口（最高权重），自动探测已暂停</span>
        <Button type="ghost" size="small" @click="clearManualPort">恢复自动</Button>
      </div>
      <div class="mt-3 pt-3 border-t border-gray-100 flex items-center gap-2">
        <input
          v-model="manualPortInput"
          type="number"
          min="1"
          max="65535"
          placeholder="手动指定端口（最高权重）"
          class="flex-1 min-w-0 rounded-md border border-gray-300 px-2 py-1 text-sm"
          @keyup.enter="applyManualPort"
        />
        <Button type="outline" size="small" @click="applyManualPort">应用</Button>
      </div>
    </Card>

    <!-- 房间工具 -->
    <div class="pt-2">
      <Button type="outline" long @click="toolsDrawerOpen = true">
        <template #icon><Cog6ToothIcon class="w-4 h-4" /></template>
        房间工具
      </Button>
    </div>

    <!-- 关闭房间 -->
    <div class="pt-2">
      <Button type="outline" long :loading="store.roomLoading" @click="handleCloseRoom">
        <template #icon><XCircleIcon class="w-4 h-4" /></template>
        关闭房间
      </Button>
    </div>
  </div>

  <RoomToolsDrawer v-model:visible="toolsDrawerOpen" />
</template>
