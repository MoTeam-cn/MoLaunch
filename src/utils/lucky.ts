/**
 * 今日人品幸运度算法
 *
 * 基于「设备 ID + 日期」构造确定性幸运值（0-100 整数）：
 * - 同一设备同一天结果恒定，跨天自动重置
 * - 纯前端哈希 + LCG 混合计算，不依赖网络与持久化存储
 */

export interface LuckyRankInfo {
  level: string
  comment: string
}

export interface TodayLuck {
  value: number
  level: string
  comment: string
}

/** 获取幸运值（0-100 整数），deviceId 用于保证同一设备唯一性 */
export function getLuckyValue(deviceId: string, date: Date = new Date()): number {
  if (!deviceId || typeof deviceId !== 'string') {
    throw new Error('设备ID不能为空，且必须为字符串')
  }

  const dateSeed = date.getFullYear() * 10000 + (date.getMonth() + 1) * 100 + date.getDate()

  let deviceHash = 0
  for (let i = 0; i < deviceId.length; i++) {
    const char = deviceId.charCodeAt(i)
    deviceHash = ((deviceHash << 5) - deviceHash) + char
    deviceHash = deviceHash & deviceHash
  }
  deviceHash = Math.abs(deviceHash)

  let seed = (dateSeed * 1000003 + deviceHash * 1000033) & 0x7fffffff
  seed = (seed * 1103515245 + 12345) & 0x7fffffff
  seed = (seed * 1103515245 + 12345) & 0x7fffffff
  seed = seed ^ (seed >> 13) ^ (seed << 7)
  seed = (seed * 9301 + 49297) & 0x7fffffff
  seed = (seed * 9301 + 49297) & 0x7fffffff

  return seed % 101
}

/** 根据幸运值划分五档趣味等级与评语 */
export function getLuckyRank(value: number): LuckyRankInfo {
  if (value >= 80) {
    return { level: '欧皇', comment: '今天你就是天选之子！' }
  }
  if (value >= 60) {
    return { level: '小欧', comment: '运气不错，可以试试手气' }
  }
  if (value >= 40) {
    return { level: '普通人', comment: '平平无奇的一天' }
  }
  if (value >= 20) {
    return { level: '非酋', comment: '今天不适合抽卡' }
  }
  return { level: '大非酋', comment: '建议换个日子' }
}

/** 获取今日人品完整信息 */
export function getTodayLuck(deviceId: string, date: Date = new Date()): TodayLuck {
  const value = getLuckyValue(deviceId, date)
  const rank = getLuckyRank(value)
  return { value, level: rank.level, comment: rank.comment }
}