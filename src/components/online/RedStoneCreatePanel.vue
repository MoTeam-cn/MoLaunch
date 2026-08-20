<script setup lang="ts">
/**
 * 红石联机 - 创建房间面板
 *
 * 选择中转服务器 + 本地 MC 端口拉起 hongshi 内核创建隧道；
 * 状态机/轮询/端口选择与事件回填逻辑见 useRedStonePanel，本文件仅模板组装。
 */
import { computed, defineAsyncComponent } from 'vue'
import {
  ArrowPathIcon,
  BoltIcon,
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
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const Alert = defineAsyncComponent(() => import('@/components/common/Alert.vue'))
import { latencyTagColor, useRedStonePanel } from '@/composables/useRedStonePanel'

const {
  serverLoading, serverError, useManualServer, server, mcPort, portSelecting,
  phase, errorMessage, creating, stopping, restarting,
  serverOptions, address, latencyTesting,
  handleSelectPort, loadServers, testServersLatency,
  handleCreate, handleRestart, handleStop, copyAddress,
} = useRedStonePanel()

const selectedOption = computed(() => serverOptions.value.find((o) => o.value === server.value))
</script>

<template>
  <div class="space-y-4">
    <Card title="中转服务器">
      <div class="space-y-3">
        <Alert variant="soft" v-if="serverError" type="error" :message="serverError" />
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
            <Select v-model="server" :options="serverOptions" placeholder="选择中转服务器" custom-option>
              <template #selected="{ label }">
                <span class="flex items-center gap-2 min-w-0 w-full">
                  <span class="truncate min-w-0 flex-1">{{ label }}</span>
                  <Tag v-if="selectedOption?.latencyMs != null" :color="latencyTagColor(selectedOption.latencyMs)" size="small">
                    {{ selectedOption.latencyMs }}ms
                  </Tag>
                </span>
              </template>
              <template #option="{ option, selected }">
                <span class="flex items-center gap-2 min-w-0 flex-1 self-center">
                  <span class="truncate min-w-0">{{ option.label }}</span>
                </span>
                <span class="flex items-center gap-2 flex-none self-center">
                  <Tag v-if="option.latencyMs != null" :color="latencyTagColor(option.latencyMs)" size="small">
                    {{ option.latencyMs }}ms
                  </Tag>
                  <svg
                    v-if="selected"
                    class="w-3 h-3 flex-none text-primary-500"
                    viewBox="0 0 1024 1024"
                    fill="currentColor"
                  >
                    <path d="M912 192c-12.8 0-25.6 4.266667-34.133333 12.8L384 699.2 234.666667 548.266667c-17.066667-17.066667-46.933333-17.066667-64 0-17.066667 17.066667-17.066667 46.933333 0 64l179.2 179.2c8.533333 8.533333 21.333333 12.8 34.133333 12.8s25.6-4.266667 34.133333-12.8l520.533334-520.533334c17.066667-17.066667 17.066667-46.933333 0-64-8.533333-8.533333-21.333333-12.8-34.133334-12.8z" />
                  </svg>
                </span>
              </template>
            </Select>
          </div>
          <Button type="outline" :loading="latencyTesting" @click="testServersLatency()">
            <template #icon><BoltIcon class="w-4 h-4" /></template>测延迟
          </Button>
          <Button type="outline" :loading="serverLoading" @click="loadServers">
            <template #icon><ArrowPathIcon class="w-4 h-4" /></template>刷新
          </Button>
        </div>
        <Alert variant="soft" type="info" message="服务器列表实时来自红石官网，选择就近节点延迟更低" />
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
        <Alert variant="soft" type="info" message="内核采用懒连接转发，游戏可不预启动；端口已自动探测并监听后端推送回填，可手动修改" />
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
        <Alert variant="soft" type="error" :message="errorMessage" />
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
      <Alert variant="soft" type="info" message="隧道上限 10 人 / 10Mbps；每次创建地址不同，无需长期保留" />
      <Alert variant="soft" type="warning" message="隧道无玩家 10 分钟或运行满 6 小时后将自动关闭" />
    </Card>
  </div>
</template>