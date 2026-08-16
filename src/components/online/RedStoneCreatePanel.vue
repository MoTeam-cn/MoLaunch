<script setup lang="ts">
/**
 * 红石联机 - 创建房间面板
 *
 * 选择中转服务器 + 本地 MC 端口拉起 hongshi 内核创建隧道；
 * 状态机/轮询/端口选择与事件回填逻辑见 useRedStonePanel，本文件仅模板组装。
 */
import { defineAsyncComponent } from 'vue'
import {
  ArrowPathIcon,
  ClipboardDocumentIcon,
  PlayIcon,
  ServerStackIcon,
  SignalIcon,
  StopIcon,
} from '@heroicons/vue/24/outline'
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const AlertV2 = defineAsyncComponent(() => import('@/components/common/AlertV2.vue'))
import { useRedStonePanel } from '@/composables/useRedStonePanel'

const {
  serverLoading, serverError, useManualServer, server, mcPort, portSelecting,
  phase, errorMessage, creating, stopping, restarting,
  serverOptions, address,
  handleSelectPort, loadServers, handleCreate, handleRestart, handleStop, copyAddress,
} = useRedStonePanel()
</script>

<template>
  <div class="space-y-4">
    <Card title="中转服务器">
      <div class="space-y-3">
        <AlertV2 v-if="serverError" type="error" :message="serverError" />
        <div v-if="useManualServer" class="flex items-center gap-2">
          <div class="flex-1">
            <Input v-model="server" placeholder="手动填写中转服务器地址（如 hk.hongshi.site）" />
          </div>
          <Button type="outline" :loading="serverLoading" @click="loadServers">
            <template #icon><ArrowPathIcon class="w-4 h-4" /></template>重试
          </Button>
        </div>
        <div v-else class="flex items-center gap-2">
          <div class="flex-1">
            <Select v-model="server" :options="serverOptions" placeholder="选择中转服务器" />
          </div>
          <Button type="outline" :loading="serverLoading" @click="loadServers">
            <template #icon><ArrowPathIcon class="w-4 h-4" /></template>刷新
          </Button>
        </div>
        <AlertV2 type="info" message="服务器列表实时来自红石官网，选择就近节点延迟更低" />
      </div>
    </Card>
    <Card title="本地 MC 服务">
      <div class="space-y-3">
        <div class="flex items-center gap-2">
          <div class="flex-1">
            <Input v-model="mcPort" placeholder="MC 服务监听端口（如 25565）" />
          </div>
          <Button type="outline" :loading="portSelecting" @click="handleSelectPort">
            <template #icon><ServerStackIcon class="w-4 h-4" /></template>选择端口
          </Button>
        </div>
        <AlertV2 type="info" message="内核采用懒连接转发，游戏可不预启动；端口已自动探测并监听后端推送回填，可手动修改" />
      </div>
    </Card>
    <Card title="隧道状态">
      <div v-if="phase === 'idle'">
        <Button type="primary" long :loading="creating" @click="handleCreate">
          <template #icon><PlayIcon class="w-4 h-4" /></template>创建隧道
        </Button>
      </div>
      <div v-else-if="phase === 'creating'" class="flex flex-col items-center justify-center py-8">
        <SignalIcon class="w-8 h-8 text-blue-400 animate-pulse" />
        <p class="text-sm text-gray-600 font-medium mt-3">等待隧道建立…</p>
        <p class="text-xs text-gray-400 mt-1">正在连接 {{ server || '中转服务器' }}，通常需要数秒</p>
      </div>
      <div v-else-if="phase === 'open'" class="flex flex-col items-center gap-3 py-2">
        <div class="text-base sm:text-lg font-bold text-gray-800 break-all text-center">{{ address }}</div>
        <div class="flex flex-wrap items-center justify-center gap-2">
          <Button type="secondary" @click="copyAddress">
            <template #icon><ClipboardDocumentIcon class="w-4 h-4" /></template>复制地址
          </Button>
          <Button type="outline" :loading="restarting" @click="handleRestart">
            <template #icon><ArrowPathIcon class="w-4 h-4" /></template>重启
          </Button>
          <Button type="ghost" :loading="stopping" @click="handleStop">
            <template #icon><StopIcon class="w-4 h-4" /></template>停止联机
          </Button>
        </div>
        <p class="text-xs text-gray-400">将地址发给好友即可直连进服；隧道 10 人上限 / 10Mbps</p>
      </div>
      <div v-else class="space-y-3">
        <AlertV2 type="error" :message="errorMessage" />
        <div class="flex items-center gap-2">
          <Button type="outline" :loading="restarting" @click="handleRestart">
            <template #icon><ArrowPathIcon class="w-4 h-4" /></template>重新创建
          </Button>
          <Button type="ghost" :loading="stopping" @click="handleStop">
            <template #icon><StopIcon class="w-4 h-4" /></template>停止隧道
          </Button>
        </div>
      </div>
    </Card>
    <Card title="说明">
      <AlertV2 type="info" message="隧道上限 10 人 / 10Mbps；每次创建地址不同，无需长期保留" />
    </Card>
  </div>
</template>