/**
 * 管理员提权重启的页面恢复
 *
 * 提权重启前保存当前路由，新实例启动后消费并跳回原页面（不再落回主页）。
 */

const RELAUNCH_RESTORE_KEY = 'molaunch-relaunch-restore'

/** 保存提权重启前所在页面（pathname + search） */
export function saveRelaunchRestore(path: string): void {
  localStorage.setItem(RELAUNCH_RESTORE_KEY, path)
}

/** 读取并清除恢复路径（一次性消费，调用方决定是否使用） */
export function consumeRelaunchRestore(): string | null {
  const saved = localStorage.getItem(RELAUNCH_RESTORE_KEY)
  if (saved) localStorage.removeItem(RELAUNCH_RESTORE_KEY)
  return saved
}

/** 清除恢复路径（UAC 被用户拒绝时调用，避免下次正常启动误恢复） */
export function clearRelaunchRestore(): void {
  localStorage.removeItem(RELAUNCH_RESTORE_KEY)
}
