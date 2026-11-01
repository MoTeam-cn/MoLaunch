/**
 * 崩溃日志的分享文件组装
 *
 * 崩溃现场通常散在几处：崩溃摘要、实例目录里的 latest.log / debug.log / 崩溃报告。
 * 优先读磁盘上的真实文件（文件名也是玩家熟悉的那几个），实例目录不可达时
 * 退回崩溃瞬间抓到的片段。拆分后上传，分享页能按文件分别展开。
 */
import type { CrashCategory, CrashInfo } from '@/types/version'
import { listInstanceLogs, readInstanceLog } from '@/utils/api/version'
import type { LogShareFile } from '@/utils/logShare'

/** 从崩溃报告路径反推实例目录（去掉尾部 crash-reports/<文件> 两级） */
export function instanceDirOf(path?: string): string {
  if (!path) return ''
  const m = path.match(/^(.*)[\\/]crash-reports[\\/][^\\/]+$/)
  return m ? m[1] : ''
}

/** 读实例目录里的真实日志文件；目录不可达时返回空数组 */
async function readRealLogs(dir: string): Promise<LogShareFile[]> {
  if (!dir) return []
  try {
    const list = await listInstanceLogs(dir)
    const picked = list
      .filter((f) => ['latest.log', 'debug.log'].includes(f.name) || /^crash-.*\.txt$/.test(f.name))
      .sort((a, b) => (a.name === 'latest.log' ? -1 : b.name === 'latest.log' ? 1 : 0))
      .slice(0, 3)
    const files: LogShareFile[] = []
    for (const f of picked) {
      const content = await readInstanceLog(dir, f.name)
      if (content.trim()) files.push({ name: f.name, content })
    }
    return files
  } catch {
    return []
  }
}

/** 组装崩溃分享用的文件列表 */
export async function buildCrashShareFiles(
  info: CrashInfo,
  category: CrashCategory,
): Promise<LogShareFile[]> {
  const files: LogShareFile[] = [
    {
      name: 'crash-summary.txt',
      content: [
        '==== MoLaunch 崩溃摘要 ====',
        '',
        `类别：${category}`,
        `原因：${info.reason ?? ''}`,
        info.problematic_mod ? `相关 Mod：${info.problematic_mod}` : '',
      ]
        .filter(Boolean)
        .join('\n'),
    },
  ]
  const real = await readRealLogs(instanceDirOf(info.crash_report_path))
  if (real.length > 0) return files.concat(real)
  const logLines = (info.log_lines ?? []).join('\n')
  if (logLines.trim()) files.push({ name: 'crash-log.txt', content: logLines })
  const logTail = (info.log_tail ?? []).join('\n')
  if (logTail.trim()) files.push({ name: 'game-tail.txt', content: logTail })
  return files
}
