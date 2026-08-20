<script setup lang="ts">
/**
 * 设置-联机 - easytier 内核卡片（外部下载安装 + 进度展示）
 */
import { computed, onMounted, ref, defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const Alert = defineAsyncComponent(() => import('@/components/common/Alert.vue'))
import { toastSuccess, toastError } from '@/utils/toast'
import { useTauriEvent } from '@/utils/tauriEvent'
import {
  getEasyTierInstallStatus,
  installEasyTier,
  updateEasyTier,
  cancelEasyTierInstall,
} from '@/utils/api/online-manager/easytier'
import type { EasyTierInstallProgress, EasyTierInstallStatus } from '@/types/online'
import { ArrowDownTrayIcon, ArrowPathIcon } from '@heroicons/vue/24/outline'

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
    // 用户主动取消时后端返回「下载已取消」，不再提示错误（error 事件已清理进度）
    if (String(e).includes('取消')) {
      toastSuccess('已取消下载')
    } else {
      toastError(`操作失败: ${e}`)
    }
  } finally {
    installBusy.value = false
    await refreshInstallStatus()
  }
}

/** 取消内核安装/更新（下载/解压阶段可点） */
async function handleCancelInstall() {
  try {
    await cancelEasyTierInstall()
  } catch (e) {
    toastError(`取消失败: ${e}`)
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

onMounted(() => {
  installProgressEvent.start()
  void refreshInstallStatus()
})
</script>

<template>
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
          <!-- 下载/解压期间显示取消按钮（主按钮保持 loading 不可点） -->
          <Button v-if="showProgress" type="outline" size="small" @click="handleCancelInstall">
            取消
          </Button>
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
          <Alert
            variant="soft"
            type="warning"
            message="受网络环境影响，GitHub 在部分地区的访问可能不稳定。若内核下载出现速度慢或进度卡顿，属正常现象，请稍安勿躁，下载完成后会自动继续安装。"
            class="mt-3"
          />
        </div>
      </div>
    </div>
  </div>
</template>