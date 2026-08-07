/**
 * "去 GitHub 点 Star"提示服务（参照 PCL2 赞助弹窗，但目标为项目仓库而非爱发电）
 *
 * - 复用游戏启动成功计数（与购买提示共用 launchCount，不重复自增）
 * - 命中阈值（对齐 PCL2 赞助弹窗：10/20/40/…/2000）且中文系统时弹出「支持 MoLaunch」
 * - 抽屉内「去点 Star」打开 GitHub 仓库并永久忽略；「暂不考虑」仅关闭，下次阈值再次提醒
 * - 文案 / 阈值 / 目标地址支持远程下发：`fetchRemoteStarHintConfig` 当前返回 null（未启用），
 *   后续 apiServer 就绪后接入即可覆盖本地默认，无需改动触发链路
 */

import { ref } from 'vue'
import type { ConfigSnapshot } from '@/utils/api/config'

/** 触发点 Star 提示的启动次数阈值（对齐 PCL2 ModLaunch.vb 赞助弹窗） */
export const STAR_HINT_THRESHOLDS = [
  10, 20, 40, 60, 80, 100, 120, 150, 200, 250, 300, 350, 400, 500, 600, 700,
  800, 900, 1000, 1200, 1400, 1600, 1800, 2000,
]

/** 可远程下发的 Star 提示配置（本地默认 + 后续 apiServer 覆盖） */
export interface StarHintRemoteConfig {
  /** 总开关：false 时即便命中阈值也不弹 */
  enabled: boolean
  /** 触发阈值列表 */
  thresholds: number[]
  /** 点 Star 目标仓库地址 */
  githubUrl: string
  /** 抽屉标题 */
  title: string
  /** 正文第一段（已含启动次数占位） */
  message: string
  /** 正文第二段（恳请支持的语气） */
  subMessage: string
  /** 主按钮文案（去点 Star） */
  confirmText: string
  /** 次按钮文案（仅关闭） */
  cancelText: string
}

/** 本地默认配置（apiServer 未下发时使用） */
const DEFAULT_STAR_HINT_CONFIG: StarHintRemoteConfig = {
  enabled: true,
  thresholds: STAR_HINT_THRESHOLDS,
  githubUrl: 'https://github.com/MoTeam-cn/MoLaunch',
  title: '支持 MoLaunch',
  message: '你已通过 MoLaunch 启动游戏 {count} 次。',
  subMessage:
    '如果 MoLaunch 对你有帮助，欢迎在 GitHub 为项目点一份 Star。你的支持是项目持续开发与维护的见证。',
  confirmText: '去点 Star',
  cancelText: '暂不考虑',
}

/**
 * 拉取 apiServer 下发的 Star 提示配置（预留，尚未接入）
 *
 * 当前返回 null 表示使用本地默认配置；后续 apiServer 就绪后在此实现拉取并返回
 * 完整配置对象（或 Partial 合并进默认值），触发链路无需改动。
 */
export async function fetchRemoteStarHintConfig(): Promise<StarHintRemoteConfig | null> {
  return null
}

/** 合并远程配置：返回最终生效配置（未下发时回退本地默认） */
export async function resolveStarHintConfig(): Promise<StarHintRemoteConfig> {
  const remote = await fetchRemoteStarHintConfig()
  if (!remote) return DEFAULT_STAR_HINT_CONFIG
  return { ...DEFAULT_STAR_HINT_CONFIG, ...remote }
}

/** 判断启动次数是否命中点 Star 阈值 */
export function hitStarThreshold(count: number, thresholds: number[]): boolean {
  return thresholds.includes(count)
}

/** 统一 HintDialog 组件实例对外暴露的点 Star 页接口 */
export interface StarHintDialogInstance {
  showStar: (count?: number) => void
}

const starHintDialogRef = ref<StarHintDialogInstance | null>(null)

export function setStarHintDialogRef(ref: StarHintDialogInstance | null) {
  starHintDialogRef.value = ref
}

/**
 * 弹出「支持 MoLaunch」抽屉（dev-API 直测用，绕过外部条件）
 *
 * @param count 当前启动次数（可选，用于文案展示）
 */
export function showStarHintDialog(count?: number): void {
  starHintDialogRef.value?.showStar(count)
}

/**
 * 启动成功后检查并触发点 Star 提示（计数已在统一入口自增）
 *
 * 已永久忽略 / 非中文系统 / 未命中阈值时不弹。非阻塞调用，失败静默忽略。
 */
export async function maybeTriggerStarHint(cfg: ConfigSnapshot, count: number): Promise<void> {
  try {
    const config = await resolveStarHintConfig()
    if (!config.enabled) return
    if (cfg.hintStar) return
    if (!hitStarThreshold(count, config.thresholds)) return
    if (!/^zh/i.test(navigator.language)) return
    showStarHintDialog(count)
  } catch (e) {
    console.error('[StarHint] 触发失败：', e)
  }
}
