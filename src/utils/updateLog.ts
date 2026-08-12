/**
 * 启动「本次更新日志」弹窗控制
 *
 * 对齐 PCL2 的做法：启动时比较 localStorage 记录的上次运行版本与当前版本，
 * 仅当版本升高时弹出一次更新日志，弹窗前先写入当前版本（防弹窗期间崩溃重复弹出）。
 * 全新安装（无记录）不弹；版本回退（如测试版退回正式版）不弹。
 *
 * 日志内容由 vite.config.ts 的 updateLogPlugin 在构建时读取 CHANGELOG.md，
 * 仅内联当前版本对应的段落（`virtual:update-log`），不打包整份 Markdown。
 */
import { ref } from 'vue'
import updateLogContent, {
  version as updateLogVersion,
  notes as updateLogNotes,
} from 'virtual:update-log'
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
 * 规则（对齐 PCL2 UpgradeSub）：首次运行仅记录当前版本（不弹窗，供后续升级对比）；
 * 同版本 / 版本回退不弹；仅当上次版本低于当前版本时弹，且弹窗前先记录当前版本，保证只弹一次。
 */
export function maybeShowUpdateLog(): void {
  const current = currentVersion()
  const lastSeen = readLastSeen()
  if (lastSeen === null) {
    writeLastSeen(current)
    return
  }
  if (lastSeen === current) return
  if (compareVersion(lastSeen, current) > 0) return
  writeLastSeen(current)
  updateLogVisible.value = true
}

/** 更新日志内容（vite 构建时从 CHANGELOG.md 提取的当前版本段落） */
export function getChangelogContent(): string {
  return updateLogContent
}

/** 更新日志对应的版本号（与内容同源，可能回退到最新发布版本） */
export function getChangelogVersion(): string {
  return updateLogVersion
}

/** 作者的话列表（vite 构建时从 git 提交中提取 `note:` 前缀的 commit，可为空数组） */
export function getChangelogNotes(): string[] {
  return updateLogNotes
}

/** 从 Markdown 内容中提取 `note:` 前缀行（服务端 release_notes 可选格式），返回文本列表 */
export function extractNoteLines(markdown: string): string[] {
  const notes: string[] = []
  for (const line of markdown.split('\n')) {
    const m = line.trim().match(/^note:\s*(.*)$/i)
    if (m && m[1].trim()) notes.push(m[1].trim())
  }
  return notes
}

/** 剔除 Markdown 中的 `note:` 前缀行（避免与提取出的高亮块重复展示） */
export function stripNoteLines(markdown: string): string {
  return markdown
    .split('\n')
    .filter((line) => !/^\s*note:\s*/i.test(line))
    .join('\n')
}

/** 直接弹出更新日志弹窗（dev-api 测试用） */
export function showUpdateLog(): void {
  updateLogVisible.value = true
}

/** 清空更新日志已读记录（dev-api 测试用：下次启动将重新弹出） */
export function resetUpdateLogRecord(): void {
  try {
    localStorage.removeItem(STORAGE_KEY)
  } catch {
    // 静默忽略
  }
}

/** 手动关闭弹窗 */
export function closeUpdateLog(): void {
  updateLogVisible.value = false
}
