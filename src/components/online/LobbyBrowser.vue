<script setup lang="ts">
/**
 * 联机大厅（Scaffolding 收敛版）
 *
 * 按整合包聚类的卡片（热度排序），点击展开该整合包下的公开房间摘要列表；
 * 加入房间：无密码直接进房，有密码弹 LobbyJoinDialog；进房后由 RoomManager 切到房客面板。
 */
import { ref, computed, onMounted, defineAsyncComponent } from 'vue'
import {
  CubeIcon,
  ChevronDownIcon,
  ArrowPathIcon,
  UserGroupIcon,
  Squares2X2Icon,
} from '@heroicons/vue/24/outline'
import { listLobbyPackages, listLobbyRooms } from '@/utils/api/online-manager'
import type { LobbyPackageItem, LobbyRoomItem } from '@/types/online'
import { useOnlineStore } from '@/stores/online'
import { toastError } from '@/utils/toast'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const AlertV2 = defineAsyncComponent(() => import('@/components/common/AlertV2.vue'))
const LobbyRoomCard = defineAsyncComponent(() => import('./LobbyRoomCard.vue'))
const LobbyJoinDialog = defineAsyncComponent(() => import('./LobbyJoinDialog.vue'))

const store = useOnlineStore()

const packages = ref<LobbyPackageItem[]>([])
const packagesLoading = ref(false)
const expandedId = ref<string | null>(null)
const roomsLoading = ref(false)
const rooms = ref<LobbyRoomItem[]>([])
/** 正在加入的房间（LobbyRoomCard joining 标记） */
const joiningId = ref<string | null>(null)
/** 密码弹窗目标房间 */
const joinTarget = ref<LobbyRoomItem | null>(null)

/** 是否已在房间中（房主/房客均禁止再加入） */
const inRoom = computed(() => store.roomState.role !== null)

async function loadPackages() {
  packagesLoading.value = true
  try {
    const res = await listLobbyPackages()
    if (res.code !== 1 || !res.data) throw new Error(res.msg || '加载大厅失败')
    packages.value = res.data.packages
    if (!res.data.packages.some((p) => p.modpackId === expandedId.value)) {
      expandedId.value = null
      rooms.value = []
    }
  } catch (e) {
    console.error('Failed to load lobby packages:', e)
    toastError(`加载大厅失败：${e instanceof Error ? e.message : String(e)}`)
  } finally {
    packagesLoading.value = false
  }
}

/** 展开/收起某整合包的房间列表 */
async function toggleExpand(pkg: LobbyPackageItem) {
  if (expandedId.value === pkg.modpackId) {
    expandedId.value = null
    rooms.value = []
    return
  }
  expandedId.value = pkg.modpackId
  roomsLoading.value = true
  rooms.value = []
  try {
    const res = await listLobbyRooms({ packageId: pkg.modpackId, page: 1, pageSize: 50 })
    if (res.code !== 1 || !res.data) throw new Error(res.msg || '加载房间列表失败')
    rooms.value = res.data.rooms
  } catch (e) {
    console.error('Failed to load lobby rooms:', e)
    toastError(`加载房间列表失败：${e instanceof Error ? e.message : String(e)}`)
    expandedId.value = null
  } finally {
    roomsLoading.value = false
  }
}

function handleJoin(room: LobbyRoomItem) {
  if (inRoom.value) return
  if (room.hasPassword) {
    joinTarget.value = room
    return
  }
  void doJoin(room, '')
}

/** 执行加入（含密码）：成功 ok=true；失败 ok=false + error 内联展示 */
async function doJoin(room: LobbyRoomItem, password: string): Promise<{ ok: boolean; error?: string }> {
  if (inRoom.value) return { ok: false, error: '您当前已在房间中' }
  joiningId.value = room.publicIdentifier
  try {
    await store.guestJoinRoom(room.publicIdentifier, password)
    return { ok: true }
  } catch (e) {
    return { ok: false, error: e instanceof Error ? e.message : String(e) }
  } finally {
    joiningId.value = null
  }
}

/** 密码弹窗的 join 回调 */
function joinWithPassword(password: string) {
  if (!joinTarget.value) return Promise.resolve({ ok: false, error: '房间信息已失效' })
  return doJoin(joinTarget.value, password)
}

onMounted(() => {
  void loadPackages()
})
</script>

<template>
  <div class="space-y-4">
    <AlertV2 type="info" message="MoLaunch 联机基于与「陶瓦联机」相同的 Scaffolding 协议与 EasyTier 实现，房间码与其他协议兼容启动器互通；大厅中的公开房间（未设密码）可直接加入，与陌生人一起游玩" />
    <AlertV2 type="info" message="大厅房间按整合包聚类展示，点击卡片可查看该整合包下的公开房间；私密房间需凭房间码从「加入房间」进入" />

    <Card title="联机大厅">
      <template #extra>
        <Button type="ghost" size="small" :loading="packagesLoading" @click="loadPackages">
          <template #icon><ArrowPathIcon class="w-3.5 h-3.5" /></template>
          刷新
        </Button>
      </template>

      <div v-if="packagesLoading" class="py-10 text-center text-sm text-gray-500">正在加载大厅...</div>

      <div v-else-if="packages.length === 0" class="py-10 flex flex-col items-center justify-center gap-2 text-gray-400">
        <Squares2X2Icon class="w-8 h-8" />
        <span class="text-sm">暂无公开房间，快去创建一个吧</span>
      </div>

      <div v-else class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
        <div
          v-for="pkg in packages"
          :key="pkg.modpackId"
          class="rounded-lg border bg-white overflow-hidden"
          :class="expandedId === pkg.modpackId ? 'border-primary-300 shadow-sm' : 'border-gray-200 hover:border-primary-300 hover:shadow-sm transition-all'"
        >
          <button
            type="button"
            class="w-full px-4 py-3 text-left flex items-center gap-2"
            @click="toggleExpand(pkg)"
          >
            <CubeIcon class="w-4 h-4 text-gray-400 shrink-0" />
            <span class="text-sm font-medium text-gray-800 truncate flex-1">{{ pkg.name }}</span>
            <Tag size="small" color="arcoblue" class="shrink-0">{{ pkg.source }}</Tag>
            <span v-if="pkg.mcVersion" class="text-xs text-gray-500 shrink-0">MC {{ pkg.mcVersion }}</span>
            <span class="inline-flex items-center gap-0.5 text-xs text-gray-500 shrink-0">
              <UserGroupIcon class="w-3.5 h-3.5" />
              {{ pkg.roomCount }}
            </span>
            <ChevronDownIcon
              class="w-4 h-4 text-gray-400 shrink-0 transition-transform"
              :class="expandedId === pkg.modpackId ? 'rotate-180' : ''"
            />
          </button>

          <div v-if="expandedId === pkg.modpackId" class="border-t border-gray-100 px-3 py-3 space-y-2 bg-gray-50/50">
            <div v-if="roomsLoading" class="py-4 text-center text-xs text-gray-500">正在加载房间...</div>
            <div v-else-if="rooms.length === 0" class="py-4 text-center text-xs text-gray-400">该整合包暂无公开房间</div>
            <template v-else>
              <LobbyRoomCard
                v-for="room in rooms"
                :key="room.publicIdentifier"
                :room="room"
                :joining="joiningId === room.publicIdentifier"
                :in-room="inRoom"
                @join="handleJoin"
              />
            </template>
          </div>
        </div>
      </div>
    </Card>

    <LobbyJoinDialog
      v-if="joinTarget"
      :room="joinTarget"
      :join="joinWithPassword"
      @close="joinTarget = null"
    />
  </div>
</template>
