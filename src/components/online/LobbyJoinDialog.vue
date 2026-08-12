<script setup lang="ts">
/**
 * 大厅加入房间抽屉（联机大厅阶段 5.8）
 *
 * 有密码 / 关联整合包的房间加入前弹此抽屉（无密码无整合包的房间直接加入，不弹此窗）：
 * - 整合包：内嵌 ModpackRequirementCard 供校验/安装
 * - 密码：输入框，空密码内联提示
 *
 * 点「加入房间」执行 props.join（父组件负责加入，成功后 role 变化触发
 * Online.vue watch(isInRoom) 自动切到房间详情）：失败时抽屉保持打开、内联展示错误可重试；
 * 成功时收起抽屉，父组件在 @close 后卸载本组件。
 */
import { onMounted, ref } from 'vue'
import { CheckCircleIcon } from '@heroicons/vue/24/outline'
import Drawer from '@/components/common/Drawer.vue'
import ModpackRequirementCard from './ModpackRequirementCard.vue'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import type { LobbyRoomItem } from '@/types/online'

const props = defineProps<{
  room: LobbyRoomItem
  /** 执行加入（含密码）：成功 ok=true；失败 ok=false + error 内联展示 */
  join: (password: string) => Promise<{ ok: boolean; error?: string }>
}>()

const emit = defineEmits<{
  close: []
}>()

const visible = ref(false)
const password = ref('')
const loading = ref(false)
const errorMsg = ref('')

onMounted(() => {
  visible.value = true
})

/** 取消/遮罩/ESC：先播完关闭动画，@close 由 Drawer 在动画结束后触发（加入中不可取消） */
function handleCancel() {
  if (loading.value) return
  visible.value = false
}

/** 加入房间：空密码内联提示；失败内联展示错误保持打开，成功收起抽屉 */
async function handleConfirm() {
  if (loading.value) return
  if (props.room.hasPassword && !password.value.trim()) {
    errorMsg.value = '请输入房间密码'
    return
  }
  errorMsg.value = ''
  loading.value = true
  const res = await props.join(password.value.trim())
  loading.value = false
  if (!res.ok) {
    errorMsg.value = res.error || '加入房间失败'
    return
  }
  visible.value = false
}

/** Drawer 关闭动画结束后通知父组件卸载（v-if 移除） */
function onClosed() {
  emit('close')
}
</script>

<template>
  <Drawer
    :visible="visible"
    placement="right"
    :width="520"
    render-in-place
    popup-container="#app-content"
    @update:visible="handleCancel"
    @close="onClosed"
  >
    <template #title>
      <div class="flex items-center gap-1.5">
        <CheckCircleIcon class="h-4 w-4 text-primary-500" />
        <span>加入房间</span>
      </div>
    </template>

    <p class="text-sm text-gray-600">
      房间
      <code class="bg-gray-100 px-1.5 py-0.5 rounded text-gray-800">{{ room.roomCode }}</code>
      <template v-if="room.hasPassword">需要密码，请输入后加入</template>
      <template v-else>关联了整合包，请确认本地已安装</template>
    </p>

    <ModpackRequirementCard v-if="room.modpack" :modpack="room.modpack" class="mt-4" />

    <div v-if="room.hasPassword" class="mt-4">
      <label class="mb-1 block text-sm font-medium text-gray-700">房间密码</label>
      <Input
        v-model="password"
        type="password"
        class="w-full"
        placeholder="请输入房间密码"
        :disabled="loading"
        @keydown.enter="handleConfirm"
      />
    </div>

    <div v-if="errorMsg" class="mt-3 rounded-lg bg-red-50 p-3">
      <p class="text-sm text-red-600">{{ errorMsg }}</p>
    </div>

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button type="ghost" size="small" :disabled="loading" @click="handleCancel">取消</Button>
        <Button type="primary" size="small" :loading="loading" @click="handleConfirm">加入房间</Button>
      </div>
    </template>
  </Drawer>
</template>
