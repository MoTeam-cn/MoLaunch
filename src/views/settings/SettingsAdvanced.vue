<script setup lang="ts">
import { ref, watch } from 'vue'
import { useConfigPage } from '@/composables/useConfigPage'
import Alert from '@/components/common/Alert.vue'
import Input from '@/components/common/Input.vue'
import Select from '@/components/common/Select.vue'
import DevModeToggle from '@/components/settings/DevModeToggle.vue'
import CommunityConfigCard from '@/components/community/CommunityConfigCard.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import Button from '@/components/common/Button.vue'
import {
  ExclamationTriangleIcon,
  EyeIcon,
  EyeSlashIcon,
  ArrowTopRightOnSquareIcon,
} from '@heroicons/vue/24/outline'

const proxyMode = ref<'none' | 'system' | 'custom'>('none')
const proxyType = ref<'http' | 'https' | 'socks5'>('http')
const proxyUrl = ref('')

// CurseForge API Key 配置（加密存储到 INI）
const cfEnabled = ref(false)
const cfApiKey = ref('')
const cfShowKey = ref(false)

// 日志级别（控制日志输出详细程度，0=关闭 ~ 5=跟踪）
const logLevel = ref(3)

const { loaded, markDirty } = useConfigPage({
  delay: 1000,
  errorLabel: 'save advanced settings',
  onLoad: (cfg) => {
    proxyMode.value = cfg.proxyMode as typeof proxyMode.value
    proxyType.value = cfg.proxyType as typeof proxyType.value
    proxyUrl.value = cfg.proxyUrl
    cfEnabled.value = cfg.curseforgeEnabled
    cfApiKey.value = cfg.curseforgeApiKey
    logLevel.value = cfg.logLevel
  },
})

// 代理：普通 INI 存储
watch(proxyMode, (v) => markDirty('proxyMode', v))
watch(proxyType, (v) => markDirty('proxyType', v))
watch(proxyUrl, (v) => markDirty('proxyUrl', v))

// CurseForge：走加密存储（applyConfig 内部分流到 secure_storage）
watch(cfEnabled, (v) => markDirty('curseforgeEnabled', v))
watch(cfApiKey, (v) => markDirty('curseforgeApiKey', v))

// 日志级别：后端会同步调用 logger::set_level 立即生效
watch(logLevel, (v) => markDirty('logLevel', v))
</script>

<template>
  <div class="space-y-6">
    <!-- 加载占位（避免初始值与实际值不一致导致的闪烁） -->
    <div v-if="!loaded" class="space-y-6">
      <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
        <div class="px-5 py-5">
          <div class="h-4 w-24 bg-gray-200 rounded animate-pulse mb-4" />
          <div class="h-10 bg-gray-100 rounded animate-pulse" />
        </div>
      </div>
      <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
        <div class="px-5 py-5">
          <div class="h-4 w-40 bg-gray-200 rounded animate-pulse mb-4" />
          <div class="h-10 bg-gray-100 rounded animate-pulse" />
        </div>
      </div>
    </div>

    <template v-else>
    <!-- 开发者模式（仅在「其他」页连续点击版本号 5 次解锁后显示） -->
    <DevModeToggle />

    <!-- 代理配置 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">代理配置</h3>

      <!-- 提示框 -->
      <div class="mx-5 mb-4">
        <Alert type="warning" message="默认不走系统代理。即使开启了系统代理设置，本启动器也不会自动使用，除非您在此处手动配置。" />
      </div>

      <div class="divide-y divide-gray-200">
        <!-- 代理模式 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">代理模式</p>
              <p class="text-xs text-gray-500 mt-0.5">选择启动器的网络代理方式</p>
            </div>
            <div class="flex-none w-40">
              <Select
                v-model="proxyMode"
                :options="[
                  { label: '不使用代理', value: 'none' },
                  { label: '系统代理', value: 'system' },
                  { label: '自定义代理', value: 'custom' },
                ]"
              />
            </div>
          </div>
          <p class="text-xs text-gray-400 mt-2">
            <template v-if="proxyMode === 'none'">不使用任何代理，直接连接</template>
            <template v-else-if="proxyMode === 'system'">使用操作系统中配置的代理设置</template>
            <template v-else>手动配置代理服务器地址和端口</template>
          </p>
        </div>

        <!-- 自定义代理配置 -->
        <div v-if="proxyMode === 'custom'" class="px-5 py-4 space-y-4">
          <!-- 代理类型 -->
          <div class="flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">代理类型</p>
              <p class="text-xs text-gray-500 mt-0.5">选择代理协议</p>
            </div>
            <div class="flex-none w-40">
              <Select
                v-model="proxyType"
                :options="[
                  { label: 'HTTP', value: 'http' },
                  { label: 'HTTPS', value: 'https' },
                  { label: 'SOCKS5', value: 'socks5' },
                ]"
              />
            </div>
          </div>

          <!-- 代理地址 -->
          <div>
            <p class="text-sm font-medium text-gray-900 mb-2">代理地址</p>
            <Input v-model="proxyUrl" placeholder="127.0.0.1:7890" />
            <p class="text-xs text-gray-400 mt-1">格式：IP地址:端口号，例如 127.0.0.1:7890</p>
          </div>
        </div>
      </div>
    </div>

    <!-- CurseForge API Key 配置 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">CurseForge API Key</h3>

      <!-- 提示框 -->
      <div class="mx-5 mb-4 space-y-2">
        <Alert
          type="warning"
          :truncate="false"
          message="推荐使用自己的 API Key 请求官方源。镜像站资源有限，很多情况下会出现等待过久的情况，使用自己的 API Key 可以显著提高加载速度。"
        />
        <Alert
          type="info"
          :truncate="false"
          message="启用并配置 API Key 后，CurseForge 请求将走官方 API；不启用则走镜像源。API Key 会用 SDK DES 加密后存入 INI，不会明文落盘。"
        />
      </div>

      <div class="divide-y divide-gray-200">
        <!-- 启用开关 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">启用 API Key</p>
              <p class="text-xs text-gray-500 mt-0.5">启用后使用官方 API，未配置 Key 时仍回退到镜像</p>
            </div>
            <div class="flex-none w-40">
              <Select
                :model-value="cfEnabled ? 'true' : 'false'"
                :options="[
                  { label: '已启用', value: 'true' },
                  { label: '未启用', value: 'false' },
                ]"
                @update:model-value="cfEnabled = $event === 'true'"
              />
            </div>
          </div>
          <p class="text-xs text-gray-400 mt-2">
            <template v-if="cfEnabled">已启用：CurseForge 请求走官方 API（api.curseforge.com）</template>
            <template v-else>未启用：CurseForge 请求走镜像源（mod.mcimirror.top）</template>
          </p>
        </div>

        <!-- API Key 输入 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between mb-2">
            <div>
              <p class="text-sm font-medium text-gray-900">API Key</p>
              <p class="text-xs text-gray-500 mt-0.5">从 CurseForge Console 申请，使用 SDK DES 加密后存入 INI</p>
            </div>
            <a
              href="https://console.curseforge.com/?#/api-keys"
              target="_blank"
              class="inline-flex items-center gap-1 text-xs text-primary-600 hover:text-primary-700"
            >
              申请地址
              <ArrowTopRightOnSquareIcon class="w-3 h-3" />
            </a>
          </div>
          <Input
            v-model="cfApiKey"
            :type="cfShowKey ? 'text' : 'password'"
            placeholder="粘贴你的 CurseForge API Key"
            class="font-mono"
          >
            <template #suffix>
              <Tooltip :text="cfShowKey ? '隐藏' : '显示'" position="top">
                <Button
                  type="ghost"
                  size="mini"
                  class="!h-6 !w-6 !p-0 text-gray-400 hover:!text-gray-600"
                  @click="cfShowKey = !cfShowKey"
                >
                  <EyeSlashIcon v-if="cfShowKey" class="w-4 h-4" />
                  <EyeIcon v-else class="w-4 h-4" />
                </Button>
              </Tooltip>
            </template>
          </Input>
          <!-- 状态提示 -->
          <div v-if="cfEnabled && !cfApiKey" class="mt-2 flex items-center gap-1.5 text-xs text-amber-600">
            <ExclamationTriangleIcon class="w-3.5 h-3.5" />
            <span>已启用但未填写 API Key，请求将回退到镜像源</span>
          </div>
          <div v-else-if="cfEnabled && cfApiKey" class="mt-2 flex items-center gap-1.5 text-xs text-green-600">
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
              <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
            </svg>
            <span>已配置，CurseForge 请求将走官方 API</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 系统设置（日志级别） -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">系统</h3>
      <div class="divide-y divide-gray-200">
        <div class="px-5 py-4 flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-900">日志级别</p>
            <p class="text-xs text-gray-500 mt-0.5">控制日志输出的详细程度</p>
          </div>
          <Select
            :model-value="logLevel"
            :options="[
              { label: '关闭', value: 0 },
              { label: '错误', value: 1 },
              { label: '警告', value: 2 },
              { label: '信息', value: 3 },
              { label: '调试', value: 4 },
              { label: '跟踪', value: 5 },
            ]"
            style="min-width: 100px"
            @update:model-value="logLevel = Number($event)"
          />
        </div>
      </div>
    </div>

    <!-- 社区资源配置卡片 -->
    <CommunityConfigCard />
    </template>
  </div>
</template>
