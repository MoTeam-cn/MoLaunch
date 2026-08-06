/**
 * 实验性功能开关组合式函数
 *
 * 共享逻辑：读取/监听 `experimentalEnabled` 配置（开关位于「设置 → 进阶设置」）。
 * - 顶部导航据此显示/隐藏「实验性」入口
 * - 实验性页面据此做访问守卫
 * - 设置页开关切换时派发 `experimental-mode-changed` 事件，实时通知各监听方
 */
import { ref, onMounted, onUnmounted } from 'vue'
import { getConfigMap } from '@/utils/api/config'
import { safeCall } from '@/utils/async'

/** 全局事件名（开关变化通知） */
export const EXPERIMENTAL_CHANGED_EVENT = 'experimental-mode-changed'

export function useExperimental() {
  const enabled = ref(false)

  /** 从配置读取当前开关状态 */
  async function load() {
    const cfg = await safeCall(() => getConfigMap(), 'check experimental flag')
    enabled.value = cfg?.experimentalEnabled ?? false
  }

  /** 本地即时切换状态（不负责持久化，持久化由设置页 applyConfig 完成） */
  function setEnabled(value: boolean) {
    enabled.value = value
    window.dispatchEvent(new CustomEvent(EXPERIMENTAL_CHANGED_EVENT, { detail: value }))
  }

  onMounted(() => {
    load()
    window.addEventListener(EXPERIMENTAL_CHANGED_EVENT, onChange)
  })
  onUnmounted(() => {
    window.removeEventListener(EXPERIMENTAL_CHANGED_EVENT, onChange)
  })

  function onChange(e: Event) {
    enabled.value = (e as CustomEvent).detail === true
  }

  return { enabled, load, setEnabled }
}
