/**
 * Mod 安装前预检（纯函数，便于测试）
 *
 * 在真正下载之前，用已有的本地数据判断这次安装会不会装错：
 * - 阻断项（blocker）：加载器不匹配、游戏版本不匹配 —— 装了也起不来
 * - 警告项（warning）：同 modid 已装、同名文件将被覆盖、实例加载器混杂、OptiFine 与 Fabric 系冲突
 *
 * 前置依赖缺失不在这里判断（由后端 check_mod_dependencies 负责）。
 */
import { ModLoaderFlags } from '@/types/community'

/** 问题级别 */
export type PrecheckLevel = 'blocker' | 'warning'

/** 一条预检结论 */
export interface PrecheckIssue {
  /** 规则标识（便于测试与埋点） */
  code: string
  level: PrecheckLevel
  /** 标题（一句话） */
  title: string
  /** 详细说明（可多行） */
  detail: string
}

/** 待安装的目标文件信息 */
export interface PrecheckTarget {
  file_name: string
  /** 加载器位掩码（0 表示未知） */
  mod_loaders: number
  /** 支持的游戏版本（可能包含加载器名，匹配时会跳过非版本号项） */
  game_versions: string[]
}

/** 已安装 Mod 的精简信息（来自 listMods） */
export interface InstalledModRef {
  file_name: string
  /** 启用时的文件名（去掉 .disabled） */
  enabled_name?: string
  is_enabled: boolean
  /** jar 内 modid（fabric.mod.json id / mods.toml modId） */
  slug: string
  /** 加载器类型（forge/fabric/neoforge/liteloader/unknown） */
  loader_type: string
}

/** 预检输入 */
export interface PrecheckInput {
  target: PrecheckTarget
  installed: InstalledModRef[]
  /** 实例的 MC 版本（如 1.20.1） */
  instanceGameVersion: string
  /** 目标 Mod 的项目 slug（用于判重） */
  projectSlug?: string
  /** 实例加载器位掩码（不传时按已装 Mod 推断） */
  instanceLoader?: number
}

const LOADER_NAMES: Array<[number, string]> = [
  [ModLoaderFlags.Forge, 'Forge'],
  [ModLoaderFlags.NeoForge, 'NeoForge'],
  [ModLoaderFlags.Fabric, 'Fabric'],
  [ModLoaderFlags.Quilt, 'Quilt'],
  [ModLoaderFlags.LiteLoader, 'LiteLoader'],
]

/** 位掩码转可读名字 */
export function loaderNames(flags: number): string[] {
  return LOADER_NAMES.filter(([flag]) => (flags & flag) !== 0).map(([, name]) => name)
}

/** 单个 mod 的 loader_type 映射到位掩码 */
function loaderTypeToFlag(loaderType: string): number {
  switch (loaderType.toLowerCase()) {
    case 'forge':
      return ModLoaderFlags.Forge
    case 'neoforge':
      return ModLoaderFlags.NeoForge
    case 'fabric':
      return ModLoaderFlags.Fabric
    case 'quilt':
      return ModLoaderFlags.Quilt
    case 'liteloader':
      return ModLoaderFlags.LiteLoader
    default:
      return ModLoaderFlags.None
  }
}

/** 按已装 Mod 的加载器类型推断实例加载器（取出现次数最多的一类） */
export function deriveInstanceLoader(installed: InstalledModRef[]): number {
  const counts = new Map<number, number>()
  for (const mod of installed) {
    const flag = loaderTypeToFlag(mod.loader_type)
    if (flag === ModLoaderFlags.None) continue
    counts.set(flag, (counts.get(flag) ?? 0) + 1)
  }
  let best: number = ModLoaderFlags.None
  let bestCount = 0
  for (const [flag, count] of counts) {
    if (count > bestCount) {
      best = flag
      bestCount = count
    }
  }
  return best
}

/** 目标文件是否支持该游戏版本（兼容 1.20 与 1.20.1 这类前缀关系） */
export function supportsGameVersion(gameVersions: string[], instanceVersion: string): boolean {
  const target = instanceVersion.trim()
  if (!target) return true
  const loaderNamesLower = LOADER_NAMES.map(([, name]) => name.toLowerCase())
  return gameVersions.some((raw) => {
    const version = raw.trim()
    if (!version) return false
    if (loaderNamesLower.includes(version.toLowerCase())) return false
    return version === target || target.startsWith(`${version}.`) || version.startsWith(`${target}.`)
  })
}

/**
 * 执行安装前预检，返回按「阻断 → 警告」排序的问题列表
 */
export function precheckModInstall(input: PrecheckInput): PrecheckIssue[] {
  const issues: PrecheckIssue[] = []
  const { target, installed, instanceGameVersion } = input

  // 1) 加载器不匹配
  const instanceLoader = input.instanceLoader ?? deriveInstanceLoader(installed)
  if (target.mod_loaders !== 0 && instanceLoader !== 0 && (target.mod_loaders & instanceLoader) === 0) {
    issues.push({
      code: 'loader_mismatch',
      level: 'blocker',
      title: '加载器不匹配',
      detail: `该文件适用于 ${loaderNames(target.mod_loaders).join(' / ') || '未知加载器'}，当前实例是 ${loaderNames(instanceLoader).join(' / ')}，装上也无法加载。`,
    })
  }

  // 2) 游戏版本不匹配
  if (target.game_versions.length > 0 && !supportsGameVersion(target.game_versions, instanceGameVersion)) {
    issues.push({
      code: 'game_version_mismatch',
      level: 'blocker',
      title: '游戏版本不匹配',
      detail: `该文件支持 ${target.game_versions.slice(0, 5).join('、')}，当前实例是 ${instanceGameVersion || '未知版本'}。`,
    })
  }

  // 3) 同一个 modid 已经装过（会变成两份）
  const projectSlug = (input.projectSlug ?? '').toLowerCase()
  if (projectSlug) {
    const sameModid = installed.filter(
      (mod) => mod.is_enabled && mod.slug.toLowerCase() === projectSlug,
    )
    if (sameModid.length > 0) {
      issues.push({
        code: 'duplicate_modid',
        level: 'warning',
        title: '同一 Mod 已安装',
        detail: `mods 目录里已有同 modid（${projectSlug}）的文件：${sameModid.map((mod) => mod.file_name).join('、')}。继续安装会出现两份同 modid，进游戏大概率报错。`,
      })
    }
  }

  // 4) 同名文件会被覆盖
  const sameFile = installed.find(
    (mod) => mod.file_name === target.file_name || mod.enabled_name === target.file_name,
  )
  if (sameFile) {
    issues.push({
      code: 'same_file_exists',
      level: 'warning',
      title: '同名文件已存在',
      detail: `mods 目录里已有 ${sameFile.file_name}，安装会覆盖它。`,
    })
  }

  // 5) 实例里同时存在 Forge 系与 Fabric 系 Mod
  const hasForgeFamily = installed.some((mod) => {
    const flag = loaderTypeToFlag(mod.loader_type)
    return (flag & (ModLoaderFlags.Forge | ModLoaderFlags.NeoForge)) !== 0
  })
  const hasFabricFamily = installed.some((mod) => {
    const flag = loaderTypeToFlag(mod.loader_type)
    return (flag & (ModLoaderFlags.Fabric | ModLoaderFlags.Quilt)) !== 0
  })
  if (hasForgeFamily && hasFabricFamily) {
    issues.push({
      code: 'mixed_loaders',
      level: 'warning',
      title: '实例里混装了 Forge 系与 Fabric 系 Mod',
      detail: '同一个版本通常只能跑一种加载器的 Mod，建议先确认这些文件是否都该存在。',
    })
  }

  // 6) OptiFine 与 Fabric 系 Mod 冲突
  const hasOptifine = installed.some((mod) =>
    `${mod.slug} ${mod.file_name}`.toLowerCase().includes('optifine'),
  )
  const targetIsFabricFamily =
    (target.mod_loaders & (ModLoaderFlags.Fabric | ModLoaderFlags.Quilt)) !== 0
  if (hasOptifine && targetIsFabricFamily) {
    issues.push({
      code: 'optifine_conflict',
      level: 'warning',
      title: 'OptiFine 与 Fabric 系 Mod 通常不兼容',
      detail: '实例里已有 OptiFine，Fabric/Quilt 的 Mod 一般无法与它共存，建议改用 Sodium 等替代方案。',
    })
  }

  return issues
}
