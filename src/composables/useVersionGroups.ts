/**
 * 资源版本分组逻辑
 * 参考 PCL2 PageDownloadCompDetail.GetGroupedVersionName / UpdateFilterResult
 *
 * 将版本按游戏版本号分组，支持折叠/展开、版本筛选
 */

import { computed, ref } from 'vue'
import type { ResourceVersion } from '@/types/community'
import { ModLoaderFlags } from '@/types/community'

/** 版本分组卡片 */
export interface VersionGroup {
  /** 卡片标题，如 "1.20.1" / "Fabric 1.20.1" / "快照版" / "远古版" / "其他" */
  title: string
  /** 组内版本列表（按发布时间降序） */
  versions: ResourceVersion[]
  /** 是否展开 */
  expanded: boolean
}

/**
 * 判断是否为标准版本号格式（参考 PCL2 IsFormatFit）
 * 匹配 1.x 或 2x.x 新格式
 */
function isFormatFit(name: string): boolean {
  return /^1\.\d/.test(name) || (/^[2-9]\d\.\d+/.test(name) && parseInt(name) >= 26)
}

/**
 * 将游戏版本号转为分组名（参考 PCL2 GetGroupedVersionName）
 * - 含 "w" 或无法识别 → "快照版"
 * - 非标准格式 → "远古版"
 * - 标准 → 原样返回
 */
function getGroupedVersionName(name: string): string {
  if (!name) return '其他'
  if (name.includes('w')) return '快照版'
  if (!isFormatFit(name)) return '远古版'
  return name
}

/** 提取加载器名称 */
function loaderNames(flags: number): string[] {
  const list: string[] = []
  if (flags & ModLoaderFlags.Forge) list.push('Forge')
  if (flags & ModLoaderFlags.NeoForge) list.push('NeoForge')
  if (flags & ModLoaderFlags.Fabric) list.push('Fabric')
  if (flags & ModLoaderFlags.Quilt) list.push('Quilt')
  return list
}

/** 比较版本号大小（参考 PCL2 CompareVersion） */
function compareVersion(a: string, b: string): number {
  const parseVer = (s: string) => s.split('.').map(n => parseInt(n) || 0)
  const pa = parseVer(a)
  const pb = parseVer(b)
  const len = Math.max(pa.length, pb.length)
  for (let i = 0; i < len; i++) {
    const va = pa[i] || 0
    const vb = pb[i] || 0
    if (va !== vb) return va - vb
  }
  return 0
}

/** 特殊分组排序权重（特殊类排后） */
function specialWeight(title: string): number {
  if (title === '快照版') return 1
  if (title === '远古版') return 2
  if (title === '其他') return 3
  return 0
}

/**
 * 对版本列表进行分组（带版本筛选）
 */
export function useVersionGroups(versions: () => ResourceVersion[]) {
  const expandedSet = ref<Set<string>>(new Set())
  /** 当前选中的筛选版本（空字符串=全部） */
  const versionFilter = ref('')

  /** 可用筛选项（从所有版本提取去重） */
  const filterOptions = computed<string[]>(() => {
    const vlist = versions()
    if (vlist.length === 0) return []
    const set = new Set<string>()
    for (const ver of vlist) {
      const gvs = ver.game_versions.length > 0 ? ver.game_versions : ['其他']
      for (const gv of gvs) {
        set.add(getGroupedVersionName(gv))
      }
    }
    // 排序：标准版本号降序，特殊类排后
    return Array.from(set).sort((a, b) => {
      const wa = specialWeight(a)
      const wb = specialWeight(b)
      if (wa !== wb) return wa - wb
      return compareVersion(b, a)
    })
  })

  const groups = computed<VersionGroup[]>(() => {
    const vlist = versions()
    if (vlist.length === 0) return []

    const dict = new Map<string, ResourceVersion[]>()

    for (const ver of vlist) {
      const loaders = loaderNames(ver.mod_loaders)
      const gvs = ver.game_versions.length > 0 ? ver.game_versions : ['其他']
      for (const gv of gvs) {
        const verName = getGroupedVersionName(gv)
        // 版本筛选：如果选中了筛选，且当前分组名不匹配，跳过
        if (versionFilter.value && versionFilter.value !== '全部' && verName !== versionFilter.value) continue
        // 如果有多个加载器，每个加载器独立分组（如 "Fabric 1.20.1"）
        const prefixes = loaders.length > 0 ? loaders : ['']
        for (const prefix of prefixes) {
          const title = prefix ? `${prefix} ${verName}` : verName
          if (!dict.has(title)) dict.set(title, [])
          // 避免同一版本被重复加入同一组
          const existing = dict.get(title)!
          if (!existing.some(v => v.id === ver.id)) {
            existing.push(ver)
          }
        }
      }
    }

    // 转为数组并排序
    const result: VersionGroup[] = []
    for (const [title, vers] of dict) {
      // 组内按发布时间降序
      vers.sort((a, b) => b.release_date.localeCompare(a.release_date))
      result.push({
        title,
        versions: vers,
        expanded: expandedSet.value.has(title),
      })
    }

    // 排序：标准版本号降序，特殊类排后
    result.sort((a, b) => {
      const wa = specialWeight(a.title)
      const wb = specialWeight(b.title)
      if (wa !== wb) return wa - wb
      // 都是标准版本号或同类特殊
      // 提取版本号部分（去掉加载器前缀）
      const va = a.title.replace(/^(Fabric|Forge|NeoForge|Quilt)\s+/, '')
      const vb = b.title.replace(/^(Fabric|Forge|NeoForge|Quilt)\s+/, '')
      return compareVersion(vb, va) // 降序，新版本在前
    })

    // 如果只有一组，自动展开
    if (result.length === 1) {
      result[0].expanded = true
      expandedSet.value.add(result[0].title)
    }

    return result
  })

  function toggleGroup(title: string) {
    if (expandedSet.value.has(title)) {
      expandedSet.value.delete(title)
    } else {
      expandedSet.value.add(title)
    }
  }

  function setFilter(f: string) {
    versionFilter.value = f
    expandedSet.value.clear()
  }

  return { groups, filterOptions, versionFilter, toggleGroup, setFilter }
}
