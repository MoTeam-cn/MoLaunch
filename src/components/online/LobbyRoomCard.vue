<script setup lang="ts">
/**
 * 大厅房间摘要行（Scaffolding 收敛版）
 *
 * 展示单个公开房间的摘要：N 段公开标识、备注、人数、密码标记、MC 版本。
 * 点击「加入」按钮 emit join 事件，由父组件处理加入流程（密码弹窗 + joinRoom + probe）。
 */
import { computed, defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
import type { LobbyRoomItem } from '@/types/online'
import {
  LockClosedIcon,
  UsersIcon,
  ArrowRightOnRectangleIcon,
} from '@heroicons/vue/24/outline'

const props = defineProps<{
  room: LobbyRoomItem
  /** 加入中（禁用按钮防重复点击） */
  joining?: boolean
  /** 当前已在房间中（禁用加入按钮，需先退出/关闭当前房间） */
  inRoom?: boolean
}>()

const emit = defineEmits<{
  join: [room: LobbyRoomItem]
}>()

const loaderLabel = computed(() => {
  const map: Record<string, string> = {
    forge: 'Forge', fabric: 'Fabric', neoforge: 'NeoForge',
    quilt: 'Quilt', vanilla: '原版', release: '原版',
  }
  return props.room.hostLoader ? (map[props.room.hostLoader] ?? props.room.hostLoader) : ''
})

function handleJoin() {
  emit('join', props.room)
}

/** 房间已满时禁用加入按钮的原因（未禁用时为空字符串） */
const disabledReason = computed(() => {
  if (props.room.playerCount >= props.room.maxPlayers) return '该房间人数已满'
  return ''
})
</script>

<template>
  <div class="rounded-lg border border-gray-200 bg-white px-4 py-3 hover:border-primary-300 hover:shadow-sm transition-all">
    <div class="flex items-center gap-2 flex-wrap">
      <code class="text-sm font-mono font-semibold text-gray-900 bg-gray-100 px-2 py-0.5 rounded">{{ room.publicIdentifier }}</code>
      <Tag v-if="loaderLabel" size="small" color="arcoblue">{{ loaderLabel }}</Tag>
      <span v-if="room.hostMcVersion" class="text-xs text-gray-500">MC {{ room.hostMcVersion }}</span>
      <Tooltip v-if="room.hasPassword" text="需要密码">
        <LockClosedIcon class="w-3.5 h-3.5 text-yellow-600" />
      </Tooltip>
    </div>

    <div v-if="room.remark" class="mt-1.5 text-sm text-gray-600 truncate">{{ room.remark }}</div>

    <div class="mt-2.5 flex items-center justify-between">
      <div class="flex items-center gap-1.5 text-xs text-gray-600">
        <UsersIcon class="w-3.5 h-3.5" />
        <span>{{ room.playerCount }} / {{ room.maxPlayers }}</span>
      </div>
      <Tooltip
        v-if="inRoom"
        text="您当前在房间中哟，如果要加入 请先退出或者关闭房间"
        position="top"
        :delay="200"
      >
        <Button type="primary" size="small" disabled>
          <template #icon><ArrowRightOnRectangleIcon class="w-3.5 h-3.5" /></template>
          加入
        </Button>
      </Tooltip>
      <Tooltip v-else-if="disabledReason" :text="disabledReason" position="top" :delay="200">
        <Button type="primary" size="small" disabled>
          <template #icon><ArrowRightOnRectangleIcon class="w-3.5 h-3.5" /></template>
          加入
        </Button>
      </Tooltip>
      <Button
        v-else
        type="primary"
        size="small"
        :loading="joining"
        @click="handleJoin"
      >
        <template #icon><ArrowRightOnRectangleIcon class="w-3.5 h-3.5" /></template>
        加入
      </Button>
    </div>
  </div>
</template>
