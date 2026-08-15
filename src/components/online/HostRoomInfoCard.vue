<script setup lang="ts">
/**
 * 房主房间信息卡（Scaffolding 收敛版）
 *
 * 房间码展示 N 段公开标识（6 位显示名），可折叠查看完整 U/xxx 码并一键复制；
 * 另展示 MC 版本/端口、加载器、备注、公开状态与剩余时间（房主新开房会续期）。
 */
import { ref, computed, defineAsyncComponent } from 'vue'
import { useOnlineStore } from '@/stores/online'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import { copyToClipboard } from '@/utils/clipboard'
import {
  ServerStackIcon,
  ClockIcon,
  ClipboardDocumentIcon,
  EyeSlashIcon,
} from '@heroicons/vue/24/outline'

const store = useOnlineStore()
const room = computed(() => store.roomState)

/** 完整房间码折叠状态（默认收起，只展示 N 段公开标识） */
const showFullCode = ref(false)

const remainingSeconds = computed(() => {
  if (!room.value.expiresAt) return 0
  return Math.max(0, room.value.expiresAt - Math.floor(Date.now() / 1000))
})

const remainingText = computed(() => {
  const s = remainingSeconds.value
  if (s <= 0) return '已过期'
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  return h > 0 ? `${h}小时${m}分钟` : `${m}分钟`
})

const loaderText = computed(() => {
  const type = room.value.hostLoader
  const map: Record<string, string> = {
    forge: 'Forge', fabric: 'Fabric', neoforge: 'NeoForge',
    quilt: 'Quilt', vanilla: '原版', release: '原版',
  }
  const name = type ? (map[type] ?? type) : ''
  return room.value.hostLoaderVersion ? `${name} ${room.value.hostLoaderVersion}` : name
})

/** 复制完整房间码（U/xxx，含 S 段密钥） */
async function copyFullCode() {
  if (!room.value.roomCode) return
  await copyToClipboard(room.value.roomCode, { toast: true })
}
</script>

<template>
  <Card title="房间信息">
    <div class="divide-y divide-gray-100">
      <div class="px-1 py-3 flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm text-gray-600">
          <ServerStackIcon class="w-4 h-4 text-gray-400" /><span>房间码</span>
        </div>
        <div class="flex items-center gap-1.5">
          <code
            class="text-base font-semibold text-primary-600 tracking-wider bg-primary-50 px-3 py-1 rounded cursor-pointer select-all"
            title="点击切换完整码"
            @click="showFullCode = !showFullCode"
          >
            {{ showFullCode ? room.roomCode : room.publicIdentifier }}
          </code>
          <Tooltip text="复制完整房间码">
            <Button type="ghost" size="mini" @click="copyFullCode">
              <template #icon><ClipboardDocumentIcon class="w-3.5 h-3.5" /></template>
            </Button>
          </Tooltip>
        </div>
      </div>
      <div class="px-1 py-3 flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm text-gray-600">
          <ServerStackIcon class="w-4 h-4 text-gray-400" /><span>MC 版本 / 端口</span>
        </div>
        <span class="text-xs text-gray-900">
          {{ room.hostMcVersion || '-' }}<template v-if="room.hostMcPort">:{{ room.hostMcPort }}</template>
        </span>
      </div>
      <div v-if="loaderText" class="px-1 py-3 flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm text-gray-600">
          <ServerStackIcon class="w-4 h-4 text-gray-400" /><span>加载器</span>
        </div>
        <span class="text-xs text-gray-900">{{ loaderText }}</span>
      </div>
      <div v-if="room.remark" class="px-1 py-3 flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm text-gray-600">
          <ServerStackIcon class="w-4 h-4 text-gray-400" /><span>备注</span>
        </div>
        <span class="text-xs text-gray-900 truncate max-w-[50%]">{{ room.remark }}</span>
      </div>
      <div class="px-1 py-3 flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm text-gray-600">
          <ServerStackIcon class="w-4 h-4 text-gray-400" />
          <span>房间类型</span>
        </div>
        <span class="text-xs text-gray-900">
          {{ room.isPublic ? '公开' : '私密' }}<template v-if="room.hasPassword"> · 有密码</template>
        </span>
      </div>
      <div class="px-1 py-3 flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm text-gray-600">
          <ClockIcon class="w-4 h-4 text-gray-400" />
          <Tooltip text="房间保留时间：若在此时间内无新玩家加入，房间将自动清退；正常游玩中的房间会自动续期保留，无需担心">
            <span>剩余时间</span>
          </Tooltip>
        </div>
        <span class="text-xs" :class="remainingSeconds < 300 ? 'text-red-600' : 'text-gray-900'">
          {{ remainingText }}
        </span>
      </div>
      <div v-if="!room.isPublic" class="px-1 py-3 flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm text-gray-600">
          <EyeSlashIcon class="w-4 h-4 text-gray-400" /><span>私密房间</span>
        </div>
        <span class="text-xs text-gray-500">仅凭房间码加入，不进大厅</span>
      </div>
    </div>
  </Card>
</template>
