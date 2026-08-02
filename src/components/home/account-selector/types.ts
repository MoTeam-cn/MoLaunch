/** 账号卡片数据（AccountCard.vue 与 useAccountCards 共享） */
export interface AccountCardData {
  uuid: string
  username: string
  loginType: string  // '正版' | '离线' | '外置'
  isExpired?: boolean
  /** 续期中：token 已过期且静默续期失败，显示「续期中」而非「已过期」 */
  refreshing?: boolean
  isActive?: boolean
  /** authlib 外置登录的服务器显示名（仅外置账号有值，用作副标题） */
  serverName?: string
  /** authlib 账号定位用：切换/删除时需要 server_url + uuid 双键 */
  serverUrl?: string
}
