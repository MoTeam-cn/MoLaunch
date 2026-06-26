/**
 * Java 运行时类型定义
 */

/** Java 运行时信息 */
export interface JavaRuntime {
  executable: string
  version: string
  major_version: number
  arch: string
  home: string
}
