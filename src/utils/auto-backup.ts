/**
 * 启动前自动备份存档（前端触发）
 *
 * - 开关存 localStorage（纯前端偏好，不改后端配置）
 * - 每个版本每天最多备份一次，只备份前 3 个存档，覆盖同名文件（不会无限膨胀）
 * - 任何失败都不阻断启动
 */
import { archiveBackup, archiveList } from '@/utils/api/tools/archive'
import { getGameDir } from '@/utils/api/system'

const SETTING_KEY = 'molaunch-auto-backup'
const STATE_KEY = 'molaunch-auto-backup-state'
const MAX_WORLDS = 3

/** 自动备份设置 */
export interface AutoBackupSetting {
  enabled: boolean
}

/** 读取设置（默认关闭） */
export function loadAutoBackupSetting(): AutoBackupSetting {
  try {
    const raw = localStorage.getItem(SETTING_KEY)
    if (!raw) return { enabled: false }
    const parsed = JSON.parse(raw) as Partial<AutoBackupSetting>
    return { enabled: parsed.enabled === true }
  } catch {
    return { enabled: false }
  }
}

/** 保存设置 */
export function saveAutoBackupSetting(setting: AutoBackupSetting): void {
  try {
    localStorage.setItem(SETTING_KEY, JSON.stringify(setting))
  } catch {
    /* 存储不可用时忽略 */
  }
}

function todayKey(): string {
  const now = new Date()
  const month = String(now.getMonth() + 1).padStart(2, '0')
  const day = String(now.getDate()).padStart(2, '0')
  return `${now.getFullYear()}-${month}-${day}`
}

function readState(): Record<string, string> {
  try {
    const raw = localStorage.getItem(STATE_KEY)
    if (!raw) return {}
    const parsed = JSON.parse(raw) as Record<string, string>
    return parsed && typeof parsed === 'object' ? parsed : {}
  } catch {
    return {}
  }
}

function writeState(state: Record<string, string>): void {
  try {
    localStorage.setItem(STATE_KEY, JSON.stringify(state))
  } catch {
    /* 存储不可用时忽略 */
  }
}

/**
 * 执行启动前备份，返回备份的存档数量（未开启或今天已备份时返回 0）
 */
export async function runAutoBackup(versionId: string): Promise<number> {
  if (!versionId) return 0
  if (!loadAutoBackupSetting().enabled) return 0

  const state = readState()
  const today = todayKey()
  if (state[versionId] === today) return 0

  const gameDir = await getGameDir()
  const list = await archiveList(versionId)
  const dir = `${gameDir}/molaunch-backups/${versionId}`
  let count = 0
  for (const world of list.items.slice(0, MAX_WORLDS)) {
    await archiveBackup(world.name, `${dir}/${world.name}.zip`, false, versionId)
    count += 1
  }

  state[versionId] = today
  writeState(state)
  return count
}
