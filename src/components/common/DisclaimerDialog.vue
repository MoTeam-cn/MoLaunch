<template>
  <Drawer
    :visible="visible"
    placement="right"
    :width="520"
    render-in-place
    popup-container="#app-content"
    :closable="false"
    :mask-closable="false"
    :esc-to-close="false"
    @update:visible="emit('update:visible', $event)"
  >
    <template #title>
      <div class="flex items-center gap-1.5">
        <ShieldExclamationIcon class="h-4 w-4 text-primary-500" />
        <span>使用协议与免责声明</span>
      </div>
    </template>

    <div class="space-y-4 text-sm leading-relaxed text-gray-600">
      <!-- 通用声明 -->
      <div class="rounded-md border border-amber-200 bg-amber-50 px-3 py-2.5">
        <p class="text-xs font-medium text-amber-800">通用声明</p>
        <p class="mt-1 text-xs leading-relaxed text-amber-700">
          本启动器及作者不对因使用本功能所引发的任何直接或间接后果承担责任。点击「我已知悉并同意」即表示您已阅读、理解并同意以下全部说明。
        </p>
      </div>

      <!-- 联机功能说明（P2P / TUN / FRP） -->
      <div v-if="kind === 'online'">
        <p class="mb-1.5 text-xs font-medium text-gray-500">联机功能说明</p>
        <div class="space-y-2 rounded-md border border-gray-200 bg-gray-50 px-3 py-2.5 text-xs leading-relaxed text-gray-600">
          <p>房间创建、管理与房间列表等操作仅由 MoLaunch 服务器完成，本启动器不涉及流量的中转及内容的传播。</p>
          <p>联机使用 P2P 技术：通过国内外服务商提供的 TURN 服务器获取本机网络类型，并在无法直连时中转流量；同时创建一个虚拟的 TUN 网络用于游戏数据互通。</p>
          <p>FRP（内网穿透）隧道由第三方服务商提供，MoLaunch 仅提供配置与管理界面，不对第三方服务的稳定性、可用性及其内容承担任何责任。每家 FRP 服务商均拥有各自的用户协议与使用条款，请在注册与使用前阅读并遵守服务商的相关规定。</p>
        </div>
      </div>

      <!-- 工具功能说明 -->
      <div v-else-if="kind === 'tools'">
        <p class="mb-1.5 text-xs font-medium text-gray-500">工具功能说明</p>
        <div class="space-y-2 rounded-md border border-gray-200 bg-gray-50 px-3 py-2.5 text-xs leading-relaxed text-gray-600">
          <p>工具页中的功能大多在本地完成（存档管理、Mod 工具、Java 管理、计算工具等），不涉及数据上传。</p>
          <p>外部下载、网络测试、种子地图等工具会访问您指定的地址或第三方服务，其稳定性与内容由第三方负责；下载内容的合法性与用途由您自行负责。</p>
          <p>使用工具所产生的结果与后果由用户自行承担，本启动器及作者不作任何保证。</p>
        </div>
      </div>

      <!-- 开发者选项说明 -->
      <div v-else-if="kind === 'developer'">
        <p class="mb-1.5 text-xs font-medium text-gray-500">开发者选项说明</p>
        <div class="space-y-2 rounded-md border border-gray-200 bg-gray-50 px-3 py-2.5 text-xs leading-relaxed text-gray-600">
          <p>开发者选项面向开发与排障场景（日志查看、证书设置、DevTools、深链接注册等），普通玩家无需开启，请勿随意修改其中配置。</p>
          <p>忽略 TLS 证书校验、自定义证书信任源等操作会改变本启动器的网络安全策略，可能暴露于中间人攻击等风险，请在充分了解后果后使用。</p>
          <p>开发者模式下产生的行为与结果由您自行承担，本启动器及作者不对因修改开发者选项所引发的任何直接或间接后果承担责任。</p>
        </div>
      </div>

      <!-- 实验性功能说明 -->
      <div v-else>
        <p class="mb-1.5 text-xs font-medium text-gray-500">实验性功能说明</p>
        <div class="space-y-2 rounded-md border border-gray-200 bg-gray-50 px-3 py-2.5 text-xs leading-relaxed text-gray-600">
          <p>实验性功能（AI 聊天、日志分析等）处于测试阶段，功能可能不稳定，数据可能随时调整或丢失。</p>
          <p>AI 聊天会将对话内容发送至您自行配置的模型服务端点（本地或第三方），数据外发与隐私风险由您自行评估与承担。</p>
          <p>实验性功能不构成任何承诺，作者不对使用实验性功能产生的后果承担责任。</p>
        </div>
      </div>

      <!-- 合规提醒 -->
      <div class="rounded-md border border-gray-200 bg-gray-50 px-3 py-2.5">
        <p class="text-xs font-medium text-gray-700">合规提醒</p>
        <p class="mt-1 text-xs leading-relaxed text-gray-500">
          您应确保使用本功能的行为符合当地法律法规及网络安全相关规定，不得利用本启动器从事任何违法活动；联机内容与 AI 对话内容由用户自行负责。
        </p>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-end">
        <Button type="primary" size="small" @click="handleAgree">我已知悉并同意</Button>
      </div>
    </template>
  </Drawer>
</template>

<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'
import { ShieldExclamationIcon } from '@heroicons/vue/24/outline'
const Drawer = defineAsyncComponent(() => import('@/components/common/Drawer.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
import { markAgreedToday, type DisclaimerKind } from '@/utils/disclaimer'
import { toastSuccess, toastWarning } from '@/utils/toast'

const props = defineProps<{
  /** 是否显示 */
  visible: boolean
  /** 协议类型：online（联机）/ experimental（实验性功能）/ tools（工具）/ developer（开发者选项） */
  kind: DisclaimerKind
}>()

const emit = defineEmits<{
  'update:visible': [visible: boolean]
}>()

/** 同意并关闭：记录当天已同意，同日内再次进入不再弹出 */
function handleAgree() {
  markAgreedToday(props.kind)
  emit('update:visible', false)
  toastSuccess('已确认使用协议，今日不再提醒')
}

// 跳往其他页面时若抽屉仍未关闭（未确认），提示用户已放弃确认，下次进入将再次提醒
onBeforeRouteLeave(() => {
  if (props.visible) {
    toastWarning('已放弃确认使用协议，下次进入将再次提醒')
  }
})
</script>
