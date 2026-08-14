<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
/**
 * 认证中心：管理厂商 OAuth2 / Device Code / API Key 认证。
 * 参见 FRP_MANAGER_DESIGN.md §6.8。
 * 逻辑见 composables/useFrpAuthCenter.ts，本文件仅保留模板组装。
 */
import { useFrpStore } from '@/stores/frp'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
import { useFrpAuthCenter } from '@/composables/useFrpAuthCenter'
import {
  ShieldCheckIcon,
  ArrowPathIcon,
  KeyIcon,
  ArrowTopRightOnSquareIcon,
  CheckCircleIcon,
  ExclamationCircleIcon,
  XCircleIcon,
} from '@heroicons/vue/24/outline'

const store = useFrpStore()
const {
  providers, loading, countdowns,
  authBadge, statusInfo, formatRemaining, formatCountdown,
  openUrl,
  handleStartOAuth2, handleStartDeviceCode,
  handleCancelDeviceCode, handleRefreshToken, handleRevokeAuth,
  handleSaveApiKey, handleRefreshAuthStatuses,
} = useFrpAuthCenter()
</script>

<template>
  <div class="space-y-4">
    <!-- 顶部操作栏 -->
    <div class="flex items-center justify-between flex-wrap gap-2">
      <p class="text-sm text-gray-500">共 {{ providers.length }} 个厂商</p>
      <Tooltip text="刷新认证状态">
        <Button type="ghost" size="small" :loading="loading" @click="handleRefreshAuthStatuses">
          <template #icon><ArrowPathIcon class="w-4 h-4" /></template>
        </Button>
      </Tooltip>
    </div>

    <!-- 厂商认证卡片 -->
    <div v-if="providers.length > 0" class="space-y-3">
      <div
        v-for="provider in providers"
        :key="provider.id"
        class="rounded-lg border border-gray-200 bg-white p-4 hover:border-primary-300 transition-all"
      >
        <div class="flex items-start gap-3">
          <!-- 厂商图标 -->
          <div class="w-10 h-10 rounded-lg bg-primary-50 flex items-center justify-center shrink-0 overflow-hidden">
            <img v-if="provider.icon" :src="provider.icon" :alt="provider.name" class="w-full h-full object-cover" />
            <ShieldCheckIcon v-else class="w-5 h-5 text-primary-600" />
          </div>

          <!-- 厂商信息 + 认证状态 -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 flex-wrap">
              <span class="text-sm font-semibold text-gray-900">{{ provider.name }}</span>
              <span v-if="authBadge(provider.authType)" class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium" :class="authBadge(provider.authType)!.cls">
                {{ authBadge(provider.authType)!.text }}
              </span>
            </div>
            <p class="text-xs text-gray-500 mt-1">{{ provider.description }}</p>

            <!-- 认证状态行 -->
            <div class="flex items-center gap-1.5 mt-2">
              <CheckCircleIcon v-if="statusInfo(store.authStatuses[provider.id]).cls.includes('green')" class="w-4 h-4" :class="statusInfo(store.authStatuses[provider.id]).cls" />
              <ExclamationCircleIcon v-else-if="statusInfo(store.authStatuses[provider.id]).cls.includes('amber')" class="w-4 h-4" :class="statusInfo(store.authStatuses[provider.id]).cls" />
              <XCircleIcon v-else-if="statusInfo(store.authStatuses[provider.id]).cls.includes('red')" class="w-4 h-4" :class="statusInfo(store.authStatuses[provider.id]).cls" />
              <span class="text-xs font-medium" :class="statusInfo(store.authStatuses[provider.id]).cls">
                {{ statusInfo(store.authStatuses[provider.id]).text }}
              </span>
              <span v-if="store.authStatuses[provider.id]?.authenticated && store.authStatuses[provider.id]?.expiresAt" class="text-xs text-gray-400">
                {{ formatRemaining(store.authStatuses[provider.id]?.expiresAt) }}
              </span>
            </div>

            <!-- 权限范围 -->
            <div v-if="store.authStatuses[provider.id]?.scopes?.length" class="flex items-center gap-1 mt-1 flex-wrap">
              <span class="text-xs text-gray-400">权限：</span>
              <span v-for="scope in store.authStatuses[provider.id]!.scopes" :key="scope" class="inline-flex px-1.5 py-0.5 rounded text-xs bg-gray-100 text-gray-600">{{ scope }}</span>
            </div>

            <!-- Device Code 流程展示 -->
            <div v-if="store.deviceCodeInfos[provider.id]" class="mt-3 p-3 bg-purple-50 rounded-md space-y-2">
              <div class="flex items-center gap-2">
                <span class="text-xs text-gray-600">用户码：</span>
                <span class="text-lg font-mono font-bold text-purple-700 tracking-wider">{{ store.deviceCodeInfos[provider.id].userCode }}</span>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-xs text-gray-600">验证链接：</span>
                <a class="text-xs text-blue-600 hover:underline flex items-center gap-0.5 cursor-pointer" @click.prevent="openUrl(store.deviceCodeInfos[provider.id]!.verificationUri)">
                  {{ store.deviceCodeInfos[provider.id].verificationUri }}
                  <ArrowTopRightOnSquareIcon class="w-3 h-3" />
                </a>
              </div>
              <div class="flex items-center justify-between">
                <span class="text-xs text-gray-500">倒计时：{{ formatCountdown(countdowns[provider.id] || 0) }}</span>
                <div class="flex items-center gap-1.5">
                  <span v-if="store.deviceCodePolling[provider.id]" class="flex items-center gap-1 text-xs text-purple-600">
                    <ArrowPathIcon class="w-3 h-3 animate-spin" />轮询中
                  </span>
                  <Button type="ghost" size="mini" @click="handleCancelDeviceCode(provider.id)">取消</Button>
                </div>
              </div>
            </div>
          </div>

          <!-- 操作区 -->
          <div class="shrink-0 flex flex-col items-end gap-1.5">
            <!-- 无需认证 -->
            <span v-if="provider.authType === 'none'" class="text-xs text-gray-400">无需认证</span>

            <!-- OAuth2 未认证 -->
            <Button v-else-if="provider.authType === 'oauth2' && !store.authStatuses[provider.id]?.authenticated" type="primary" size="small" :loading="!!store.authActionLoading[provider.id]" @click="handleStartOAuth2(provider.id)">
              <template #icon><KeyIcon class="w-3.5 h-3.5" /></template>
              开始认证
            </Button>

            <!-- Device Code 未认证 -->
            <Button v-else-if="provider.authType === 'device_code' && !store.authStatuses[provider.id]?.authenticated && !store.deviceCodeInfos[provider.id]" type="primary" size="small" :loading="!!store.authActionLoading[provider.id]" @click="handleStartDeviceCode(provider.id)">
              <template #icon><KeyIcon class="w-3.5 h-3.5" /></template>
              开始认证
            </Button>

            <!-- API Key 输入 -->
            <div v-else-if="provider.authType === 'api_key' && !store.authStatuses[provider.id]?.authenticated" class="flex items-center gap-1.5 w-48">
              <Input :model-value="store.apiKeyInputs[provider.id] || ''" placeholder="输入 API Key" size="small" @update:model-value="(v: string) => store.apiKeyInputs[provider.id] = v" />
              <Button type="primary" size="small" :loading="!!store.authActionLoading[provider.id]" @click="handleSaveApiKey(provider.id)">保存</Button>
            </div>

            <!-- 已认证：刷新 + 撤销 -->
            <template v-else-if="store.authStatuses[provider.id]?.authenticated && provider.authType !== 'none'">
              <Button v-if="provider.authType !== 'api_key'" type="outline" size="mini" :loading="!!store.authActionLoading[provider.id]" @click="handleRefreshToken(provider.id)">
                <template #icon><ArrowPathIcon class="w-3.5 h-3.5" /></template>
                刷新
              </Button>
              <Button type="ghost" size="mini" :loading="!!store.authActionLoading[provider.id]" @click="handleRevokeAuth(provider)">
                撤销
              </Button>
            </template>
          </div>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="!loading" class="flex flex-col items-center justify-center py-16">
      <ShieldCheckIcon class="w-12 h-12 text-gray-300 mb-3" />
      <p class="text-sm font-medium text-gray-500">暂无厂商</p>
      <p class="text-xs text-gray-400 mt-1">系统默认厂商将自动显示，或先安装外部厂商</p>
    </div>

    <!-- 加载中 -->
    <div v-else class="flex items-center justify-center py-16">
      <ArrowPathIcon class="w-6 h-6 text-gray-400 animate-spin" />
    </div>
  </div>
</template>
