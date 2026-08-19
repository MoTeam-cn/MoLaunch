/**
 * easytier 内核安装 composable（联机页搭桥前置依赖）
 *
 * 搭桥联机（房主 hostStart / 房客 probe）前确保内核已安装：
 * 未安装时自动下载安装，进度经 `easytier-install-progress` 事件驱动，支持取消。
 * 进度弹窗由 EasyTierInstallModal 独立监听事件展示，本 composable 只负责触发与状态。
 */
import { onMounted, ref } from 'vue'
import { useTauriEvent } from '@/composables/useTauriEvent'
import {
  cancelEasyTierInstall,
  getEasyTierInstallStatus,
  installEasyTier,
} from '@/utils/api/online-manager/easytier'
import type { EasyTierInstallProgress } from '@/types/online'

export function useEasyTierInstall() {
  const installing = ref(false)
  const progress = ref<EasyTierInstallProgress | null>(null)

  const progressEvent = useTauriEvent<EasyTierInstallProgress>(
    'easytier-install-progress',
    (p) => {
      progress.value = p
      if (p.phase === 'done' || p.phase === 'error') {
        progress.value = null
        installing.value = false
      }
    },
  )
  onMounted(() => progressEvent.start())

  /** 确保已安装：未安装则自动下载安装，返回是否就绪 */
  async function ensureInstalled(): Promise<{ ok: boolean; error?: string }> {
    try {
      const status = await getEasyTierInstallStatus()
      if (status.installed) return { ok: true }
      installing.value = true
      await installEasyTier()
      return { ok: true }
    } catch (e) {
      return { ok: false, error: e instanceof Error ? e.message : String(e) }
    } finally {
      installing.value = false
    }
  }

  /** 取消安装（下载链实时中断，进度弹窗随之隐藏） */
  async function cancel(): Promise<void> {
    try {
      await cancelEasyTierInstall()
    } catch {
      // 下载链可能已结束，静默
    }
  }

  return { installing, progress, ensureInstalled, cancel }
}