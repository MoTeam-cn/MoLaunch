<script setup lang="ts">
/** 隧道创建/编辑表单：模板保留，状态与交互逻辑由 composable 管理。 */
import { toRef, defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Checkbox = defineAsyncComponent(() => import('@/components/common/Checkbox.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const InputGroup = defineAsyncComponent(() => import('@/components/common/InputGroup.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import { useTunnelCreateForm, modeOptions, typeOptions, bandwidthLimitModeOptions, proxyProtocolVersionOptions } from '@/composables/useTunnelCreateForm'
import type { CreateTunnelParams, Tunnel, UpdateTunnelParams } from '@/types/frp'
import {
  AdjustmentsHorizontalIcon,
  ArrowPathIcon,
  CheckCircleIcon,
  ChevronDownIcon,
  ServerStackIcon,
  XCircleIcon,
} from '@heroicons/vue/24/outline'

const props = defineProps<{
  actionLoading: boolean
  editTunnel?: Tunnel
}>()
const emit = defineEmits<{
  create: [params: CreateTunnelParams]
  update: [params: UpdateTunnelParams]
  cancel: []
}>()

const {
  form, isEdit, isOfficial, publicServersLoading, publicServerOptions,
  handlePublicServerChange, portSelecting, handleSelectPort, checkHint,
  handleImportConfig, handleSubmit,
} = useTunnelCreateForm(
  toRef(props, 'editTunnel'),
  params => emit('create', params),
  params => emit('update', params),
)
</script>

<template>
  <div class="rounded-lg border border-gray-200 bg-gray-50 p-4 space-y-3">
    <div v-if="!isEdit">
      <label class="block text-xs font-medium text-gray-700 mb-1">服务器模式</label>
      <Select v-model="form.mode" :options="modeOptions" />
    </div>

    <div v-if="isOfficial">
      <label class="block text-xs font-medium text-gray-700 mb-1">公共服务器</label>
      <Select
        v-model="form.publicServerId"
        :options="publicServerOptions"
        :disabled="publicServersLoading"
        :placeholder="publicServersLoading ? '加载中...' : '选择服务器'"
        @update:model-value="handlePublicServerChange"
      />
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div>
        <label class="block text-xs font-medium text-gray-700 mb-1">隧道名称</label>
        <Input v-model="form.name" placeholder="我的隧道" />
      </div>
      <div>
        <label class="block text-xs font-medium text-gray-700 mb-1">隧道类型</label>
        <Select v-model="form.tunnelType" :options="typeOptions" />
      </div>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div>
        <label class="block text-xs font-medium text-gray-700 mb-1">本地 IP</label>
        <Input v-model="form.localIp" placeholder="127.0.0.1" />
      </div>
      <div>
        <label class="block text-xs font-medium text-gray-700 mb-1">本地端口</label>
        <div class="flex items-center gap-2">
          <Input v-model="form.localPort" type="number" placeholder="25565" class="flex-1" />
          <Tooltip text="选择本机端口">
            <Button type="outline" :loading="portSelecting" @click="handleSelectPort">
              <template #icon><ServerStackIcon class="w-4 h-4" /></template>
            </Button>
          </Tooltip>
        </div>
      </div>
    </div>

    <div>
      <label class="block text-xs font-medium text-gray-700 mb-1">服务器地址</label>
      <InputGroup :ratio="[3, 1]">
        <Input v-model="form.serverAddr" placeholder="frps.example.com" :readonly="isOfficial" />
        <Input v-model="form.serverPort" type="number" placeholder="7000" :readonly="isOfficial" />
      </InputGroup>
      <p
        v-if="checkHint"
        class="mt-1 flex items-center gap-1 text-xs"
        :class="checkHint.type === 'success' ? 'text-green-600' : checkHint.type === 'error' ? 'text-red-500' : 'text-gray-500'"
      >
        <CheckCircleIcon v-if="checkHint.type === 'success'" class="w-3.5 h-3.5" />
        <XCircleIcon v-else-if="checkHint.type === 'error'" class="w-3.5 h-3.5" />
        <ArrowPathIcon v-else class="w-3.5 h-3.5 animate-spin" />
        {{ checkHint.text }}
      </p>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <div>
        <label class="block text-xs font-medium text-gray-700 mb-1">远程端口</label>
        <Input v-model="form.remotePort" type="number" placeholder="30000" :readonly="isOfficial" />
      </div>
      <div>
        <label class="block text-xs font-medium text-gray-700 mb-1">Token（可选）</label>
        <Input v-model="form.token" placeholder="留空表示无鉴权" :readonly="isOfficial" />
      </div>
    </div>

    <div class="border-t border-dashed border-gray-200 pt-2">
      <button
        type="button"
        class="flex w-full items-center gap-3 py-2 text-left text-xs text-gray-500 transition-colors hover:text-gray-800"
        @click="form.advancedOpen = !form.advancedOpen"
      >
        <AdjustmentsHorizontalIcon class="h-4 w-4 text-gray-400" />
        <span class="flex-1">高级设置</span>
        <ChevronDownIcon class="h-4 w-4 transition-transform" :class="form.advancedOpen ? 'rotate-180' : ''" />
      </button>
      <Transition
        enter-active-class="transition-all duration-200 ease-out"
        leave-active-class="transition-all duration-150 ease-in"
        enter-from-class="max-h-0 opacity-0 -translate-y-1"
        enter-to-class="max-h-64 opacity-100 translate-y-0"
        leave-from-class="max-h-64 opacity-100 translate-y-0"
        leave-to-class="max-h-0 opacity-0 -translate-y-1"
      >
        <div v-if="form.advancedOpen" class="ml-7 max-h-64 space-y-3 overflow-hidden pb-2 pt-1">
          <div class="flex items-center gap-2">
            <span class="w-24 shrink-0 text-xs text-gray-500">带宽限制</span>
            <Input v-model="form.bandwidthLimit" placeholder="例如 4MB" />
            <Select
              v-model="form.bandwidthLimitMode"
              :options="bandwidthLimitModeOptions"
              class="w-28 shrink-0"
            />
          </div>
          <div class="flex items-center gap-2">
            <span class="w-24 shrink-0 text-xs text-gray-500">Proxy 传输</span>
            <Checkbox v-model="form.proxyUseEncryption">加密</Checkbox>
            <Checkbox v-model="form.proxyUseCompression">压缩</Checkbox>
            <Select
              v-model="form.proxyProtocolVersion"
              :options="proxyProtocolVersionOptions"
              class="w-24 shrink-0"
            />
          </div>
          <div class="flex items-center gap-3">
            <span class="w-24 shrink-0 text-xs text-gray-500">连接加密</span>
            <Checkbox v-model="form.useTls" :disabled="isOfficial">启用传输层 TLS</Checkbox>
          </div>
        </div>
      </Transition>
    </div>

    <div class="flex items-center justify-between border-t border-dashed border-gray-200 py-3">
      <span class="text-xs text-gray-400">从标准 frpc 配置快速填充</span>
      <Button type="outline" size="small" @click="handleImportConfig">导入配置</Button>
    </div>

    <div class="flex justify-end gap-2 pt-2">
      <Button type="ghost" size="small" @click="emit('cancel')">取消</Button>
      <Button
        type="primary"
        size="small"
        :loading="actionLoading"
        :disabled="!form.name.trim() || !form.serverAddr.trim()"
        @click="handleSubmit"
      >
        {{ isEdit ? '保存' : '创建' }}
      </Button>
    </div>
  </div>
</template>
