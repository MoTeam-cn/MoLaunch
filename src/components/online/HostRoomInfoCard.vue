<script setup lang="ts">
/**
 * 房主房间信息卡（Scaffolding 收敛版）
 *
 * 房间码展示 N 段公开标识（6 位显示名），可折叠查看完整 U/xxx 码并一键复制；
 * 另展示 MC 版本/端口、加载器、备注与公开状态。
 */
import { ref, computed, defineAsyncComponent } from 'vue'
import { useOnlineStore } from '@/stores/online'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const AlertV2 = defineAsyncComponent(() => import('@/components/common/AlertV2.vue'))
import { copyToClipboard } from '@/utils/clipboard'
import {
  ServerStackIcon,
  ClipboardDocumentIcon,
  EyeSlashIcon,
} from '@heroicons/vue/24/outline'

const store = useOnlineStore()
const room = computed(() => store.roomState)

/** 完整房间码折叠状态（默认收起，只展示 N 段公开标识） */
const showFullCode = ref(false)

const loaderText = computed(() => {
  const type = room.value.hostLoader
  const map: Record<string, string> = {
    forge: 'Forge', fabric: 'Fabric', neoforge: 'NeoForge',
    quilt: 'Quilt', vanilla: '原版', release: '原版',
  }
  const name = type ? (map[type] ?? type) : ''
  return room.value.hostLoaderVersion ? `${name} ${room.value.hostLoaderVersion}` : name
})

/** MC 端口是否已偏离创建时的端口（自动热更新 / 手动覆盖后为 true，仅房主侧有意义） */
const portChanged = computed(
  () =>
    room.value.role === 'host' &&
    store.easytierRuntime.mcPort > 0 &&
    store.easytierRuntime.mcPort !== room.value.hostMcPort,
)

const portAlertMessage = computed(
  () =>
    `MC 端口已变更为 ${store.easytierRuntime.mcPort}（创建时为 ${room.value.hostMcPort}）：` +
    '未使用 MoLaunch 启动器联机的朋友无法感知新端口，需要退出房间后重新加入才能进入；' +
    'MoLaunch 的端口热更新仅对同启动器生效',
)

/** 展示端口：优先实时端口（手动指定/自动探测），未探测到回退创建时快照 */
const displayMcPort = computed(() => {
  const live = store.easytierRuntime.mcPort
  return live > 0 ? live : room.value.hostMcPort
})

/** 复制完整房间码（U/xxx，含 S 段密钥） */
async function copyFullCode() {
  if (!room.value.roomCode) return
  await copyToClipboard(room.value.roomCode, { toast: true })
}
</script>

<template>
  <Card title="房间信息">
    <AlertV2
      v-if="portChanged"
      type="warning"
      class="mb-3"
      :message="portAlertMessage"
    />
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
          {{ room.hostMcVersion || '-' }}<template v-if="displayMcPort">:{{ displayMcPort }}</template>
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
      <div v-if="!room.isPublic" class="px-1 py-3 flex items-center justify-between">
        <div class="flex items-center gap-2 text-sm text-gray-600">
          <EyeSlashIcon class="w-4 h-4 text-gray-400" /><span>私密房间</span>
        </div>
        <span class="text-xs text-gray-500">仅凭房间码加入，不进大厅</span>
      </div>
    </div>
  </Card>
</template>
