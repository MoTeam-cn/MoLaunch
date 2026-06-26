/** 应用设置 */
export interface AppSettings {
  game_dir: string
  max_download_threads: number
  mirror_url?: string
  log_level: number
  java_path?: string
  min_memory: number
  max_memory: number
  theme: Theme
  language: Language
}

/** 主题类型 */
export type Theme = 'light' | 'dark' | 'system'

/** 语言类型 */
export type Language = 'zh-CN' | 'en-US'
