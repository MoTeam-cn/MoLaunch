/** 账号卡片数据（AccountCard.vue 与 useAccountCards 共享） */
export interface AccountCardData {
  uuid: string
  username: string
  loginType: string  // '正版' | '离线'
  isExpired?: boolean
  isActive?: boolean
}
