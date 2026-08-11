<script setup lang="ts">
/**
 * 参与者列表（房主面板子组件，置于「参与者」抽屉内）
 *
 * 纯展示 + emit 事件。
 * 接收参与者列表与连接状态查询函数，渲染为列表，每条提供踢出按钮。
 */
import Button from '@/components/common/Button.vue'
import { UsersIcon } from '@heroicons/vue/24/outline'
import type { ParticipantInfo } from '@/types/online'

defineProps<{
  participants: ParticipantInfo[]
  /** 获取参与者连接状态文本（由父组件代理查 hostMesh.getConnState） */
  connStateText: (participantId: string) => string
}>()

const emit = defineEmits<{
  kick: [participantId: string, devicePk: string]
}>()
</script>

<template>
  <TransitionGroup
    v-if="participants.length > 0"
    tag="div"
    name="participant"
    class="divide-y divide-gray-100"
  >
    <div
      v-for="p in participants"
      :key="p.participantId"
      class="participant-item px-1 py-2.5 flex items-center justify-between"
    >
      <div>
        <div class="text-xs font-medium text-gray-900">{{ p.devicePk.slice(0, 12) }}...</div>
        <div class="text-xs text-gray-500">
          {{ p.virtualIp }} · {{ p.status }} · {{ connStateText(p.participantId) }}
        </div>
      </div>
      <Button type="ghost" size="mini" @click="emit('kick', p.participantId, p.devicePk)">踢出</Button>
    </div>
  </TransitionGroup>
  <div v-else class="py-8 flex flex-col items-center justify-center gap-2 text-gray-400">
    <UsersIcon class="w-8 h-8" />
    <span class="text-xs">暂无参与者加入</span>
  </div>
</template>

<style scoped>
.participant-enter-active {
  transition: all 0.4s cubic-bezier(0.22, 1, 0.36, 1);
}
.participant-leave-active {
  transition: all 0.25s ease-in;
}
.participant-move {
  transition: transform 0.4s cubic-bezier(0.22, 1, 0.36, 1);
}
.participant-enter-from {
  opacity: 0;
  transform: translateY(-8px);
}
.participant-leave-to {
  opacity: 0;
  transform: translateX(16px);
}
</style>
