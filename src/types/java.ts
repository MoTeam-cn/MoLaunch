/**
 * Java 运行时类型定义
 */

/** Java 运行时信息（与后端 JavaRuntimeInfo 对齐） */
export interface JavaRuntime {
  /** java.exe 完整路径 */
  executable: string
  /** 所在文件夹 */
  path_folder: string
  /** 是否手动导入 */
  is_user_import: boolean
  /** 详细版本号 */
  version: string
  /** 大版本号 */
  major_version: number
  /** 是否为 JRE */
  is_jre: boolean
  /** 是否 64 位 */
  is_64bit: boolean
}

/** Java 版本需求信息 */
export interface JavaRequirements {
  /** MC 版本号 */
  mc_version: string
  /** 最低 Java 版本（0 表示无下限约束） */
  min_java_version: number
  /** 最高 Java 版本（0 表示无上限约束） */
  max_java_version: number
  /** 推荐 Java 版本 */
  recommended_java_version: number
  /** 加载器类型 */
  loader: string | null
}

/** Java 兼容性检查结果 */
export interface JavaCompatResult {
  /** 是否兼容 */
  compatible: boolean
  /** 当前 Java 大版本号（0 表示无法检测） */
  current_version: number
  /** 最低要求（0 表示无下限约束） */
  min_required: number
  /** 最高要求（0 表示无上限约束） */
  max_required: number
  /** 人类可读的警告信息（兼容时为空字符串） */
  warning: string
}

/** Java 下载进度事件 payload（事件名 `java-download-progress`） */
export interface JavaDownloadProgress {
  /** 阶段：fetching|matching|manifest|downloading|verifying|done */
  stage: string
  /** 已完成文件数 */
  current: number
  /** 总文件数 */
  total: number
  /** 已下载字节数 */
  bytes_downloaded: number
  /** 总字节数 */
  bytes_total: number
  /** 人类可读消息 */
  message: string
}
