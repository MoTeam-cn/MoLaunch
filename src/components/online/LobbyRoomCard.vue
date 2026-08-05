<script setup lang="ts">
/**
 * 大厅房间卡片（联机大厅阶段 5）
 *
 * 展示单个公开房间的摘要信息：房间码、加载器、MC 版本、人数、整合包摘要。
 * 点击「加入」按钮 emit join 事件，由父组件处理加入流程（密码弹窗 + joinRoom）。
 */
import { computed } from 'vue'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import Tag from '@/components/common/Tag.vue'
import { formatBytes } from '@/utils/format'
import type { LobbyRoomItem } from '@/types/online'
import {
  LockClosedIcon,
  UsersIcon,
  CubeIcon,
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

const statusLabel = computed(() => {
  switch (props.room.status) {
    case 'waiting': return '等待中'
    case 'active': return '游戏中'
    case 'closed': return '已关闭'
    default: return props.room.status
  }
})

const statusColor = computed(() => {
  switch (props.room.status) {
    case 'waiting': return 'bg-green-100 text-green-700'
    case 'active': return 'bg-blue-100 text-blue-700'
    default: return 'bg-gray-100 text-gray-500'
  }
})

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
</script>

<template>
  <div class="rounded-lg border border-gray-200 bg-white p-4 hover:border-primary-300 hover:shadow-sm transition-all">
    <!-- 第一行：房间码 + 状态 + 加载器 + MC 版本 -->
    <div class="flex items-center gap-2 flex-wrap">
      <code class="text-sm font-mono font-semibold text-gray-900 bg-gray-100 px-2 py-0.5 rounded">{{ room.roomCode }}</code>
      <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium" :class="statusColor">
        {{ statusLabel }}
      </span>
      <Tag v-if="loaderLabel" size="small" color="arcoblue">{{ loaderLabel }}</Tag>
      <span v-if="room.hostMcVersion" class="text-xs text-gray-500">MC {{ room.hostMcVersion }}</span>
      <Tooltip v-if="room.hasPassword" text="需要密码">
        <LockClosedIcon class="w-3.5 h-3.5 text-yellow-600" />
      </Tooltip>
    </div>

    <!-- 第二行：整合包摘要（有整合包时显示） -->
    <div v-if="room.modpack" class="mt-2.5 flex items-start gap-2">
      <CubeIcon class="w-4 h-4 text-gray-400 mt-0.5 shrink-0" />
      <div class="flex-1 min-w-0">
        <div class="text-sm text-gray-800 truncate">
          {{ room.modpack.name }}
          <span v-if="room.modpack.modpackVersion" class="text-gray-500 text-xs">{{ room.modpack.modpackVersion }}</span>
        </div>
        <div class="flex flex-wrap gap-x-3 gap-y-0.5 text-xs text-gray-500 mt-0.5">
          <span v-if="room.modpack.fileCount">{{ room.modpack.fileCount }} mods</span>
          <span v-if="room.modpack.fileSize">{{ formatBytes(room.modpack.fileSize) }}</span>
          <span class="capitalize">{{ room.modpack.source }}</span>
        </div>
      </div>
    </div>

    <!-- 第三行：人数 + 加入按钮 -->
    <div class="mt-3 flex items-center justify-between">
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
        <Button
          type="primary"
          size="small"
          disabled
        >
          <template #icon><ArrowRightOnRectangleIcon class="w-3.5 h-3.5" /></template>
          加入
        </Button>
      </Tooltip>
      <Button
        v-else
        type="primary"
        size="small"
        :loading="joining"
        :disabled="room.status === 'closed' || room.playerCount >= room.maxPlayers"
        @click="handleJoin"
      >
        <template #icon><ArrowRightOnRectangleIcon class="w-3.5 h-3.5" /></template>
        加入
      </Button>
    </div>
  </div>
</template>
