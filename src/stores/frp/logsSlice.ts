/**
 * Frp store 日志切片（阶段三）
 *
 * 日志状态（实时事件 + 历史读取合并）自持，无跨切片依赖；读取失败时清空日志与 hasMore。
 */

import { ref } from 'vue'
import type { LogFileInfo } from '@/types/frp'
import {
  listLogFiles as apiListLogFiles,
  readLogFile as apiReadLogFile,
  clearLogFile as apiClearLogFile,
} from '@/utils/api/frp-manager'
import { toastError } from '@/utils/toast'

/** 创建 Frp 日志切片（无外部依赖） */
export function useFrpLogsSlice() {
  /** 当前日志页显示的日志行（实时事件 + 历史读取合并） */
  const logs = ref<string[]>([])
  /** 日志读取中 */
  const logsLoading = ref(false)
  /** 日志页选中的隧道 ID（空字符串=全部） */
  const selectedLogTunnelId = ref('')
  /** 所有隧道的日志文件信息（用于日志页隧道下拉选项） */
  const logFiles = ref<LogFileInfo[]>([])
  /** 日志读取时后端返回的 hasMore（指示是否还有更早的历史日志） */
  const logsHasMore = ref(false)

  /** 加载所有隧道的日志文件信息（用于日志页隧道下拉选项） */
  async function loadLogFiles(): Promise<void> {
    try {
      logFiles.value = await apiListLogFiles()
    } catch (e) {
      toastError('加载日志文件列表失败：' + e)
    }
  }

  /** 读取指定隧道的历史日志（tunnelId 为空时后端返回空数组） */
  async function readLogs(tunnelId: string, maxLines?: number): Promise<void> {
    logsLoading.value = true
    try {
      const content = await apiReadLogFile(tunnelId, maxLines)
      logs.value = content.lines
      logsHasMore.value = content.hasMore
    } catch (e) {
      toastError('读取日志失败：' + e)
      logs.value = []
      logsHasMore.value = false
    } finally {
      logsLoading.value = false
    }
  }

  /** 清空当前日志页显示的日志行（仅清前端缓存，不删后端文件） */
  function clearLogs(): void {
    logs.value = []
    logsHasMore.value = false
  }

  /** 清空后端日志文件内容（tunnelId 为空时清空全部），并同步清空前端显示 */
  async function clearLogFile(tunnelId: string): Promise<void> {
    try {
      await apiClearLogFile(tunnelId)
      logs.value = []
      logsHasMore.value = false
      // 刷新日志文件列表（大小清零）
      await loadLogFiles()
    } catch (e) {
      toastError('清空日志失败：' + e)
    }
  }

  return {
    // state
    logs,
    logsLoading,
    selectedLogTunnelId,
    logFiles,
    logsHasMore,
    // actions
    loadLogFiles,
    readLogs,
    clearLogs,
    clearLogFile,
  }
}
