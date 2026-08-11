<script setup lang="ts">
/**
 * 待确认加入请求列表（房主面板子组件，置于「加入申请」抽屉内）
 *
 * 纯展示 + emit 事件，业务逻辑由父组件处理。
 * 接收待确认 Answer 列表，渲染为卡片列表，每条提供接受/拒绝按钮。
 */
import { computed } from 'vue'
import Button from '@/components/common/Button.vue'
import { ClockIcon } from '@heroicons/vue/24/outline'
import { formatTimestamp } from '@/utils/format'
import type { PendingAnswer } from '@/types/online'

const props = defineProps<{
  answers: PendingAnswer[]
}>()

const emit = defineEmits<{
  confirm: [answer: PendingAnswer, accepted: boolean]
}>()

/** 空状态：无待确认申请 */
const isEmpty = computed(() => props.answers.length === 0)
</script>

<template>
  <!-- 空状态：icon + text 垂直水平居中 -->
  <div
    v-if="isEmpty"
    class="flex flex-col items-center justify-center py-10 text-gray-400"
  >
    <ClockIcon class="w-8 h-8 mb-2" />
    <span class="text-xs">暂无待确认的加入申请</span>
  </div>

  <!-- 待确认申请列表 -->
  <div v-else class="space-y-2">
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
          <Button type="primary" size="mini" @click="emit('confirm', ans, true)">接受</Button>
          <Button type="ghost" size="mini" @click="emit('confirm', ans, false)">拒绝</Button>
        </div>
      </div>
      <div class="text-xs text-gray-400">加入时间: {{ formatTimestamp(ans.joinedAt) }}</div>
    </div>
  </div>
</template>
