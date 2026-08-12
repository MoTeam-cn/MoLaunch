import { reactive } from 'vue'

export type UpdateStatus =
  | 'idle'
  | 'checking'
  | 'available'
  | 'no-update'
  | 'downloading'
  | 'installing'
  | 'done'
  | 'error'

/** 后端返回的更新信息（与 Rust `UpdateInfo` 结构对应） */
export interface UpdateInfo {
  available: boolean
  version: string
  notes: string
  forceUpdate: boolean
  downloadUrl: string
  signature: string
}

export interface UpdateState {
  /** 当前状态 */
  status: UpdateStatus
  /** 新版本号 */
  version: string
  /** 更新日志 */
  notes: string
  /** 是否强制更新（来自 manifest 扩展字段） */
  forceUpdate: boolean
  /** 已下载字节数（当前版本不报告精确进度，保持 0） */
  downloaded: number
  /** 总字节数（当前版本不报告精确进度，保持 0） */
  total: number
  /** 错误信息 */
  error: string
  /** 是否显示弹窗（手动触发或发现更新时为 true） */
  showDialog: boolean
  /** Windows 后台静默预下载更新包中（供 UI 禁用「立即更新」并提示） */
  silentDownloading: boolean
}

/** 全局更新状态（响应式，组件可直接 watch） */
export const updateState = reactive<UpdateState>({
  status: 'idle',
  version: '',
  notes: '',
  forceUpdate: false,
  downloaded: 0,
  total: 0,
  error: '',
  showDialog: false,
  silentDownloading: false,
})

/** 内部可变单例状态（跨切片共享，经对象属性读写避免跨模块重新绑定） */
export const updaterFlags = {
  /** 当前待安装的更新信息（checkForUpdate 后缓存，downloadAndInstall 使用） */
  pendingUpdate: null as UpdateInfo | null,
  /** 防止并发检查（仅手动 checkForUpdate 使用） */
  checking: false,
  /** 防止并发静默检查+后台下载（仅 silentCheckAndDownload 使用，避免占用 checking 阻塞手动检查） */
  silentChecking: false,
  /** 防止并发下载 */
  installing: false,
  /** Windows 后台静默下载已完成的版本号（避免 10 分钟定时重复下载同一版本） */
  appdataDownloadedVersion: null as string | null,
}