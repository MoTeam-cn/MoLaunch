/**
 * 使用协议 / 免责声明抽屉展示控制（按自然日，存 localStorage）
 *
 * 进入联机 / 实验性功能 / 工具 / 开发者选项页时，若当天未同意过则弹出「使用协议与免责声明」抽屉；
 * 用户点击「我已知悉并同意」后记录当天日期，当天再次进入不再弹出（次日重新提醒）。
 *
 * localStorage 不可用（如隐私模式 / 配额满）时静默降级：仅当天可重复弹出，不影响功能。
 */

export type DisclaimerKind = 'online' | 'experimental' | 'tools' | 'developer'

/** 存储键：{ [kind]: 'YYYY-MM-DD' } */
const STORAGE_KEY = 'molaunch.disclaimerAgreed'

/** 本地日期（YYYY-MM-DD） */
function todayStr(): string {
  const d = new Date()
  const mm = String(d.getMonth() + 1).padStart(2, '0')
  const dd = String(d.getDate()).padStart(2, '0')
  return `${d.getFullYear()}-${mm}-${dd}`
}

/** 读取已同意记录（异常时返回空对象） */
function readMap(): Record<string, string> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    return raw ? (JSON.parse(raw) as Record<string, string>) : {}
  } catch {
    return {}
  }
}

/** 今日是否已同意指定类型的协议 */
export function hasAgreedToday(kind: DisclaimerKind): boolean {
  return readMap()[kind] === todayStr()
}

/** 标记指定类型今天已同意 */
export function markAgreedToday(kind: DisclaimerKind): void {
  try {
    const map = readMap()
    map[kind] = todayStr()
    localStorage.setItem(STORAGE_KEY, JSON.stringify(map))
  } catch {
    // 写入失败忽略：下次进入继续提醒
  }
}