<script setup lang="ts">
/**
 * 待确认加入请求列表（房主面板子组件，置于「加入申请」抽屉内）
 *
 * 纯展示 + emit 事件，业务逻辑由父组件处理。
 * 接收待确认加入申请的参与者列表（status=joined/answered），
 * 渲染为卡片列表，每条提供接受/拒绝按钮。
 */
import { computed, defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
import { ClockIcon } from '@heroicons/vue/24/outline'
import { formatTimestamp } from '@/utils/format'
import type { ParticipantInfo } from '@/types/online'

const props = defineProps<{
  requests: ParticipantInfo[]
  /** 正在处理确认/拒绝的参与者集合（key=participantId），处理期间禁用对应按钮防连点 */
  busy?: Set<string>
}>()

const emit = defineEmits<{
  confirm: [request: ParticipantInfo, accepted: boolean]
}>()

/** 空状态：无待处理申请 */
const isEmpty = computed(() => props.requests.length === 0)
</script>

<template>
  <!-- 空状态：icon + text 垂直水平居中 -->
  <div
    v-if="isEmpty"
    class="flex flex-col items-center justify-center py-10 text-gray-400"
  >
    <ClockIcon class="w-8 h-8 mb-2" />
    <span class="text-xs">暂无待处理的加入申请</span>
  </div>

  <!-- 待确认申请列表 -->
  <div v-else class="space-y-2">
    <div
      v-for="req in requests"
      :key="req.participantId"
      class="p-3 bg-gray-50 rounded-lg"
    >
      <div class="flex items-center justify-between mb-2">
        <div>
          <div class="text-xs font-medium text-gray-900">{{ req.devicePk.slice(0, 12) }}...</div>
          <div class="text-xs text-gray-500">虚拟 IP: {{ req.virtualIp || '分配中' }}</div>
        </div>
        <div class="flex items-center gap-1">
          <Button type="primary" size="mini" :disabled="props.busy?.has(req.participantId)" @click="emit('confirm', req, true)">接受</Button>
          <Button type="ghost" size="mini" :disabled="props.busy?.has(req.participantId)" @click="emit('confirm', req, false)">拒绝</Button>
        </div>
      </div>
      <div class="text-xs text-gray-400">加入时间: {{ formatTimestamp(req.joinedAt) }}</div>
    </div>
  </div>
</template>
