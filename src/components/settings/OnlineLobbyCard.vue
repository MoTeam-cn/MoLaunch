<script setup lang="ts">
/**
 * 设置-联机 - 联机大厅开关卡片
 *
 * 联机大厅是启动器与 MoLaunch 云端服务交互的唯一入口开关：
 * - 关闭（默认）：不与云端发生任何交互，启动时不静默注册设备、不续期令牌
 * - 开启：需先确认与云端交互说明，之后启动时自动完成设备注册与令牌续期
 */
import { defineAsyncComponent, ref } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { showConfirmAsync } from '@/utils/modal'

const ToggleRow = defineAsyncComponent(() => import('@/components/settings/ToggleRow.vue'))

const onlineStore = useOnlineStore()

/** 是否正在保存开关（避免连点重复触发） */
const saving = ref(false)

/** 与云端交互说明（启用前必须确认） */
const CONSENT_MESSAGE = [
  '启用后，本启动器会与 MoLaunch 云端服务交互，用于实现联机大厅与房间功能：',
  '',
  '· 注册本机设备标识，并在每次启动时静默完成设备注册与访问令牌续期；',
  '· 上传与查询公开房间信息（房间名、整合包、在线人数等）；',
  '· 在联机大厅中按整合包浏览并加入公开房间。',
  '',
  '关闭该开关后，启动器不再与云端发生任何交互（不注册设备、不续期令牌），联机大厅与房间功能将不可用。',
  'FRP 内网穿透使用独立的第三方凭证，不受此开关影响。',
].join('\n')

/**
 * 开关变更
 *
 * 开启：先确认与云端交互说明，确认后才写入配置并连接云端
 * 关闭：直接写入配置并停止云端交互
 */
async function handleChange(value: boolean) {
  if (saving.value) return
  if (value) {
    const agreed = await showConfirmAsync('启用联机大厅', CONSENT_MESSAGE)
    if (!agreed) return
  }
  saving.value = true
  await onlineStore.setLobbyEnabled(value)
  saving.value = false
}
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">联机大厅</h3>
    <div class="divide-y divide-gray-200">
      <ToggleRow
        :model-value="onlineStore.lobbyEnabled"
        label="启用联机大厅（与云端交互）"
        description="关闭时启动器不与云端发生任何交互：不会静默注册设备、不会续期访问令牌，联机大厅与房间功能不可用。开启需先确认与云端交互的说明。"
        :hover="false"
        @update:model-value="handleChange"
      />
    </div>
  </div>
</template>
