<script setup lang="ts">
/**
 * 联机大厅（Scaffolding 收敛版）
 *
 * 按整合包聚类的卡片（热度排序），点击展开该整合包下的公开房间摘要列表；
 * 加入房间：无密码直接进房，有密码弹 LobbyJoinDialog；进房后由 RoomManager 切到房客面板。
 */
import { ref, computed, onMounted, onActivated, defineAsyncComponent } from 'vue'
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
import { useEasyTierInstall } from '@/composables/useEasyTierInstall'
import { toastError } from '@/utils/toast'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const Alert = defineAsyncComponent(() => import('@/components/common/Alert.vue'))
const LobbyRoomCard = defineAsyncComponent(() => import('./LobbyRoomCard.vue'))
const LobbyJoinDialog = defineAsyncComponent(() => import('./LobbyJoinDialog.vue'))

const store = useOnlineStore()
/** easytier 内核前置检查（大厅加入前必须已安装，缺失时弹窗引导前往设置页） */
const install = useEasyTierInstall()

const packages = ref<LobbyPackageItem[]>([])
const packagesLoading = ref(false)
const expandedId = ref<string | null>(null)
const roomsLoading = ref(false)
const rooms = ref<LobbyRoomItem[]>([])
/** 未关联整合包（纯原版等）的公开房间数（「其他房间」分组） */
const otherRoomCount = ref(0)
/** 正在加入的房间（LobbyRoomCard joining 标记） */
const joiningId = ref<string | null>(null)
/** 密码弹窗目标房间 */
const joinTarget = ref<LobbyRoomItem | null>(null)

/** 「其他房间」分组展开标识（独立于整合包 modpackId 命名空间） */
const OTHER_GROUP_ID = '__other__'

/** 是否已在房间中（房主/房客均禁止再加入） */
const inRoom = computed(() => store.roomState.role !== null)

async function loadPackages(silent = false) {
  if (!silent) packagesLoading.value = true
  try {
    const res = await listLobbyPackages()
    if (res.code !== 1 || !res.data) throw new Error(res.msg || '加载大厅失败')
    packages.value = res.data.packages
    otherRoomCount.value = res.data.otherRoomCount ?? 0
    if (!res.data.packages.some((p) => p.modpackId === expandedId.value)) {
      expandedId.value = null
      rooms.value = []
    }
  } catch (e) {
    console.error('Failed to load lobby packages:', e)
    toastError(`加载大厅失败：${e instanceof Error ? e.message : String(e)}`)
  } finally {
    if (!silent) packagesLoading.value = false
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

/** 展开/收起「其他房间」（未关联整合包/纯原版公开房间，服务端不传 package_id 返回全部再过滤） */
async function toggleOtherGroup() {
  if (expandedId.value === OTHER_GROUP_ID) {
    expandedId.value = null
    rooms.value = []
    return
  }
  expandedId.value = OTHER_GROUP_ID
  roomsLoading.value = true
  rooms.value = []
  try {
    const res = await listLobbyRooms({ page: 1, pageSize: 50 })
    if (res.code !== 1 || !res.data) throw new Error(res.msg || '加载房间列表失败')
    rooms.value = res.data.rooms.filter((r) => !r.modpack)
  } catch (e) {
    console.error('Failed to load other rooms:', e)
    toastError(`加载其他房间失败：${e instanceof Error ? e.message : String(e)}`)
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
  // 点击后立即标记该房间加入中（内核检查/登记全程锁定按钮，避免延迟感）
  joiningId.value = room.publicIdentifier
  try {
    // 前置依赖：easytier 内核未安装时不加入房间（弹窗引导前往设置页下载）
    const kernelOk = await install.ensureKernel('加入房间')
    if (!kernelOk) return { ok: false, error: 'easytier 内核未安装' }
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

/** 激活序号：keep-alive 下首次激活紧邻 onMounted，跳过避免重复加载 */
let activatedCount = 0
onMounted(() => {
  void loadPackages()
})
onActivated(() => {
  activatedCount += 1
  // 从其他菜单切回联机大厅时静默刷新（避免点亮的 loading 闪烁）
  if (activatedCount > 1) void loadPackages(true)
})
</script>

<template>
  <div class="space-y-4">
    <Alert variant="soft" type="info" message="MoLaunch 联机基于与「陶瓦联机」相同的 Scaffolding 协议与 EasyTier 实现，房间码与其他协议兼容启动器互通；大厅中的公开房间（未设密码）可直接加入，与陌生人一起游玩" />
    <Alert variant="soft" type="info" message="大厅房间按整合包聚类展示，点击卡片可查看该整合包下的公开房间；未关联整合包的原版房间归入「其他房间」；私密房间需凭房间码从「加入房间」进入" />

    <Card title="联机大厅">
      <template #extra>
        <Button type="ghost" size="small" :loading="packagesLoading" @click="loadPackages()">
          <template #icon><ArrowPathIcon class="w-3.5 h-3.5" /></template>
          刷新
        </Button>
      </template>

      <div v-if="packagesLoading" class="py-10 text-center text-sm text-gray-500">正在加载大厅...</div>

      <div v-else-if="packages.length === 0 && otherRoomCount === 0" class="py-10 flex flex-col items-center justify-center gap-2 text-gray-400">
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

        <div
          v-if="otherRoomCount > 0"
          class="rounded-lg border bg-white overflow-hidden"
          :class="expandedId === OTHER_GROUP_ID ? 'border-primary-300 shadow-sm' : 'border-gray-200 hover:border-primary-300 hover:shadow-sm transition-all'"
        >
          <button
            type="button"
            class="w-full px-4 py-3 text-left flex items-center gap-2"
            @click="toggleOtherGroup"
          >
            <Squares2X2Icon class="w-4 h-4 text-gray-400 shrink-0" />
            <span class="text-sm font-medium text-gray-800 truncate flex-1">其他房间</span>
            <Tag size="small" color="gray" class="shrink-0">原版</Tag>
            <span class="inline-flex items-center gap-0.5 text-xs text-gray-500 shrink-0">
              <UserGroupIcon class="w-3.5 h-3.5" />
              {{ otherRoomCount }}
            </span>
            <ChevronDownIcon
              class="w-4 h-4 text-gray-400 shrink-0 transition-transform"
              :class="expandedId === OTHER_GROUP_ID ? 'rotate-180' : ''"
            />
          </button>

          <div v-if="expandedId === OTHER_GROUP_ID" class="border-t border-gray-100 px-3 py-3 space-y-2 bg-gray-50/50">
            <div v-if="roomsLoading" class="py-4 text-center text-xs text-gray-500">正在加载房间...</div>
            <div v-else-if="rooms.length === 0" class="py-4 text-center text-xs text-gray-400">暂无其他公开房间</div>
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
