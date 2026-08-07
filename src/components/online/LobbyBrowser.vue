<script setup lang="ts">
/**
 * 大厅浏览页（联机大厅阶段 5）
 *
 * 功能：
 * - 搜索框 + 加载器过滤 + 刷新
 * - 房间卡片列表（LobbyRoomCard）
 * - 空状态（icon + text 垂直水平居中）
 * - 分页（复用 community/Pagination）
 * - 加入房间流程（无密码直接加入，有密码弹 showPrompt 输入）
 *
 * 加入流程复用 Online.vue provide 的 guestWebrtc + store.guestJoinRoom，
 * 加入成功后 store.roomState.role 变化触发 Online.vue watch(isInRoom) 自动跳转房间详情。
 */
import { ref, computed, onMounted, inject } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useWebRTC } from '@/composables/useWebRTC'
import { listLobbyRooms } from '@/utils/api/online-manager'
import { resolveIceServers } from '@/utils/online/webrtc-helpers'
import { showPrompt } from '@/utils/modal'
import { toastError, toastInfo } from '@/utils/toast'
import Input from '@/components/common/Input.vue'
import Select from '@/components/common/Select.vue'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import Pagination from '@/components/community/Pagination.vue'
import LobbyRoomCard from './LobbyRoomCard.vue'
import LobbyJoinConfirmDialog from './LobbyJoinConfirmDialog.vue'
import type { LobbyRoomItem } from '@/types/online'
import {
  MagnifyingGlassIcon,
  ArrowPathIcon,
  ServerStackIcon,
} from '@heroicons/vue/24/outline'

const store = useOnlineStore()
const guestWebrtc = inject('guestWebrtc') as ReturnType<typeof useWebRTC>

/** 当前是否已在房间中（role !== null）。在房间中时禁用大厅加入按钮，需先退出/关闭当前房间 */
const isInRoom = computed(() => store.roomState.role !== null)

const rooms = ref<LobbyRoomItem[]>([])
const total = ref(0)
const page = ref(0) // 0-indexed，与 Pagination 组件一致
const pageSize = ref(20)
const loading = ref(false)
const joiningCode = ref('') // 当前正在加入的房间码（禁用重复点击）
const confirmRoom = ref<LobbyRoomItem | null>(null) // 加入确认弹窗中的房间（有整合包时先弹确认）

const keyword = ref('')
const loader = ref('') // 空=不过滤

const loaderOptions = [
  { label: '全部加载器', value: '' },
  { label: 'Forge', value: 'forge' },
  { label: 'Fabric', value: 'fabric' },
  { label: 'NeoForge', value: 'neoforge' },
  { label: 'Quilt', value: 'quilt' },
  { label: '原版', value: 'vanilla' },
]

/** 搜索防抖定时器 */
let searchTimer: ReturnType<typeof setTimeout> | null = null

async function fetchRooms(): Promise<boolean> {
  loading.value = true
  try {
    const result = await listLobbyRooms({
      page: page.value + 1, // 后端 1-indexed
      pageSize: pageSize.value,
      loader: loader.value || undefined,
      keyword: keyword.value.trim() || undefined,
    })
    if (result.code === 1 && result.data) {
      rooms.value = result.data.items
      total.value = result.data.total
      return true
    } else {
      toastError(result.msg || '获取大厅列表失败')
      rooms.value = []
      total.value = 0
      return false
    }
  } catch (e) {
    toastError(e instanceof Error ? e.message : String(e))
    rooms.value = []
    total.value = 0
    return false
  } finally {
    loading.value = false
  }
}

/** 手动刷新（仅点击刷新按钮时调用）：成功时提示，自动加载/搜索/翻页不提示 */
async function handleManualRefresh() {
  const ok = await fetchRooms()
  if (ok) toastInfo('已刷新房间列表')
}

function onSearchInput() {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    page.value = 0
    void fetchRooms()
  }, 400)
}

function onLoaderChange() {
  page.value = 0
  void fetchRooms()
}

function onPageChange(p: number) {
  page.value = p
  void fetchRooms()
}

async function handleJoin(room: LobbyRoomItem) {
  // 兜底校验：按钮已 disabled，但防止未来代码变更绕过
  if (isInRoom.value) {
    toastInfo('您当前在房间中哟，如果要加入 请先退出或者关闭房间')
    return
  }
  // 房间关联了整合包时先弹确认窗，供加入方校验/安装整合包
  if (room.modpack) {
    confirmRoom.value = room
    return
  }
  await proceedJoin(room.roomCode, room.hasPassword)
}

/** 弹窗确认后或无整合包时直接走密码/加入流程 */
async function proceedJoin(roomCode: string, hasPassword: boolean) {
  if (hasPassword) {
    showPrompt('加入房间', `房间 ${roomCode} 需要密码，请输入：`, (password) => {
      void doJoin(roomCode, password)
    }, { placeholder: '房间密码' })
  } else {
    await doJoin(roomCode, '')
  }
}

function onConfirmJoin() {
  const room = confirmRoom.value
  confirmRoom.value = null
  if (room) void proceedJoin(room.roomCode, room.hasPassword)
}

function onCloseConfirm() {
  confirmRoom.value = null
}

async function doJoin(roomCode: string, password: string) {
  joiningCode.value = roomCode
  try {
    const joinResp = await store.guestJoinRoom(roomCode, password)
    const iceServers = resolveIceServers(joinResp.iceServers, joinResp.stunServers)
    await guestWebrtc.fetchOfferAndAnswer(roomCode, joinResp.participantId, iceServers)
    // 加入成功后 store.roomState.role='guest'，Online.vue watch(isInRoom) 自动跳转房间详情
  } catch (e) {
    toastError(e instanceof Error ? e.message : String(e))
  } finally {
    joiningCode.value = ''
  }
}

onMounted(() => {
  void fetchRooms()
})
</script>

<template>
  <div class="space-y-4">
    <!-- 搜索栏 -->
    <div class="flex items-center gap-2">
      <Input
        v-model="keyword"
        placeholder="搜索房间码 / 整合包名称"
        width="320px"
        @input="onSearchInput"
      >
        <template #prefix><MagnifyingGlassIcon class="w-4 h-4 text-gray-400" /></template>
      </Input>
      <Select v-model="loader" :options="loaderOptions" style="width: 180px" @update:model-value="onLoaderChange" />
      <Tooltip text="刷新列表" class="ml-auto">
        <Button type="ghost" size="small" :loading="loading" @click="handleManualRefresh">
          <template #icon><ArrowPathIcon class="w-4 h-4" /></template>
        </Button>
      </Tooltip>
    </div>

    <!-- 加载中 -->
    <div v-if="loading && rooms.length === 0" class="flex items-center justify-center py-12 text-gray-400">
      <ArrowPathIcon class="w-5 h-5 animate-spin mr-2" />
      <span class="text-sm">正在加载联机大厅列表...</span>
    </div>

    <!-- 空状态 -->
    <div
      v-else-if="rooms.length === 0"
      class="flex flex-col items-center justify-center py-16 text-gray-400"
    >
      <ServerStackIcon class="w-10 h-10 mb-3 text-gray-300" />
      <span class="text-sm">暂无公开房间，去创建一个吧</span>
    </div>

    <!-- 房间列表 -->
    <div v-else class="space-y-3">
      <LobbyRoomCard
        v-for="room in rooms"
        :key="room.roomCode"
        :room="room"
        :joining="joiningCode === room.roomCode"
        :in-room="isInRoom"
        @join="handleJoin"
      />
    </div>

    <!-- 分页 -->
    <Pagination
      v-if="total > pageSize"
      :page="page"
      :total="total"
      :page-size="pageSize"
      @change="onPageChange"
    />

    <!-- 加入确认弹窗（房间有整合包时先弹此窗） -->
    <LobbyJoinConfirmDialog
      v-if="confirmRoom"
      :room="confirmRoom"
      @close="onCloseConfirm"
      @confirm="onConfirmJoin"
    />
  </div>
</template>
