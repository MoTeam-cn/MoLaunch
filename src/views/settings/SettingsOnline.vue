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
  setGithubProxies,
} from '@/utils/api/online-manager/easytier'
import { getConfigMap, refreshConfig } from '@/utils/tauri'
import { restoreDefaultProxies } from '@/utils/githubProxy'
import type { EasyTierInstallProgress, EasyTierInstallStatus } from '@/types/online'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const ApiServerCard = defineAsyncComponent(() => import('@/components/settings/ApiServerCard.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const AlertV2 = defineAsyncComponent(() => import('@/components/common/AlertV2.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import {
  ArrowDownTrayIcon,
  ArrowPathIcon,
  ArrowRightOnRectangleIcon,
  CheckIcon,
  CloudIcon,
  PlusIcon,
  ServerStackIcon,
  TrashIcon,
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

const hasUpdate = computed(() => {
  const s = installStatus.value
  return !!s?.installed && !!s.latestVersion && s.version !== s.latestVersion
})
/** 进度展示：download / extract 阶段事件驱动（done/error 清除），避免状态轮询 */
const showProgress = computed(() => {
  const ph = installProgress.value?.phase
  return ph === 'download' || ph === 'extract'
})

/** 状态 Tag：检查中 gray / 下载中 blue / 未安装 red / 有新版本 gold / 已安装 green */
const tagColor = computed(() => {
  if (!installStatus.value) return 'gray'
  if (showProgress.value) return 'blue'
  if (!installStatus.value.installed) return 'red'
  if (hasUpdate.value) return 'gold'
  return 'green'
})

const tagText = computed(() => {
  if (showProgress.value) {
    const ph = installProgress.value?.phase
    const prefix = ph === 'extract' ? '解压安装' : '下载中'
    return `${prefix} ${installProgress.value?.percent ?? 0}%`
  }
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

// ============ GitHub 镜像源管理 ============
/** 编辑行（name 必填字符串，便于 v-model 绑定） */
interface ProxyRow {
  name: string
  type: 'path' | 'full'
  base: string
}

const githubProxies = ref<ProxyRow[]>([])
const proxiesLoaded = ref(false)
/** 重新测速 / 保存 独立加载态（互不干扰：点重新测速时保存按钮保持可用） */
const probeBusy = ref(false)
const saveBusy = ref(false)

/** type 选项（full: 镜像 + 完整 GitHub URL / path: 镜像前缀 + GitHub 路径），文案精简以适配窄列 */
const proxyTypeOptions = [
  { label: '追加路径', value: 'path' },
  { label: '完整 URL', value: 'full' },
]

async function loadGithubProxies() {
  try {
    const cfg = await getConfigMap(true)
    githubProxies.value = (cfg.onlineGithubProxies ?? []).map((p) => ({
      name: p.name ?? '',
      type: p.type,
      base: p.base,
    }))
  } catch (e) {
    console.error('加载 GitHub 镜像源失败', e)
  } finally {
    proxiesLoaded.value = true
  }
}

function addProxyRow() {
  githubProxies.value.push({ name: '', type: 'path', base: '' })
}

function removeProxyRow(index: number) {
  githubProxies.value.splice(index, 1)
}

/** 保存：过滤空 base 行，写运行时缓存 + 持久化配置 */
async function handleSaveProxies() {
  const list = githubProxies.value
    .filter((p) => p.base.trim().length > 0)
    .map((p) => ({ name: p.name?.trim() || undefined, type: p.type, base: p.base.trim() }))
  if (list.length === 0) {
    toastError('至少保留一个镜像源')
    return
  }
  saveBusy.value = true
  try {
    await setGithubProxies(list)
    refreshConfig()
    toastSuccess('镜像源已保存')
  } catch (e) {
    toastError(`保存失败: ${e}`)
  } finally {
    saveBusy.value = false
  }
}

/** 重新测速：对 githubProxy.json 全部源并发探测，筛选最快的前 10 个替换当前列表 */
async function handleRestoreDefaultProxies() {
  probeBusy.value = true
  try {
    const list = await restoreDefaultProxies()
    if (list.length === 0) {
      toastError('默认镜像源均不可用，请稍后重试')
    } else {
      githubProxies.value = list.map((p) => ({ name: p.name ?? '', type: p.type, base: p.base }))
      refreshConfig()
      toastSuccess(`已重新测速，筛选出 ${list.length} 个可用镜像源`)
    }
  } catch (e) {
    toastError(`重新测速失败: ${e}`)
  } finally {
    probeBusy.value = false
  }
}

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
  void loadGithubProxies()
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
          <div class="mt-3 flex items-center justify-end gap-4">
            <Button
              :type="installStatus?.installed && !hasUpdate ? 'outline' : 'primary'"
              size="small"
              :loading="installBusy || showProgress"
              :disabled="!installStatus || showProgress"
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
            <!-- 下载安抚提示：GitHub 部分地区网络不稳定时避免用户干等 -->
            <AlertV2
              type="warning"
              message="受网络环境影响，GitHub 在部分地区的访问可能不稳定。若内核下载出现速度慢或进度卡顿，属正常现象，请稍安勿躁，下载完成后会自动继续安装。"
              class="mt-3"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- GitHub 镜像源（easytier 等外部下载竞速选源） -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">GitHub 镜像源</h3>
      <div class="px-5 pb-5">
        <p class="text-xs text-gray-500">
          easytier 内核等外部文件下载时按镜像优先、官方保底竞速选源。path 模式为「镜像前缀 + GitHub 路径」，full 模式为「镜像 + 完整 GitHub URL」。
        </p>
        <div v-if="!proxiesLoaded" class="mt-3 h-20 bg-gray-100 rounded animate-pulse" />
        <template v-else>
          <!-- 表头：grid 固定列宽（名称/类型固定，镜像地址收窄），与输入行对齐 -->
          <div
            class="mt-3 grid grid-cols-[6rem_7rem_12rem_auto] items-center gap-2 text-xs text-gray-400"
          >
            <span>名称</span>
            <span>类型</span>
            <span>镜像地址</span>
            <span />
          </div>
          <div
            v-for="(p, i) in githubProxies"
            :key="i"
            class="mt-2 grid grid-cols-[6rem_7rem_12rem_auto] items-center gap-2"
          >
            <Input v-model="p.name" placeholder="可选" size="small" width="100%" />
            <Select v-model="p.type" :options="proxyTypeOptions" />
            <Input v-model="p.base" placeholder="https://mirror.example.com" size="small" width="100%" />
            <Button type="text" size="small" @click="removeProxyRow(i)">删除</Button>
          </div>
          <div
            v-if="githubProxies.length === 0"
            class="mt-3 py-8 flex flex-col items-center justify-center"
          >
            <CloudIcon class="w-8 h-8 text-gray-300" />
            <p class="mt-2 text-xs text-gray-400">暂无镜像源，可添加自定义或恢复默认</p>
          </div>
          <div class="mt-4 flex items-center justify-between gap-2">
            <Button type="outline" size="small" :disabled="probeBusy || saveBusy" @click="addProxyRow">
              <template #icon><PlusIcon class="w-4 h-4" /></template>
              添加镜像
            </Button>
            <div class="flex items-center gap-2">
              <Tooltip text="对内置镜像源清单全部并发测速，筛选响应最快的前 10 个替换当前列表" position="top">
                <Button type="outline" size="small" :loading="probeBusy" @click="handleRestoreDefaultProxies">
                  <template #icon><ArrowPathIcon class="w-4 h-4" /></template>
                  重新测速
                </Button>
              </Tooltip>
              <Button type="primary" size="small" :loading="saveBusy" @click="handleSaveProxies">
                <template #icon><CheckIcon class="w-4 h-4" /></template>
                保存
              </Button>
            </div>
          </div>
        </template>
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
