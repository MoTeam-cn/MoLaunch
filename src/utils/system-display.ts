/**
 * 系统信息显示辅助函数（从 SettingsDeveloper.vue 抽出）
 *
 * 纯展示函数：将后端返回的原始 os/arch 字符串映射为用户可读的本地化显示名。
 */

/** 操作系统显示名 */
export function osDisplay(os: string): string {
  switch (os) {
    case 'windows': return 'Windows'
    case 'macos': return 'macOS'
    case 'linux': return 'Linux'
    default: return os
  }
}

/** 架构显示名 */
export function archDisplay(arch: string): string {
  switch (arch) {
    case 'x86_64': return 'x64 (64-bit)'
    case 'aarch64': return 'ARM64'
    case 'x86': return 'x86 (32-bit)'
    default: return arch
  }
}
