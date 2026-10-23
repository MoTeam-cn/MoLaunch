<script setup lang="ts">
/**
 * 设置-联机 - 联机大厅开关卡片
 *
 * 联机大厅是**联机功能**的开关，只管房间与搭桥联机：
 * - 关闭（默认）：联机大厅与房间功能不可用
 * - 开启：需先确认说明，之后可使用联机大厅与房间功能
 *
 * 基础联网项（设备身份注册与续期、启动上报、更新检查、公告拉取、FRP 服务器列表）
 * 属启动器基础能力，**不受本开关影响**，文案须如实列出，避免被读成「关闭即不联网」。
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
  '启用后，本启动器会使用你的设备标识与 MoLaunch 云端服务交互，用于实现联机大厅与房间功能：',
  '',
  '· 上传与查询公开房间信息（房间名、整合包、在线人数等）；',
  '· 在联机大厅中按整合包浏览并加入公开房间；',
  '· 在云端登记房间会话（房主心跳、房主关闭房间）。',
  '',
  '不受本开关影响的基础联网项（启动时默认进行）：',
  '· 设备标识与访问令牌的注册/续期（更新检查按设备灰度、活跃统计需要设备身份）；',
  '· 启动上报（每次启动一次，用于活跃统计）；',
  '· 更新检查与公告拉取；',
  '· FRP 公共服务器列表（进入内网穿透页时读取）。',
  '',
  '关闭该开关后，仅联机大厅与房间功能不可用，上述基础联网项仍会继续。',
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
        label="启用联机大厅（房间与搭桥）"
        description="关闭时联机大厅与房间功能不可用。设备标识注册与续期、启动上报、更新检查、公告拉取、FRP 服务器列表属基础联网项，默认进行、不受此开关影响。开启需先确认说明。"
        :hover="false"
        @update:model-value="handleChange"
      />
    </div>
  </div>
</template>
