/**
 * 《用户协议》全局门禁（首次启动同意后才能使用）
 *
 * - 系统存储记录 userAgreed / userAgreedVersion（Windows 注册表 / 其他系统全局共用文件）
 * - 启动时若未同意，或已同意版本低于当前协议版本，则弹出强制弹窗，同意后方可继续使用
 * - 文案 / 版本 / 外链支持远程下发：`fetchRemoteUserAgreementConfig` 拉官网 /agreement.json
 *   （经 Rust 侧 user_agreement_fetch 绕开 WebView CORS），失败回退本地默认
 */

import { ref } from 'vue'
import { applyConfig, getConfigMap } from '@/utils/api/config'
import { onlineManager } from '@/utils/api/online-manager'

/** 本地默认《用户协议》版本号（协议内容每次有实质更新时自增，触发用户重新同意） */
export const USER_AGREEMENT_VERSION = 2

/** 可远程下发的《用户协议》配置（本地默认 + 后续 apiServer 覆盖） */
export interface UserAgreementRemoteConfig {
  /** 总开关：false 时跳过门禁，直接进入 */
  enabled: boolean
  /** 协议版本号（大于本地已同意版本时重新要求同意） */
  version: number
  /** 弹窗标题 */
  title: string
  /** 引言段（简短总述） */
  intro: string
  /** 协议要点列表（自设计的简短内容） */
  sections: string[]
  /** 服务条款外链 */
  termsUrl: string
  /** 隐私声明外链 */
  privacyUrl: string
  /** 服务条款链接文案 */
  termsText: string
  /** 隐私声明链接文案 */
  privacyText: string
  /** 底部提示语 */
  notice: string
  /** 同意按钮文案 */
  confirmText: string
}

/** 本地默认《用户协议》配置（apiServer 未下发时使用，简短自设计内容） */
const DEFAULT_USER_AGREEMENT_CONFIG: UserAgreementRemoteConfig = {
  enabled: true,
  version: USER_AGREEMENT_VERSION,
  title: 'MoLaunch 用户协议',
  intro:
    '本协议是您与 MoTeam 之间就使用 MoLaunch 启动器（下称"本软件"）所订立的条款。首次使用前请您仔细阅读并同意，方可继续使用。',
  sections: [
    '账号与使用：请您遵循当地法律法规，不得利用本软件从事任何违法、侵权或危害网络安全的行为；使用第三方账号登录时，请同时遵守对应平台的规则。',
    '软件与内容：本软件及授权内容的许可以项目仓库最新 LICENSE 为准，第三方内容受其各自许可约束，可在「设置 → 更多 → 许可协议」查看版本说明。',
    '隐私与数据：本软件的云端、联机、AI 等功能可能涉及数据外发，具体处理方式见《隐私声明》与对应功能说明；请妥善管理并控制您主动上传的内容。',
    '服务与变更：本协议可能随版本更新调整，更新后以您再次明确同意为准；完整条款与最新版本以官网《服务条款》《隐私政策》为准。',
  ],
  termsUrl: 'https://molaunch.moiu.cn/terms',
  privacyUrl: 'https://molaunch.moiu.cn/privacy',
  termsText: '《服务条款》',
  privacyText: '《隐私政策》',
  notice: '您需同意《用户协议》后方可继续使用 MoLaunch；完整条款请查看上方链接。',
  confirmText: '同意并继续',
}

/** 后端 user_agreement_fetch 响应 */
interface UserAgreementFetchResponse {
  success: boolean
  config?: Partial<UserAgreementRemoteConfig> | null
  error: string
}

/**
 * 拉取官网下发的《用户协议》配置
 *
 * 由 Rust 侧发起（online_manager 的 user_agreement_fetch），绕开 WebView CORS。
 * 返回 null 表示使用本地默认配置：网络失败、响应非成功、或字段非法都回退，
 * 不阻塞启动。字段与官网 src/logic/agreement.ts 的 AgreementManifest 一一对应。
 */
export async function fetchRemoteUserAgreementConfig(): Promise<Partial<UserAgreementRemoteConfig> | null> {
  try {
    const resp = await onlineManager<UserAgreementFetchResponse>('user_agreement_fetch')
    const cfg = resp?.config
    if (!resp?.success || !cfg) return null
    // 只接受能安全驱动门禁的清单：版本号为正数、要点列表非空。
    // 否则宁可回退本地默认，也不要用脏数据把门禁卡死。
    if (typeof cfg.version !== 'number' || !Number.isFinite(cfg.version) || cfg.version <= 0) return null
    if (!Array.isArray(cfg.sections) || cfg.sections.length === 0) return null
    return cfg
  } catch {
    return null
  }
}

/** 合并远程配置：返回最终生效配置（未下发时回退本地默认） */
export async function resolveUserAgreementConfig(): Promise<UserAgreementRemoteConfig> {
  const remote = await fetchRemoteUserAgreementConfig()
  if (!remote) return DEFAULT_USER_AGREEMENT_CONFIG
  return { ...DEFAULT_USER_AGREEMENT_CONFIG, ...remote }
}

/** 统一 UserAgreementDialog 组件实例对外暴露的接口 */
export interface UserAgreementDialogInstance {
  showAgreement: (version: number) => void
}

const userAgreementDialogRef = ref<UserAgreementDialogInstance | null>(null)

export function setUserAgreementDialogRef(ref: UserAgreementDialogInstance | null) {
  userAgreementDialogRef.value = ref
}

/**
 * 弹出《用户协议》弹窗（dev-API 直测用）
 *
 * @param version 当前协议版本号，同意后以此持久化
 */
export function showUserAgreementDialog(version: number): void {
  userAgreementDialogRef.value?.showAgreement(version)
}

/**
 * 持久化用户已同意指定版本（由弹窗「同意并继续」调用）
 */
export async function acceptUserAgreement(version: number): Promise<void> {
  await applyConfig({ userAgreed: true, userAgreedVersion: version })
}

/**
 * 重置门禁（dev-API 直测用：清空同意记录，下次启动重新要求同意）
 */
export async function resetUserAgreement(): Promise<void> {
  await applyConfig({ userAgreed: false, userAgreedVersion: 0 })
}

/**
 * 启动入门禁检查：未同意 / 已同意版本过低时弹《用户协议》弹窗。
 *
 * 非阻塞调用，失败静默忽略（不因协议加载失败阻塞启动）。
 */
export async function maybeRequireUserAgreement(): Promise<void> {
  try {
    const config = await resolveUserAgreementConfig()
    if (!config.enabled) return
    const cfg = await getConfigMap()
    if (cfg.userAgreed && (cfg.userAgreedVersion ?? 0) >= config.version) return
    showUserAgreementDialog(config.version)
  } catch (e) {
    console.error('[UserAgreement] 门禁检查失败：', e)
  }
}