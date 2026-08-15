<script setup lang="ts">
/**
 * 联机房间管理（Scaffolding 收敛版主控制器）
 *
 * 由父组件 [Online.vue](src/views/Online.vue) 通过 `mode` prop 指定当前模式：
 * - `mode='create'`：显示创建房间表单（role=null 时）或房主面板（role=host 时）
 * - `mode='join'`：显示加入房间表单（role=null 时）或加入方面板（role=guest 时）
 *
 * 已进入房间时（role=host/guest），无论 mode 如何都显示对应面板，
 * 保证用户在房间内切换子菜单不会丢失连接。
 *
 * 加入流程：输入房间码（N 段公开标识或完整 U/xxx 码）→ join 闸门拿完整码 →
 * 切到 RoomGuestPanel 由该面板自动探测进服地址（scaffolding_client_probe）。
 */
import { ref, computed, defineAsyncComponent } from 'vue'
import { useOnlineStore } from '@/stores/online'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const AlertV2 = defineAsyncComponent(() => import('@/components/common/AlertV2.vue'))
const RoomHostPanel = defineAsyncComponent(() => import('./RoomHostPanel.vue'))
const RoomGuestPanel = defineAsyncComponent(() => import('./RoomGuestPanel.vue'))
const CreateRoomForm = defineAsyncComponent(() => import('./CreateRoomForm.vue'))
import {
  ArrowRightOnRectangleIcon,
} from '@heroicons/vue/24/outline'
import { toastError } from '@/utils/toast'

defineProps<{
  /** 当前模式：'create' = 创建房间，'join' = 加入房间 */
  mode: 'create' | 'join'
}>()

const store = useOnlineStore()

/** 加入房间表单 */
const joinForm = ref({
  roomCode: '',
  password: '',
})

/** 房间码输入提示（支持 N 段公开标识或完整 U/xxx 码） */
const roomCodeHint = computed(() => {
  const raw = joinForm.value.roomCode.trim()
  if (!raw) return '请输入 N 段公开标识（如 YNZE-U61D）或完整 U/xxx 房间码'
  return '提交后经加入闸门校验，成功会自动组网并探测进服地址'
})

/** 加入房间（join 闸门返回完整码，探测由 RoomGuestPanel 挂载后自动执行） */
async function handleJoinRoom() {
  const code = joinForm.value.roomCode.trim()
  if (!code) {
    toastError('请输入房间码')
    return
  }
  try {
    await store.guestJoinRoom(code, joinForm.value.password)
  } catch (e) {
    toastError(`加入房间失败：${e instanceof Error ? e.message : String(e)}`)
    store.resetRoomState()
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
      <AlertV2 type="info" message="联机基于 easytier 虚拟局域网：凭房间码加入后自动组网，进入房间后即可看到房主开服的进服地址" />
      <AlertV2 type="info" message="如遇到违法违规房间，请及时向我们举报" />
      <Card title="加入房间">
        <div class="space-y-4 py-1">
          <div class="flex items-start gap-3">
            <label class="w-24 text-xs text-gray-600 pt-2 shrink-0">房间码</label>
            <Input
              v-model="joinForm.roomCode"
              placeholder="N 段公开标识或完整 U/xxx 码"
              :hint="roomCodeHint"
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
