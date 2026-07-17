/**
 * 资源版本分组逻辑
 * 参考 PCL2 PageDownloadCompDetail.GetGroupedVersionName / UpdateFilterResult
 *
 * 将版本按游戏版本号分组，支持折叠/展开、版本筛选
 *
 * 性能优化：expanded 状态独立于 groups computed，toggleGroup 只触发 expanded 变化，
 * 不重新计算 groups，避免 v-for 全部重渲染导致展开/收起动画卡顿。
 */

import { computed, nextTick, reactive, ref, shallowRef, watch } from 'vue'
import type { ResourceVersion } from '@/types/community'
import { ModLoaderFlags } from '@/types/community'

/** 版本分组卡片 */
export interface VersionGroup {
  /** 卡片标题，如 "1.20.1" / "Fabric 1.20.1" / "快照版" / "远古版" / "其他" */
  title: string
  /** 组内版本列表（按发布时间降序） */
  versions: ResourceVersion[]
}

/**
 * 顶部筛选滑块用的版本号（截断到二级）
 * - 1.12.2 → "1.12"，1.20.1 → "1.20"，26.1.3 → "26.1"
 * - 低于 1.12 的版本统一归"远古版"（顶部筛选项不显示 1.8/1.9/1.10/1.11 各项，避免过多）
 * - 含 "w" → "快照版"
 * - 新格式快照版：26.2-snapshot-2 / 26.2-snapshot-3 → "26.2"（与正式版 26.2 归为同一 tag）
 * - 非标准格式 → "远古版"
 * - 空 → "其他"
 *
 * 导出供外部调用：例如从 ModTab 打开资源详情弹窗时，根据整合包的 MC 版本号
 * 自动选中顶部筛选 tag，需要用本函数把 "1.20.1" 转成 "1.20"。
 */
export function getFilterVersionName(name: string): string {
  if (!name) return '其他'
  if (name.includes('w')) return '快照版'
  const m = /^1\.(\d+)/.exec(name)
  if (m) {
    const minor = parseInt(m[1])
    if (minor < 12) return '远古版' // 低于 1.12 合并成单个远古版标签
    return `1.${minor}`
  }
  // 新格式（26.x）：截断到二级版本号
  // 先去掉 -snapshot-数字 等后缀（26.2-snapshot-2 → 26.2），再截断到二级（26.2.1 → 26.2）
  if (/^[2-9]\d\.\d+/.test(name) && parseInt(name) >= 26) {
    const base = name.split('-')[0]
    return base.split('.').slice(0, 2).join('.')
  }
  return '远古版'
}

/**
 * 下面版本分组卡片用的版本号（所有标准版本保留完整版本号）
 * - 1.12.2 → "1.12.2"，1.10.1 → "1.10.1"，1.8.9 → "1.8.9"，26.1.3 → "26.1.3"
 * - 含 "w" → "快照版"
 * - 非标准格式（无法识别的版本号）→ "远古版"
 * - 空 → "其他"
 *
 * 注：顶部筛选滑块用 getFilterVersionName 截断到二级，下面分组卡片用本函数保留完整版本号
 */
function getGroupedVersionName(name: string): string {
  if (!name) return '其他'
  if (name.includes('w')) return '快照版'
  // 标准 1.x 格式：保留完整版本号（1.10.1 → "1.10.1"，1.12.2 → "1.12.2"）
  if (/^1\.\d+/.test(name)) return name
  // 新格式（26.x）：保留完整版本号
  if (/^[2-9]\d\.\d+/.test(name) && parseInt(name) >= 26) return name
  // 非标准格式归远古版
  return '远古版'
}

/** 提取加载器名称（顺序与 useVersionMeta.ts 的 typeMetaMap.order 对齐：Forge→NeoForge→Fabric→Quilt→LiteLoader） */
function loaderNames(flags: number): string[] {
  const list: string[] = []
  if (flags & ModLoaderFlags.Forge) list.push('Forge')
  if (flags & ModLoaderFlags.NeoForge) list.push('NeoForge')
  if (flags & ModLoaderFlags.Fabric) list.push('Fabric')
  if (flags & ModLoaderFlags.Quilt) list.push('Quilt')
  if (flags & ModLoaderFlags.LiteLoader) list.push('LiteLoader')
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

/** 计算分组（不依赖 expanded 状态） */
function computeGroups(versions: ResourceVersion[], versionFilter: string): VersionGroup[] {
  if (versions.length === 0) return []

  const dict = new Map<string, ResourceVersion[]>()

  for (const ver of versions) {
    const loaders = loaderNames(ver.mod_loaders)
    const gvs = ver.game_versions.length > 0 ? ver.game_versions : ['其他']
    for (const gv of gvs) {
      // 筛选用截断到二级的版本号匹配（1.12 匹配 1.12/1.12.1/1.12.2）
      const filterName = getFilterVersionName(gv)
      if (versionFilter && versionFilter !== '全部' && filterName !== versionFilter) continue
      // 分组用完整版本号（1.12.2 保持 1.12.2）
      const verName = getGroupedVersionName(gv)
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
    result.push({ title, versions: vers })
  }

  // 排序：标准版本号降序，特殊类排后
  result.sort((a, b) => {
    const wa = specialWeight(a.title)
    const wb = specialWeight(b.title)
    if (wa !== wb) return wa - wb
    // 都是标准版本号或同类特殊
    // 提取版本号部分（去掉加载器前缀）
    const va = a.title.replace(/^(Fabric|Forge|NeoForge|Quilt|LiteLoader)\s+/, '')
    const vb = b.title.replace(/^(Fabric|Forge|NeoForge|Quilt|LiteLoader)\s+/, '')
    return compareVersion(vb, va) // 降序，新版本在前
  })

  return result
}

/**
 * 对版本列表进行分组（带版本筛选）
 *
 * expanded 状态独立存储，toggleGroup 只修改 expandedMap，
 * 不触发 groups 重新计算，避免 v-for 全部重渲染导致展开/收起动画卡顿。
 */
export function useVersionGroups(versions: () => ResourceVersion[]) {
  /** 当前选中的筛选版本（空字符串=全部） */
  const versionFilter = ref('')

  /** 可用筛选项（从所有版本提取去重，截断到二级版本号） */
  const filterOptions = computed<string[]>(() => {
    const vlist = versions()
    if (vlist.length === 0) return []
    const set = new Set<string>()
    for (const ver of vlist) {
      const gvs = ver.game_versions.length > 0 ? ver.game_versions : ['其他']
      for (const gv of gvs) {
        set.add(getFilterVersionName(gv))
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

  /** 展开状态：独立 reactive 对象，toggleGroup 只改这里，不触发 groups 重算 */
  const expandedMap = reactive<Record<string, boolean>>({})

  /** 内容懒挂载标记：首次展开后才渲染版本条目 DOM，折叠卡片不产生内容 DOM，避免大量兄弟节点拖慢 reflow */
  const mountedMap = reactive<Record<string, boolean>>({})

  // groups 用 shallowRef + watch 重建，只在 versions/filter 变化时重新计算
  // 不依赖 expanded 状态，避免 toggleGroup 触发 groups 重算
  // 注意：expandedMap/mountedMap 必须在 watch 之前声明，因为 immediate+sync 会在声明时同步执行回调
  const groups = shallowRef<VersionGroup[]>([])
  watch(
    [() => versions(), versionFilter],
    ([vlist, vf]) => {
      groups.value = computeGroups(vlist, vf)
      // 数据/筛选变化时清空挂载与展开状态
      for (const k of Object.keys(expandedMap)) delete expandedMap[k]
      for (const k of Object.keys(mountedMap)) delete mountedMap[k]
      // 如果只有一组，自动展开（需先挂载再展开，保证动画）
      if (groups.value.length === 1) {
        const title = groups.value[0].title
        mountedMap[title] = true
        nextTick(() => { expandedMap[title] = true })
      }
    },
    { immediate: true, flush: 'sync' },
  )

  /** 可响应的 expanded 读取（模板用 expandedOf(g.title)） */
  function expandedOf(title: string): boolean {
    return expandedMap[title] === true
  }

  /** 内容是否已挂载（模板用 mountedOf(g.title) 控制 v-if） */
  function mountedOf(title: string): boolean {
    return mountedMap[title] === true
  }

  /** 展开/收起：首次展开时先挂载内容（保持 0fr 折叠态），下一 tick 再展开触发 0fr→1fr 动画 */
  async function toggleGroup(title: string) {
    const willExpand = !expandedMap[title]
    if (willExpand && !mountedMap[title]) {
      mountedMap[title] = true
      await nextTick()
    }
    expandedMap[title] = willExpand
  }

  function setFilter(f: string) {
    versionFilter.value = f
    // watch 会清空 mounted/expanded，无需重复处理
  }

  return { groups, filterOptions, versionFilter, toggleGroup, setFilter, expandedOf, mountedOf }
}
