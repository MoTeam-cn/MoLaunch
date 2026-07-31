/**
 * 联机功能类型定义 - 房主白名单域
 *
 * 阶段三子任务 8 安全加强。
 *
 * 对应后端 `minecraft::online::signaling::WhitelistEntry` / `WhitelistResponse`。
 * 房主可启用白名单后指定允许加入的设备（按 `device_id` 友好标识），
 * 启用且白名单为空 = 拒绝所有人加入（仅房主可进入）。
 */

/** 白名单条目（房主查询/管理用） */
export interface WhitelistEntry {
  /** 设备主键（UUID） */
  devicePk: string
  /** 设备友好标识（如 `mcsdk-xxxx-xxxx-xxxx-xxxx`） */
  deviceId: string
  /** 加入白名单时间（Unix 秒） */
  addedAt: number
}

/** 白名单列表响应 */
export interface WhitelistResponse {
  /** 是否启用白名单 */
  enabled: boolean
  /** 白名单条目数组 */
  entries: WhitelistEntry[]
}
