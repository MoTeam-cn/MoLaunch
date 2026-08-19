/**
 * easytier 内核安装状态 composable（联机页前置依赖检查）
 *
 * 封存方案：进入创建/加入/大厅页面时检查内核状态，缺失时由 useOnlineNav 封存页面，
 * 引导用户前往 设置-联机 页面下载内核。本 composable 提供：
 * - installed：内核是否已安装（null=未知/检查中）
 * - checkStatus()：查询后端安装状态
 * - ensureKernel(label)：入口检查，未安装时弹窗引导并返回 false
 * - promptMissing(label)：内核缺失弹窗（前往设置）
 * 监听 `easytier-install-progress` 事件：安装完成（done）自动刷新 installed 解除封存。
 */
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useTauriEvent } from '@/composables/useTauriEvent'
import { getEasyTierInstallStatus } from '@/utils/api/online-manager/easytier'
import { showModal } from '@/utils/modal'
import type { EasyTierInstallProgress } from '@/types/online'

export function useEasyTierInstall() {
  const router = useRouter()
  const installed = ref<boolean | null>(null)
  const checking = ref(false)

  const progressEvent = useTauriEvent<EasyTierInstallProgress>(
    'easytier-install-progress',
    (p) => {
      // 安装完成自动解除封存（设置页下载完成后回到联机页无需手动刷新）
      if (p.phase === 'done') installed.value = true
    },
  )
  onMounted(() => progressEvent.start())

  /** 查询内核安装状态，返回是否已安装 */
  async function checkStatus(): Promise<boolean> {
    checking.value = true
    try {
      const status = await getEasyTierInstallStatus()
      installed.value = status.installed
      return status.installed
    } catch {
      installed.value = null
      return false
    } finally {
      checking.value = false
    }
  }

  /** 内核缺失弹窗：引导前往 设置-联机 页面下载 */
  function promptMissing(label: string): void {
    showModal({
      type: 'warning',
      title: '功能已封存',
      message: `「${label}」需要 easytier 内核，当前未安装。请前往 设置-联机 页面下载内核后重试。`,
      confirmText: '前往设置',
      onConfirm: () => router.push('/apps/settings?tab=online'),
    })
  }

  /** 入口检查：未安装时弹窗引导并返回 false（调用方不继续创建/加入） */
  async function ensureKernel(label: string): Promise<boolean> {
    const ok = await checkStatus()
    if (!ok) promptMissing(label)
    return ok
  }

  return { installed, checking, checkStatus, ensureKernel, promptMissing }
}