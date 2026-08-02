/**
 * Frp store 日志切片（阶段三）
 *
 * 从 stores/frp.ts 抽取的日志相关 state + actions，按 Pinia setup store 的
 * composable 切片模式组织，返回独立的日志 state 与 actions，由主 store 解构合并。
 *
 * 切片内部闭环：
 * - 日志页状态（logs 实时事件 + 历史读取合并）自持，无跨切片依赖
 * - 读取失败时清空日志与 hasMore，保证 UI 不显示过期数据
 */

import { ref } from 'vue'
import type { LogFileInfo } from '@/types/frp'
import {
  listLogFiles as apiListLogFiles,
  readLogFile as apiReadLogFile,
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
  }
}
