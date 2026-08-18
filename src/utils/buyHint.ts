/**
 * 正版购买提示服务
 *
 * - 游戏启动成功时计数自增并持久化（系统存储，非 AppConfig）
 * - 命中阈值且非微软账号、中文系统时弹出「正版购买建议」
 * - 抽屉内「前往购买」打开官网并永久忽略；「暂不考虑」仅关闭，下次阈值再次提醒
 */

import { ref } from 'vue'
import { applyConfig, getConfigMap } from '@/utils/api/config'
import { useAuthStore } from '@/stores/auth'
import { maybeTriggerStarHint } from '@/utils/starHint'

/** 触发购买提示的启动次数阈值 */
const BUY_HINT_THRESHOLDS = [
  3, 8, 15, 30, 50, 70, 90, 110, 130, 180, 220, 280, 330, 380, 450, 550, 660,
  750, 880, 950, 1100, 1300, 1500, 1700, 1900,
]

/** 统一 HintDialog 组件实例对外暴露的正版购买页接口 */
export interface BuyHintDialogInstance {
  showBuy: (count?: number) => void
}

const buyHintDialogRef = ref<BuyHintDialogInstance | null>(null)

export function setBuyHintDialogRef(ref: BuyHintDialogInstance | null) {
  buyHintDialogRef.value = ref
}

/** 判断启动次数是否命中提示阈值 */
function hitThreshold(count: number): boolean {
  return BUY_HINT_THRESHOLDS.includes(count)
}

/**
 * 启动成功后统一入口：计数自增一次并持久化，随后分别检查正版购买提示与「点 Star」提示。
 * 已永久忽略 / 微软账号 / 非中文系统时不弹窗；非阻塞调用，失败仅计数不影响启动。
 */
export async function maybeTriggerLaunchHints(): Promise<void> {
  try {
    const cfg = await getConfigMap()
    const count = (cfg.launchCount ?? 0) + 1
    await applyConfig({ launchCount: count })
    // 正版购买提示（非微软账号、中文系统才提示）
    if (!cfg.hintBuy && hitThreshold(count)) {
      const authStore = useAuthStore()
      if (!authStore.isMicrosoftLogin && /^zh/i.test(navigator.language)) {
        showBuyHintDialog(count)
      }
    }
    // 去 GitHub 点 Star 提示（独立阈值 + 独立忽略标记，内部再做条件判断）
    await maybeTriggerStarHint(cfg, count)
  } catch (e) {
    console.error('[BuyHint] 计数 / 触发失败：', e)
  }
}

/**
 * 弹出「正版购买建议」抽屉（dev-API 直测用，绕过外部条件）
 *
 * @param count 当前启动次数（可选，用于文案展示）
 */
export function showBuyHintDialog(count?: number): void {
  buyHintDialogRef.value?.showBuy(count)
}
