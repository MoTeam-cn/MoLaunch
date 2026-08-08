<script setup lang="ts">
/**
 * 大厅加入确认弹窗（联机大厅阶段 5.7）
 *
 * 当大厅房间关联了整合包时，加入前先弹此确认窗，内嵌 ModpackRequirementCard
 * 供加入方校验本地是否已安装同款整合包，未安装时可一键安装。
 *
 * 无论校验结果如何，「加入房间」按钮始终可用（整合包非强制，用户可先加入再补装）。
 * 一键安装会跳转下载页，组件随后被父组件卸载，无需额外处理。
 */
import { onMounted, ref } from 'vue'
import { CheckCircleIcon } from '@heroicons/vue/24/outline'
import Drawer from '@/components/common/Drawer.vue'
import ModpackRequirementCard from './ModpackRequirementCard.vue'
import Button from '@/components/common/Button.vue'
import type { LobbyRoomItem } from '@/types/online'

defineProps<{
  room: LobbyRoomItem
}>()

const emit = defineEmits<{
  close: []
  confirm: []
}>()

const visible = ref(false)
onMounted(() => {
  visible.value = true
})

/** 关闭（取消按钮 / 遮罩 / ESC 统一走此路径，通知父组件卸载） */
function handleClose() {
  visible.value = false
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
    @update:visible="handleClose"
  >
    <template #title>
      <div class="flex items-center gap-1.5">
        <CheckCircleIcon class="h-4 w-4 text-primary-500" />
        <span>加入房间确认</span>
      </div>
    </template>

    <p class="text-sm text-gray-600">
      房间
      <code class="bg-gray-100 px-1.5 py-0.5 rounded text-gray-800">{{ room.roomCode }}</code>
      关联了整合包，请确认本地已安装：
    </p>
    <ModpackRequirementCard v-if="room.modpack" :modpack="room.modpack" class="mt-4" />

    <template #footer>
      <div class="flex justify-end gap-2">
        <Button type="ghost" size="small" @click="handleClose">取消</Button>
        <Button type="primary" size="small" @click="emit('confirm')">加入房间</Button>
      </div>
    </template>
  </Drawer>
</template>
