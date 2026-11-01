/**
 * 崩溃后自动上传日志并给出分享链接
 *
 * 只在用户显式开启「崩溃后自动上传」时生效，默认关闭：上传会把日志送到第三方，
 * 其中的本机路径、Mod 列表等信息不经用户确认不该外发。开启后崩溃弹窗一出现
 * 就会拿到一条可复制的链接，省去手动点分享再复制。
 */
import { ref, type Ref } from 'vue'
import { getConfigMap } from '@/utils/api/config'
import { toastError } from '@/utils/toast'
import {
  joinShareFiles,
  sanitizeShareFiles,
  uploadLogShare,
  type LogShareFile,
  type LogShareProvider,
} from '@/utils/logShare'

export interface CrashAutoShare {
  /** 自动上传得到的分享链接，没有则为空 */
  url: Ref<string>
  /** 上传中（用于界面显示进度） */
  busy: Ref<boolean>
  /** 按当前配置尝试自动上传；未开启或失败时静默 */
  run: (build: () => LogShareFile[]) => Promise<void>
}

export function useCrashAutoShare(): CrashAutoShare {
  const url = ref('')
  const busy = ref(false)

  async function run(build: () => LogShareFile[]): Promise<void> {
    url.value = ''
    let enabled = false
    let provider: LogShareProvider = 'mclogs'
    try {
      const cfg = await getConfigMap()
      enabled = Boolean(cfg.logShareAutoUpload)
      if (cfg.logShareProvider === 'logshare') provider = 'logshare'
    } catch {
      // 读配置失败就不自动上传，不影响崩溃弹窗
      return
    }
    if (!enabled) return
    const files = sanitizeShareFiles(build())
    if (files.length === 0) return
    busy.value = true
    try {
      url.value = await uploadLogShare(joinShareFiles(files), provider, files)
    } catch (e) {
      // 自动上传失败只在提示里说一声，不打断用户看崩溃原因
      toastError('自动上传日志失败：' + String(e))
    } finally {
      busy.value = false
    }
  }

  return { url, busy, run }
}
