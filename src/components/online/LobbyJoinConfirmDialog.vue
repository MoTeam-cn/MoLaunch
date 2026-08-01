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
</script>

<template>
  <teleport to="body">
    <transition
      enter-active-class="transition ease-out duration-150"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition ease-in duration-100"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="visible"
        class="fixed inset-0 z-[10000] flex items-center justify-center p-4"
        @click.self="emit('close')"
      >
        <div class="absolute inset-0 bg-black/40" />
        <div class="relative w-full max-w-lg bg-white rounded-lg shadow-xl">
          <!-- 标题栏 -->
          <div class="flex items-center px-5 py-3.5 border-b border-gray-200">
            <h3 class="text-base font-semibold text-gray-900">加入房间确认</h3>
          </div>

          <!-- 内容区 -->
          <div class="px-5 py-4 space-y-3">
            <p class="text-sm text-gray-600">
              房间
              <code class="bg-gray-100 px-1.5 py-0.5 rounded text-gray-800">{{ room.roomCode }}</code>
              关联了整合包，请确认本地已安装：
            </p>
            <ModpackRequirementCard v-if="room.modpack" :modpack="room.modpack" />
          </div>

          <!-- 底部按钮栏 -->
          <div class="flex justify-end gap-2 px-5 py-3.5 bg-gray-50 rounded-b-lg">
            <Button type="ghost" size="small" @click="emit('close')">取消</Button>
            <Button type="primary" size="small" @click="emit('confirm')">加入房间</Button>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>
