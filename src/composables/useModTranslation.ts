/**
 * 实验性 - 模组翻译组合式函数
 *
 * 单任务模型：分析 → 启动翻译 → 进度事件订阅 → 完成/失败/取消。
 * 进度事件 `mod-translation-event` 由后端 emit，经 useTauriEvent 订阅。
 */
import { ref, computed, onMounted, onUnmounted } from 'vue'
import {
  modTranslationAnalyze,
  modTranslationCancel,
  modTranslationStart,
  modTranslationStatus,
  type ModTranslationAnalyzeResult,
  type ModTranslationTaskSnapshot,
} from '@/utils/api/experimental-mod-translation'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { useTauriEvent } from '@/composables/useTauriEvent'
import { setDragSuppressed } from '@/composables/useDragDrop/state'
import { safeCall } from '@/utils/async'

/** 后端事件名（与 mod_translation::EVENT_NAME 一致） */
export const MOD_TRANSLATION_EVENT = 'mod-translation-event'

export function useModTranslation() {
  const analyzing = ref(false)
  const analyzeResult = ref<ModTranslationAnalyzeResult | null>(null)
  const snapshot = ref<ModTranslationTaskSnapshot | null>(null)
  const jarPath = ref('')
  /** 局部拖放框悬停状态（拖入 jar 时高亮） */
  const dragging = ref(false)
  let unlistenDrag: (() => void) | null = null

  const running = computed(() => snapshot.value?.status === 'running')
  const completed = computed(() => snapshot.value?.status === 'completed')

  /** 分析 JAR：解包并汇总语言源 */
  async function analyze(path: string): Promise<boolean> {
    analyzing.value = true
    const result = await safeCall(() => modTranslationAnalyze(path), 'analyze mod jar')
    analyzing.value = false
    if (result) {
      jarPath.value = path
      analyzeResult.value = result
      return true
    }
    return false
  }

  /** 启动翻译任务（后台执行，进度经事件推送） */
  async function start(
    model: string,
    batchSize: number,
    options?: { generateModName?: boolean; repairEnabled?: boolean; classTextEnabled?: boolean },
  ): Promise<boolean> {
    if (!jarPath.value) return false
    const result = await safeCall(
      () =>
        modTranslationStart({
          jarPath: jarPath.value,
          model,
          batchSize,
          generateModName: options?.generateModName,
          repairEnabled: options?.repairEnabled,
          classTextEnabled: options?.classTextEnabled,
        }),
      'start mod translation',
    )
    if (result) {
      snapshot.value = result
      return true
    }
    return false
  }

  /** 取消当前任务 */
  async function cancel(): Promise<void> {
    await safeCall(() => modTranslationCancel(), 'cancel mod translation')
  }

  /** 主动拉取状态（页面挂载 / 事件丢失兜底） */
  async function refreshStatus(): Promise<void> {
    const status = await safeCall(() => modTranslationStatus(), 'query mod translation status')
    if (status) snapshot.value = status
  }

  /** 重置为初始状态（重新选择 JAR 时调用） */
  function reset(): void {
    analyzeResult.value = null
    snapshot.value = null
  }

  const { start: startListen } = useTauriEvent<ModTranslationTaskSnapshot>(
    MOD_TRANSLATION_EVENT,
    (payload) => {
      snapshot.value = payload
    },
  )

  onMounted(() => {
    startListen()
    refreshStatus()
  })

  /**
   * 注册页面局部拖放：挂载期间抑制全局拖拽（避免 jar 被当作 Mod 安装），
   * 拖入单个 jar 时回调 onJarDrop；卸载时自动注销并恢复全局拖拽。
   */
  function initDragDrop(onJarDrop: (path: string) => void): void {
    setDragSuppressed(true)
    getCurrentWebview()
      .onDragDropEvent(async (event) => {
        const payload = event.payload
        if (payload.type === 'enter') {
          const paths = payload.paths ?? []
          dragging.value = paths.length === 1 && paths[0].toLowerCase().endsWith('.jar')
        } else if (payload.type === 'leave') {
          dragging.value = false
        } else if (payload.type === 'drop') {
          dragging.value = false
          const paths = payload.paths
          if (paths && paths.length === 1 && paths[0].toLowerCase().endsWith('.jar')) {
            await onJarDrop(paths[0])
          }
        }
      })
      .then((unlisten) => {
        unlistenDrag = unlisten
      })
      .catch((err) => {
        console.warn('[ModTranslation] 注册拖拽监听失败:', err)
      })
  }

  onUnmounted(() => {
    unlistenDrag?.()
    unlistenDrag = null
    setDragSuppressed(false)
  })

  return {
    analyzing,
    analyzeResult,
    snapshot,
    dragging,
    running,
    completed,
    analyze,
    start,
    cancel,
    refreshStatus,
    reset,
    initDragDrop,
  }
}