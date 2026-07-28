<script setup lang="ts">
/**
 * 封禁列表（房主面板子组件，阶段 6.2）
 *
 * 纯展示 + emit 事件。接收封禁列表与服务端时间，渲染为列表，每条提供解封按钮。
 * 封禁类型：
 * - bannedUntil = 0 → 永久封禁（红色标签）
 * - bannedUntil > serverTime → 临时封禁，显示剩余时长（橙色标签）
 */
import { computed } from 'vue'
import Card from '@/components/common/Card.vue'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import { ArrowPathIcon, LockOpenIcon } from '@heroicons/vue/24/outline'
import type { RoomBan } from '@/types/online'

const props = defineProps<{
  bans: RoomBan[]
  /** 服务端当前 Unix 秒（由 listBannedParticipants 返回） */
  serverTime: number
}>()

const emit = defineEmits<{
  unban: [devicePk: string]
  refresh: []
}>()

/** 格式化剩余封禁时长 */
function formatRemaining(ban: RoomBan): string {
  if (ban.bannedUntil === 0) return '永久'
  const remaining = ban.bannedUntil - props.serverTime
  if (remaining <= 0) return '已过期'
  const h = Math.floor(remaining / 3600)
  const m = Math.floor((remaining % 3600) / 60)
  return h > 0 ? `${h}小时${m}分钟` : `${m}分钟`
}

/** 是否永久封禁 */
function isPermanent(ban: RoomBan): boolean {
  return ban.bannedUntil === 0
}

/** 空状态：无封禁记录 */
const isEmpty = computed(() => props.bans.length === 0)
</script>

<template>
  <Card title="封禁列表">
    <template #extra>
      <Tooltip text="刷新封禁列表">
        <Button type="ghost" size="mini" @click="emit('refresh')">
          <template #icon><ArrowPathIcon class="w-3.5 h-3.5" /></template>
        </Button>
      </Tooltip>
    </template>

    <!-- 空状态：icon + text 垂直水平居中 -->
    <div
      v-if="isEmpty"
      class="flex flex-col items-center justify-center py-8 text-gray-400"
    >
      <LockOpenIcon class="w-8 h-8 mb-2" />
      <span class="text-xs">暂无封禁记录</span>
    </div>

    <!-- 封禁列表 -->
    <div v-else class="divide-y divide-gray-100">
      <div
        v-for="ban in bans"
        :key="ban.id"
        class="px-1 py-2.5 flex items-center justify-between"
      >
        <div class="min-w-0">
          <div class="text-xs font-medium text-gray-900 truncate">
            {{ ban.devicePk.slice(0, 12) }}...
          </div>
          <div class="flex items-center gap-1.5 mt-0.5">
            <span
              class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium"
              :class="isPermanent(ban)
                ? 'bg-red-50 text-red-600'
                : 'bg-amber-50 text-amber-600'"
            >
              {{ formatRemaining(ban) }}
            </span>
            <span class="text-[10px] text-gray-400">
              封禁于 {{ new Date(ban.createdAt * 1000).toLocaleString('zh-CN', { hour12: false }) }}
            </span>
          </div>
        </div>
        <Tooltip text="解封">
          <Button type="ghost" size="mini" @click="emit('unban', ban.devicePk)">
            <template #icon><LockOpenIcon class="w-3.5 h-3.5" /></template>
          </Button>
        </Tooltip>
      </div>
    </div>
  </Card>
</template>
