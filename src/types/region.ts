/**
 * 地区判定结果（后端 utils::region 的 RegionInfo 序列化，字段 camelCase）
 *
 * 机器层：UTC+8 时区 + 系统区域为中国大陆（Windows 区域名/GeoID）；
 * 网络层：机器层命中后再按网络出口国家确认，判定失败按受限处理。
 */

/** 地区判定结果 */
export interface RegionInfo {
  /** 机器环境是否位于中国大陆（时区 + 系统区域） */
  isChinaMainlandMachine: boolean
  /** 网络出口是否位于中国大陆（仅机器层命中时判定） */
  isChinaMainlandNetwork: boolean
  /** 网络出口国家代码（未知为 null） */
  countryCode: string | null
  /** 是否命中下载限制（机器层与网络层同时命中） */
  isRestricted: boolean
}
