/** 版本个性化信息 */
export interface VersionPersonalization {
  logo: string
  customInfo: string
  displayType: number
  isStar: boolean
  indieType: number
  versionType: string
  originalVersion: string
  windowTitle: string
  serverEnter: string
  advanceJvmArgs: string
  advanceGameArgs: string
  advanceRunCmd: string
  javaPath: string
  /** Java 选择模式：空/auto=自动选择, "auto_version"=自动选择指定版本范围, "folder"=使用版本文件夹中的 Java, "custom"=使用指定的 Java */
  javaMode: string
  /** 自动选择时的最小 Java 主版本（仅 auto_version 模式生效，0=不限） */
  javaVersionMin: number
  /** 自动选择时的最大 Java 主版本（仅 auto_version 模式生效，0=不限） */
  javaVersionMax: number
  /** 内存模式（空=跟随全局, "auto"=自动, "custom"=自定义） */
  memoryMode: string
  /** 版本独立最小内存（MB，仅 custom 模式生效，0 表示未设置） */
  minMemory: number
  /** 版本独立最大内存（MB，仅 custom 模式生效，0 表示未设置） */
  maxMemory: number
  // ===== 高级选项开关 =====
  advanceDisableModUpdate: boolean
  advanceIgnoreJavaWarning: boolean
  advanceDisableAssetsVerify: boolean
  advanceDisableJlw: boolean
  advanceDisableLua: boolean
}

/** 版本个性化字段更新（undefined 的字段不会被修改） */
export interface PersonalizationUpdate {
  logo?: string
  customInfo?: string
  displayType?: number
  isStar?: boolean
  indieType?: number
  windowTitle?: string
  serverEnter?: string
  advanceJvmArgs?: string
  advanceGameArgs?: string
  advanceRunCmd?: string
  javaPath?: string
  /** Java 选择模式：空/auto=自动选择, "auto_version"=自动选择指定版本范围, "folder"=使用版本文件夹中的 Java, "custom"=使用指定的 Java */
  javaMode?: string
  /** 自动选择时的最小 Java 主版本（仅 auto_version 模式生效，0=不限） */
  javaVersionMin?: number
  /** 自动选择时的最大 Java 主版本（仅 auto_version 模式生效，0=不限） */
  javaVersionMax?: number
  /** 内存模式：传空字符串=跟随全局, "auto"=自动, "custom"=自定义 */
  memoryMode?: string
  /** 版本独立最小内存（MB） */
  minMemory?: number
  /** 版本独立最大内存（MB） */
  maxMemory?: number
  // ===== 高级选项开关 =====
  advanceDisableModUpdate?: boolean
  advanceIgnoreJavaWarning?: boolean
  advanceDisableAssetsVerify?: boolean
  advanceDisableJlw?: boolean
  advanceDisableLua?: boolean
}