<script setup lang="ts">
/**
 * 厂商列表
 *
 * 展示已安装的 Frp 厂商：
 * - 内置「系统默认」厂商（不可卸载/禁用），首次使用时提供「下载 frpc」入口
 * - 外部厂商（manifest.toml 安装）：支持启用/禁用切换、卸载
 *
 * 顶部操作栏提供「从文件夹安装」「从 ZIP 安装」两种安装入口。
 *
 * 徽章覆盖：
 * - authType：none/oauth2/device_code/api_key
 * - distribution：system/bundled/url
 */
import { onMounted, computed } from 'vue'
import { useFrpStore } from '@/stores/frp'
import { showConfirm } from '@/utils/modal'
import { toastInfo } from '@/utils/toast'
import { pickFile, pickDirectory } from '@/utils/fileDialog'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import Select from '@/components/common/Select.vue'
import {
  ServerStackIcon,
  ArrowDownTrayIcon,
  ArrowPathIcon,
  CheckCircleIcon,
  ExclamationCircleIcon,
  FolderOpenIcon,
  ArchiveBoxIcon,
  TrashIcon,
} from '@heroicons/vue/24/outline'
import type { ProviderInfo } from '@/types/frp'

const store = useFrpStore()

const providers = computed(() => store.providers)
const loading = computed(() => store.providersLoading)
const frpcDownloading = computed(() => store.frpcDownloading)
const actionLoading = computed(() => store.providerActionLoading)

/** 启用/禁用选项（每个外部厂商卡片一个 Select） */
const enableOptions = [
  { label: '启用', value: 'enabled' },
  { label: '禁用', value: 'disabled' },
]

/** authType 徽章：颜色 + 文案 */
function authBadge(authType: string): { cls: string; text: string } | null {
  switch (authType) {
    case 'none': return { cls: 'bg-green-50 text-green-700', text: '无需认证' }
    case 'oauth2': return { cls: 'bg-blue-50 text-blue-700', text: 'OAuth2' }
    case 'device_code': return { cls: 'bg-purple-50 text-purple-700', text: 'Device Code' }
    case 'api_key': return { cls: 'bg-yellow-50 text-yellow-700', text: 'API Key' }
    default: return null
  }
}

/** distribution 徽章：颜色 + 文案 */
function distBadge(dist: string): { cls: string; text: string } | null {
  switch (dist) {
    case 'system': return { cls: 'bg-gray-100 text-gray-600', text: '系统' }
    case 'bundled': return { cls: 'bg-blue-50 text-blue-700', text: '内置' }
    case 'url': return { cls: 'bg-cyan-50 text-cyan-700', text: '在线下载' }
    default: return null
  }
}

onMounted(() => {
  void store.loadProviders()
})

async function handleRefresh() {
  await store.loadProviders()
  toastInfo('厂商列表已刷新')
}

async function handleDownloadFrpc() {
  await store.downloadFrpc()
}

/** 从文件夹安装厂商（manifest.toml + frpc 二进制） */
async function handleInstallFromDir() {
  const dir = await pickDirectory({ title: '选择厂商目录（含 manifest.toml）' })
  if (!dir) return
  await store.installProviderFromDir(dir)
}

/** 从 ZIP 包安装厂商 */
async function handleInstallFromZip() {
  const file = await pickFile({
    title: '选择厂商 ZIP 包',
    filters: [{ name: 'ZIP', extensions: ['zip'] }],
  })
  if (!file) return
  await store.installProviderFromZip(file)
}

/** 切换厂商启用/禁用状态 */
async function handleToggleProvider(providerId: string, enabled: boolean) {
  await store.toggleProvider(providerId, enabled)
}

/** 卸载外部厂商（二次确认） */
function handleUninstall(p: ProviderInfo) {
  showConfirm(
    '卸载厂商',
    `确定要卸载厂商「${p.name}」吗？相关 frpc 二进制和配置将被清除，此操作不可恢复。`,
    async () => {
      await store.uninstallProvider(p.id)
    },
  )
}
</script>

<template>
  <div class="space-y-4">
    <!-- 顶部操作栏 -->
    <div class="flex items-center justify-between flex-wrap gap-2">
      <p class="text-sm text-gray-500">共 {{ providers.length }} 个厂商</p>
      <div class="flex items-center gap-1.5">
        <Tooltip text="从文件夹安装（含 manifest.toml）">
          <Button type="outline" size="small" :loading="actionLoading" @click="handleInstallFromDir">
            <template #icon><FolderOpenIcon class="w-4 h-4" /></template>
            从文件夹
          </Button>
        </Tooltip>
        <Tooltip text="从 ZIP 包安装厂商">
          <Button type="outline" size="small" :loading="actionLoading" @click="handleInstallFromZip">
            <template #icon><ArchiveBoxIcon class="w-4 h-4" /></template>
            从 ZIP
          </Button>
        </Tooltip>
        <Tooltip text="刷新厂商列表">
          <Button type="ghost" size="small" :loading="loading" @click="handleRefresh">
            <template #icon><ArrowPathIcon class="w-4 h-4" /></template>
          </Button>
        </Tooltip>
      </div>
    </div>

    <!-- 厂商卡片列表 -->
    <div v-if="providers.length > 0" class="space-y-3">
      <div
        v-for="provider in providers"
        :key="provider.id"
        class="rounded-lg border border-gray-200 bg-white p-4 hover:border-primary-300 hover:shadow-sm transition-all"
        :class="{ 'opacity-60': !provider.enabled }"
      >
        <div class="flex items-start gap-3">
          <!-- 厂商图标 -->
          <div class="w-10 h-10 rounded-lg bg-primary-50 flex items-center justify-center shrink-0">
            <ServerStackIcon class="w-5 h-5 text-primary-600" />
          </div>

          <!-- 厂商信息 -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 flex-wrap">
              <span class="text-sm font-semibold text-gray-900">{{ provider.name }}</span>
              <span
                v-if="provider.builtin"
                class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-600"
              >
                内置
              </span>
              <span
                v-if="authBadge(provider.authType)"
                class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium"
                :class="authBadge(provider.authType)!.cls"
              >
                {{ authBadge(provider.authType)!.text }}
              </span>
              <span
                v-if="distBadge(provider.distribution)"
                class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium"
                :class="distBadge(provider.distribution)!.cls"
              >
                {{ distBadge(provider.distribution)!.text }}
              </span>
            </div>
            <p class="text-xs text-gray-500 mt-1">{{ provider.description }}</p>
            <p class="text-xs text-gray-400 mt-0.5">
              v{{ provider.version }} · {{ provider.author }}
            </p>
          </div>

          <!-- 操作区：内置厂商显示 frpc 状态，外部厂商显示启禁 + 卸载 -->
          <div class="shrink-0 flex flex-col items-end gap-1.5">
            <template v-if="provider.builtin">
              <div v-if="provider.frpcReady" class="flex items-center gap-1 text-xs text-green-600">
                <CheckCircleIcon class="w-4 h-4" />
                <span>frpc 就绪</span>
              </div>
              <div v-else class="flex items-center gap-1 text-xs text-amber-600">
                <ExclamationCircleIcon class="w-4 h-4" />
                <span>frpc 未就绪</span>
              </div>
              <Button
                v-if="!provider.frpcReady"
                type="primary"
                size="mini"
                :loading="frpcDownloading"
                @click="handleDownloadFrpc"
              >
                <template #icon><ArrowDownTrayIcon class="w-3.5 h-3.5" /></template>
                下载 frpc
              </Button>
            </template>
            <template v-else>
              <Select
                :model-value="provider.enabled ? 'enabled' : 'disabled'"
                :options="enableOptions"
                :disabled="actionLoading"
                custom-option
                @update:model-value="(v: string | number) => handleToggleProvider(provider.id, v === 'enabled')"
              />
              <Tooltip text="卸载厂商">
                <Button
                  type="ghost"
                  size="mini"
                  :loading="actionLoading"
                  @click="handleUninstall(provider)"
                >
                  <template #icon><TrashIcon class="w-3.5 h-3.5" /></template>
                  卸载
                </Button>
              </Tooltip>
            </template>
          </div>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="!loading" class="flex flex-col items-center justify-center py-16">
      <ServerStackIcon class="w-12 h-12 text-gray-300 mb-3" />
      <p class="text-sm font-medium text-gray-500">暂无厂商</p>
      <p class="text-xs text-gray-400 mt-1">系统默认厂商将自动显示，或从文件夹/ZIP 安装外部厂商</p>
    </div>

    <!-- 加载中 -->
    <div v-else class="flex items-center justify-center py-16">
      <ArrowPathIcon class="w-6 h-6 text-gray-400 animate-spin" />
    </div>
  </div>
</template>
