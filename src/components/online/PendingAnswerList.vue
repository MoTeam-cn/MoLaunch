<script setup lang="ts">
/**
 * 待确认加入请求列表（房主面板子组件）
 *
 * 从 RoomHostPanel 拆出，纯展示 + emit 事件，业务逻辑由父组件处理。
 * 接收待确认 Answer 列表，渲染为卡片列表，每条提供接受/拒绝按钮。
 */
import Card from '@/components/common/Card.vue'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import { CheckCircleIcon, XCircleIcon } from '@heroicons/vue/24/outline'
import { formatTimestamp } from '@/utils/format'
import type { PendingAnswer } from '@/types/online'

defineProps<{
  answers: PendingAnswer[]
}>()

const emit = defineEmits<{
  confirm: [answer: PendingAnswer, accepted: boolean]
}>()
</script>

<template>
  <Card title="待确认加入请求">
    <div class="space-y-2 py-1">
      <div
        v-for="ans in answers"
        :key="ans.participantId"
        class="p-3 bg-gray-50 rounded-lg"
      >
        <div class="flex items-center justify-between mb-2">
          <div>
            <div class="text-xs font-medium text-gray-900">{{ ans.devicePk.slice(0, 12) }}...</div>
            <div class="text-xs text-gray-500">虚拟 IP: {{ ans.playerVirtualIp }}</div>
          </div>
          <div class="flex items-center gap-1">
            <Tooltip text="接受连接">
              <Button type="primary" size="mini" @click="emit('confirm', ans, true)">
                <template #icon><CheckCircleIcon class="w-3.5 h-3.5" /></template>
              </Button>
            </Tooltip>
            <Tooltip text="拒绝连接">
              <Button type="ghost" size="mini" @click="emit('confirm', ans, false)">
                <template #icon><XCircleIcon class="w-3.5 h-3.5" /></template>
              </Button>
            </Tooltip>
          </div>
        </div>
        <div class="text-xs text-gray-400">加入时间: {{ formatTimestamp(ans.joinedAt) }}</div>
      </div>
    </div>
  </Card>
</template>
