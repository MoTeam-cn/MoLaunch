/**
 * 地区判定结果（后端 utils::region 的 RegionInfo 序列化，字段 camelCase）
 *
 * 受限地区列表、提示文案与「开发者模式绕过」开关均由官网下发的
 * /download-policy.json 决定（拉取失败按不限制处理，fail-open）；
 * 机器环境 + 网络出口判定沿用启动器既有实现。
 */

/** 官网下发的提示文案（与官网弹窗同一份） */
export interface PolicyNotice {
  /** 提示标题 */
  title: string
  /** 提示正文 */
  text: string
  /** 确认按钮文案 */
  confirmText: string
}

/** 地区判定结果 */
export interface RegionInfo {
  /** 机器环境是否位于中国大陆（时区 + 系统区域） */
  isChinaMainlandMachine: boolean
  /** 网络出口是否位于中国大陆 */
  isChinaMainlandNetwork: boolean
  /** 网络出口国家代码（未知为 null） */
  countryCode: string | null
  /** 最终是否禁用下载（已扣除开发者模式临时绕过） */
  isRestricted: boolean
  /** 当前策略的受限地区列表（ISO 3166-1 alpha-2，归一化后） */
  blockedRegions: string[]
  /** 官网下发的提示文案 */
  notice: PolicyNotice
  /** 策略是否允许开发者模式绕过限制 */
  allowDeveloperModeBypass: boolean
  /** 启动器当前是否处于开发者模式 */
  developerMode: boolean
  /** 是否已通过开发者模式临时关闭限制（仅本次运行有效） */
  bypassed: boolean
}
