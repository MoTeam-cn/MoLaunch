<script setup lang="ts">
/**
 * 参与者列表（房主面板子组件）
 *
 * 从 RoomHostPanel 拆出，纯展示 + emit 事件。
 * 接收参与者列表与连接状态查询函数，渲染为列表，每条提供踢出按钮。
 */
import Card from '@/components/common/Card.vue'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import { XCircleIcon } from '@heroicons/vue/24/outline'
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
  <Card title="参与者">
    <div class="divide-y divide-gray-100">
      <div
        v-for="p in participants"
        :key="p.participantId"
        class="px-1 py-2.5 flex items-center justify-between"
      >
        <div>
          <div class="text-xs font-medium text-gray-900">{{ p.devicePk.slice(0, 12) }}...</div>
          <div class="text-xs text-gray-500">
            {{ p.virtualIp }} · {{ p.status }} · {{ connStateText(p.participantId) }}
          </div>
        </div>
        <Tooltip text="踢出">
          <Button type="ghost" size="mini" @click="emit('kick', p.participantId, p.devicePk)">
            <template #icon><XCircleIcon class="w-3.5 h-3.5" /></template>
          </Button>
        </Tooltip>
      </div>
    </div>
  </Card>
</template>
