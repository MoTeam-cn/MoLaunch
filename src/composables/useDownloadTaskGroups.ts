/**
 * 下载任务分组逻辑
 *
 * 将下载阶段按 `group` 字段聚合：有 group 的聚合成一个任务分组卡片，
 * 无 group 的独立成卡片。计算每个分组的聚合状态、加权进度和字节累加。
 *
 * 抽取自 Downloads.vue，参考 PCL2 PageDownloads 任务分组显示。
 */
import { computed, ref } from 'vue'
import type { DownloadStage, StageStatus } from '@/types/download'

/** 下载任务分组（聚合同一 group 的多个 stage） */
export interface TaskGroup {
  /** 分组键：group 名或 stage.name（无 group 时用 name 作为独立分组） */
  key: string
  /** 显示名 */
  title: string
  /** 组内阶段列表 */
  stages: DownloadStage[]
  /** 聚合状态：组内任一 failed → failed；全部 finished → finished；任一 loading → loading；否则 waiting */
  status: StageStatus
  /** 分组加权进度（0-1，按 stage.weight 加权平均） */
  progress: number
  /** 分组已下载字节（Finished + Loading 阶段累加） */
  bytesDownloaded: number
  /** 分组总字节 */
  bytesTotal: number
  /** 是否独立阶段（无 group） */
  isIndependent: boolean
}

/**
 * 按分组聚合下载阶段
 *
 * @param stages getter 函数，返回当前所有下载阶段（通常来自 downloadProgress.stages）
 * @returns taskGroups 计算属性 + 折叠/展开控制函数
 */
export function useDownloadTaskGroups(stages: () => DownloadStage[]) {
  const taskGroups = computed<TaskGroup[]>(() => {
    const groups: TaskGroup[] = []
    const groupMap = new Map<string, TaskGroup>()

    for (const s of stages()) {
      const groupKey = s.group || s.name
      let g = groupMap.get(groupKey)
      if (!g) {
        g = {
          key: groupKey,
          title: s.group || s.name,
          stages: [],
          status: 'waiting',
          progress: 0,
          bytesDownloaded: 0,
          bytesTotal: 0,
          isIndependent: !s.group,
        }
        groupMap.set(groupKey, g)
        groups.push(g)
      }
      g.stages.push(s)
    }

    // 计算每个分组的聚合状态、加权进度和字节
    for (const g of groups) {
      const statuses = g.stages.map(s => s.status)
      if (statuses.includes('failed')) {
        g.status = 'failed'
      } else if (statuses.every(s => s === 'finished')) {
        g.status = 'finished'
      } else if (statuses.includes('loading')) {
        g.status = 'loading'
      } else {
        g.status = 'waiting'
      }
      // 加权进度（与整体 percentage 算法一致：按 stage.weight 加权平均）
      let weightedProgress = 0
      let totalWeight = 0
      for (const s of g.stages) {
        totalWeight += s.weight
        weightedProgress += s.progress * s.weight
      }
      g.progress = totalWeight > 0 ? weightedProgress / totalWeight : 0
      // 字节累加（仅 Finished + Loading 阶段，与后端 global_bytes 算法一致）
      for (const s of g.stages) {
        if (s.status === 'finished' || s.status === 'loading') {
          g.bytesDownloaded += s.bytes_downloaded
          g.bytesTotal += s.bytes_total
        }
      }
    }

    return groups
  })

  // 折叠状态：默认全展开，用户点击可折叠
  const collapsedGroups = ref<Set<string>>(new Set())

  function toggleGroup(key: string) {
    if (collapsedGroups.value.has(key)) {
      collapsedGroups.value.delete(key)
    } else {
      collapsedGroups.value.add(key)
    }
  }

  function isExpanded(key: string): boolean {
    return !collapsedGroups.value.has(key)
  }

  return { taskGroups, toggleGroup, isExpanded }
}
