/**
 * 日志显示辅助函数（从 SettingsDeveloper.vue 抽出）
 *
 * 纯展示函数：解析日志文本为带行号与级别的行数组，并按业界惯例返回级别对应的文字颜色 class。
 *
 * 后端日志格式：[HH:MM:SS.ms] [LEVEL] message
 * 红石内核日志格式：2026/08/09 15:32:39 [LEVEL] message（单括号时间戳）
 * 会话分隔标记格式：=== MoLaunch Started ===（后端每次启动时打印，开头会带一个 \n）
 */

export interface LogLine {
  /** 行号（从 1 开始，保留原始文件行号；开头的空行被跳过） */
  no: number
  /** 原始文本 */
  text: string
  /** 级别（error/warn/info/debug/trace/session/other） */
  level: 'error' | 'warn' | 'info' | 'debug' | 'trace' | 'session' | 'other'
}

/** 解析日志内容为带行号和级别的行数组
 *
 * - 跳过开头连续空行（后端每次会话开始时打印 "\n=== MoLaunch Started ==="，导致日志开头出现空行）
 * - 中间空行保留（作为日志段落分隔，便于阅读）
 * - 行号保留原始文件行号（被跳过的空行不计入显示但占据文件行号）
 */
export function parseLogLines(content: string): LogLine[] {
  const allLines = content.split('\n')
  // 跳过开头连续的空行
  let startIdx = 0
  while (startIdx < allLines.length && allLines[startIdx].trim() === '') {
    startIdx++
  }
  const lines = allLines.slice(startIdx)
  return lines.map((text, i) => {
    const no = startIdx + i + 1
    // 会话分隔标记（如 === MoLaunch Started === / === MoLaunch Ended ===）
    if (/^===\s+.+\s+===\s*$/.test(text.trim())) {
      return { no, text, level: 'session' as const }
    }
    // 匹配第二个方括号内的级别（后端格式 [time] [LEVEL]）
    // 兼容红石内核等单括号时间戳格式：2026/08/09 15:32:39 [INFO] message
    const m =
      text.match(/^\[[^\]]*\]\s*\[(\w+)\]/) ??
      text.match(/^\d{4}\/\d{2}\/\d{2} \d{2}:\d{2}:\d{2} \[(\w+)\]/)
    let level: LogLine['level'] = 'other'
    if (m) {
      const lv = m[1].toUpperCase()
      if (lv === 'ERROR') level = 'error'
      else if (lv === 'WARN') level = 'warn'
      else if (lv === 'INFO') level = 'info'
      else if (lv === 'DEBUG') level = 'debug'
      else if (lv === 'TRACE') level = 'trace'
    }
    return { no, text, level }
  })
}

/**
 * 日志级别对应的文字颜色 class
 *
 * 参考业界日志查看器惯例（VS Code 终端 / Chrome DevTools / journalctl）：
 * - ERROR   → 红色（异常，需立即关注）
 * - WARN    → 黄色（警告，潜在问题）
 * - INFO    → 绿色（正常运行状态，业界惯例而非白色）
 * - DEBUG   → 青色（调试信息，冷色调区分）
 * - TRACE   → 暗灰（最详细的跟踪信息，弱化显示）
 * - SESSION → 靛蓝（会话分隔标记，如 === MoLaunch Started ===，醒目区分启动边界）
 * - 其他    → 浅灰（分隔线、原始输出等）
 */
export function logLineClass(level: LogLine['level']): string {
  switch (level) {
    case 'error': return 'text-red-400'
    case 'warn': return 'text-yellow-400'
    case 'info': return 'text-green-400'
    case 'debug': return 'text-cyan-400'
    case 'trace': return 'text-slate-500'
    case 'session': return 'text-indigo-400'
    default: return 'text-slate-300'
  }
}
