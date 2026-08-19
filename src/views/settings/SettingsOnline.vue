<script setup lang="ts">
/**
 * 设置 - 联机 Tab：ApiServerCard + easytier 内核/中继节点 + 设备管理
 */
import { computed, onMounted, ref, watch, defineAsyncComponent } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useConfigPage } from '@/composables/useConfigPage'
import { toastError, toastSuccess } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { useTauriEvent } from '@/composables/useTauriEvent'
import {
  getEasyTierInstallStatus,
  installEasyTier,
  updateEasyTier,
} from '@/utils/api/online-manager/easytier'
import type { EasyTierInstallProgress, EasyTierInstallStatus } from '@/types/online'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const ApiServerCard = defineAsyncComponent(() => import('@/components/settings/ApiServerCard.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
import {
  ArrowDownTrayIcon,
  ArrowPathIcon,
  ArrowRightOnRectangleIcon,
  TrashIcon,
  ServerStackIcon,
} from '@heroicons/vue/24/outline'

const onlineStore = useOnlineStore()

// ============ easytier 公共中继节点 ============
const peersText = ref('')
const peersSaved = ref(false)

const {
  loaded: loadedPeers,
  markDirty: markDirtyPeers,
  flushSave: flushSavePeers,
} = useConfigPage({
  delay: 800,
  errorLabel: 'save easytier peers',
  onLoad: (cfg) => {
    peersText.value = (cfg.onlineEasytierPublicPeers ?? []).join('\n')
  },
})

watch(peersText, () => {
  peersSaved.value = false
  markDirtyPeers('onlineEasytierPublicPeers', parsePeers())
})

function parsePeers(): string[] {
  return peersText.value
    .split('\n')
    .map((l) => l.trim())
    .filter((l) => l.length > 0)
}

async function handleSavePeers() {
  await flushSavePeers()
  peersSaved.value = true
  toastSuccess('已保存')
}

// ============ easytier 内核（外部下载安装） ============
const installStatus = ref<EasyTierInstallStatus | null>(null)
const installProgress = ref<EasyTierInstallProgress | null>(null)
const installBusy = ref(false)

const downloading = computed(() => installStatus.value?.downloading ?? false)
const hasUpdate = computed(() => {
  const s = installStatus.value
  return !!s?.installed && !!s.latestVersion && s.version !== s.latestVersion
})
const showProgress = computed(() => downloading.value && !!installProgress.value)

/** 状态 Tag：检查中 gray / 下载中 blue / 未安装 red / 有新版本 gold / 已安装 green */
const tagColor = computed(() => {
  if (!installStatus.value) return 'gray'
  if (downloading.value) return 'blue'
  if (!installStatus.value.installed) return 'red'
  if (hasUpdate.value) return 'gold'
  return 'green'
})

const tagText = computed(() => {
  if (downloading.value) return `下载中 ${installProgress.value?.percent ?? 0}%`
  const s = installStatus.value
  if (!s) return '检查中'
  if (!s.installed) return '未安装'
  if (hasUpdate.value) return `有新版本 v${s.latestVersion}`
  return `已安装 v${s.version}`
})

const buttonText = computed(() => {
  if (!installStatus.value?.installed) return '下载'
  if (hasUpdate.value) return '更新'
  return '重新下载'
})

async function refreshInstallStatus() {
  try {
    installStatus.value = await getEasyTierInstallStatus()
  } catch (e) {
    console.error('查询 easytier 内核安装状态失败', e)
  }
}

async function handleInstall() {
  installBusy.value = true
  try {
    if (hasUpdate.value) {
      await updateEasyTier()
      toastSuccess('easytier 内核已更新')
    } else {
      await installEasyTier()
      toastSuccess('easytier 内核安装完成')
    }
  } catch (e) {
    toastError(`操作失败: ${e}`)
  } finally {
    installBusy.value = false
    await refreshInstallStatus()
  }
}

/** 安装进度事件：done/error 阶段清除进度并刷新状态 */
const installProgressEvent = useTauriEvent<EasyTierInstallProgress>(
  'easytier-install-progress',
  (p) => {
    installProgress.value = p
    if (p.phase === 'done' || p.phase === 'error') {
      installProgress.value = null
      void refreshInstallStatus()
    }
  },
)

// ============ 设备登出 / 清除凭证 ============
async function handleLogout() {
  await onlineStore.logout()
}

function handleClearCredentials() {
  showConfirm(
    '清除设备凭证',
    '此操作将删除本地密钥与 JWT，需要重新注册设备才能继续使用联机功能。\n是否继续？',
    async () => {
      await onlineStore.clear()
    },
  )
}

onMounted(() => {
  void onlineStore.refreshStatus()
  installProgressEvent.start()
  void refreshInstallStatus()
})
</script>

<template>
  <div class="space-y-6">
    <!-- api-server 配置（自管理加载状态） -->
    <ApiServerCard />

    <!-- easytier 内核（外部下载安装） -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">easytier 内核</h3>
      <div class="divide-y divide-gray-200">
        <div class="px-5 py-4">
          <div class="flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">内核程序</p>
              <p class="text-xs text-gray-500 mt-0.5">从 GitHub 下载安装 easytier-core，未安装时首次组网会自动下载</p>
            </div>
            <Tag :color="tagColor" size="small">{{ tagText }}</Tag>
          </div>
          <div class="mt-3 flex items-center justify-between gap-4">
            <Button
              :type="installStatus?.installed && !hasUpdate ? 'outline' : 'primary'"
              size="small"
              :loading="installBusy || downloading"
              :disabled="!installStatus || downloading"
              @click="handleInstall"
            >
              <template #icon>
                <ArrowDownTrayIcon v-if="!hasUpdate" class="w-4 h-4" />
                <ArrowPathIcon v-else class="w-4 h-4" />
              </template>
              {{ buttonText }}
            </Button>
          </div>
          <!-- 安装进度 -->
          <div v-if="showProgress" class="mt-3">
            <div class="flex items-center gap-3">
              <div class="relative h-2 flex-1 overflow-hidden rounded-full bg-gray-100">
                <div
                  class="h-full rounded-full bg-primary-500 transition-all duration-300"
                  :style="{ width: (installProgress?.percent ?? 0) + '%' }"
                />
              </div>
              <span class="w-10 shrink-0 text-right text-xs font-semibold tabular-nums text-primary-600">
                {{ installProgress?.percent ?? 0 }}%
              </span>
            </div>
            <p class="text-xs text-gray-400 mt-1">{{ installProgress?.message }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- easytier 公共中继节点 -->
    <div v-if="!loadedPeers" class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <div class="px-5 py-5">
        <div class="h-4 w-28 bg-gray-200 rounded animate-pulse mb-4" />
        <div class="h-20 bg-gray-100 rounded animate-pulse" />
      </div>
    </div>
    <div v-else class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">easytier 公共中继节点</h3>
      <div class="divide-y divide-gray-200">
        <div class="px-5 py-4">
          <div class="flex items-center justify-between mb-2">
            <div>
              <p class="text-sm font-medium text-gray-900">中继节点列表</p>
              <p class="text-xs text-gray-500 mt-0.5">公网组网时用于穿越 NAT，每行一个节点</p>
            </div>
            <Button type="outline" size="small" @click="handleSavePeers">
              <template #icon><ServerStackIcon class="w-4 h-4" /></template>
              保存
            </Button>
          </div>
          <Input
            v-model="peersText"
            textarea
            :rows="4"
            placeholder="tcp://relay.example.com:11010&#10;udp://relay.example.com:11010"
            class="font-mono"
          />
          <div class="mt-2 flex items-center justify-between">
            <span class="text-xs text-gray-400">
              <template v-if="peersSaved">已保存（对新建的虚拟网络生效）</template>
              <template v-else>留空则不指定 --peers 参数</template>
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- 设备管理 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">设备管理</h3>
      <div class="divide-y divide-gray-200">
        <!-- 登出 -->
        <div class="px-5 py-4 flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">登出设备</p>
            <p class="text-xs text-gray-500 mt-0.5">撤销当前 JWT，保留本地密钥，可重新登录</p>
          </div>
          <Button
            type="outline"
            size="small"
            :loading="onlineStore.loading"
            :disabled="!onlineStore.deviceStatus?.logged_in"
            @click="handleLogout"
          >
            <template #icon><ArrowRightOnRectangleIcon class="w-4 h-4" /></template>
            登出
          </Button>
        </div>
        <!-- 清除凭证 -->
        <div class="px-5 py-4 flex items-center justify-between gap-4">
          <div class="min-w-0">
            <p class="text-sm font-medium text-gray-900">清除设备凭证</p>
            <p class="text-xs text-gray-500 mt-0.5">删除本地密钥与 JWT，需要重新注册设备</p>
          </div>
          <Button
            type="outline"
            size="small"
            :loading="onlineStore.loading"
            :disabled="!onlineStore.deviceStatus?.registered"
            @click="handleClearCredentials"
          >
            <template #icon><TrashIcon class="w-4 h-4" /></template>
            清除
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
