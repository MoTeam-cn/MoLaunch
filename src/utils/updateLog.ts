/**
 * 启动「本次更新日志」弹窗控制
 *
 * 对齐 PCL2 的做法：启动时比较 localStorage 记录的上次运行版本与当前版本，
 * 仅当版本升高时弹出一次更新日志，弹窗前先写入当前版本（防弹窗期间崩溃重复弹出）。
 * 全新安装（无记录）不弹；版本回退（如测试版退回正式版）不弹。
 */
import { ref } from 'vue'
import changelogMd from '../../CHANGELOG.md?raw'
import { compareVersion, getVersionInfo } from '@/utils/version'

/** localStorage 键：上次运行（已展示过更新日志）的版本号 */
const STORAGE_KEY = 'molaunch.lastSeenUpdate'

/** 完整更新日志的 GitHub Releases 地址 */
export const UPDATE_LOG_GITHUB_URL = 'https://github.com/MoTeam-cn/MoLaunch/releases'

/** 更新日志弹窗可见状态（供 UpdateLogDialog.vue 绑定） */
export const updateLogVisible = ref(false)

/** 当前应用版本号（vite define 注入 __APP_VERSION__） */
function currentVersion(): string {
  return getVersionInfo().raw
}

/** 读取上次记录版本（localStorage 不可用时返回 null） */
function readLastSeen(): string | null {
  try {
    return localStorage.getItem(STORAGE_KEY)
  } catch {
    return null
  }
}

/** 写入当前版本（写入失败忽略：下次启动再弹一次） */
function writeLastSeen(version: string): void {
  try {
    localStorage.setItem(STORAGE_KEY, version)
  } catch {
    // 静默忽略
  }
}

/**
 * 启动时检测并展示「本次更新」日志
 *
 * 规则（对齐 PCL2 UpgradeSub）：全新安装不弹；同版本 / 版本回退不弹；
 * 仅当上次版本低于当前版本时弹，且弹窗前先记录当前版本，保证只弹一次。
 */
export function maybeShowUpdateLog(): void {
  const current = currentVersion()
  const lastSeen = readLastSeen()
  if (lastSeen === null || lastSeen === current) return
  if (compareVersion(lastSeen, current) > 0) return
  writeLastSeen(current)
  updateLogVisible.value = true
}

/** 更新日志内容（CHANGELOG.md 原文，构建时经 vite ?raw 内联） */
export function getChangelogContent(): string {
  return changelogMd
}

/** 手动关闭弹窗 */
export function closeUpdateLog(): void {
  updateLogVisible.value = false
}
