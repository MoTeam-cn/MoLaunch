<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
/**
 * 参与者列表（房主面板子组件，置于「参与者」抽屉内）
 *
 * 纯展示 + emit 事件。
 * 接收参与者列表与连接状态查询函数，渲染为列表，每条提供踢出按钮。
 */
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
import { UsersIcon } from '@heroicons/vue/24/outline'
import type { ParticipantInfo } from '@/types/online'
import { resolveNatMeta, getNatFeasibilityColorClass } from '@/utils/online/nat-type'

defineProps<{
  participants: ParticipantInfo[]
  /** 获取参与者连接状态文本（由父组件代理查 hostMesh.getConnState） */
  connStateText: (participantId: string) => string
  /** 获取参与者 NAT 类型（由父组件代理查 participantNatTypes，未上报返回 null） */
  natTypeOf: (participantId: string) => string | null
}>()

const emit = defineEmits<{
  kick: [participantId: string, devicePk: string]
}>()

/** NAT 类型展示（未知回退原始字符串/未获取） */
function natBadgeText(natType: string | null) {
  return resolveNatMeta(natType)?.label ?? (natType || '未获取')
}

/** NAT 徽章配色 */
function natBadgeClass(natType: string | null) {
  const meta = resolveNatMeta(natType)
  return meta ? getNatFeasibilityColorClass(meta.feasibility) : 'bg-gray-100 text-gray-600'
}
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
        <div class="mt-0.5 flex items-center gap-1.5 text-xs text-gray-500">
          <span>{{ p.virtualIp }} · {{ p.status }} · {{ connStateText(p.participantId) }}</span>
          <span
            class="inline-flex items-center px-1.5 py-0.5 rounded-full text-[11px] font-medium"
            :class="natBadgeClass(natTypeOf(p.participantId))"
          >
            {{ natBadgeText(natTypeOf(p.participantId)) }}
          </span>
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
