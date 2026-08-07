import { computed, onMounted, ref } from 'vue'
import { useTauriEvent } from '@/composables/useTauriEvent'
import { aiLoadConfig } from '@/utils/api/ai'
import {
  experimentalManager,
  experimentalAiAnalyzeLog,
  EXPERIMENTAL_ACTIONS,
  type AiAnalyzeStreamEvent,
} from '@/utils/api/experimental'
import { toastError } from '@/utils/toast'

export function useAiLogAnalyzer(stageCount: number) {
  const model = ref('')
  const models = ref<string[]>([])
  const analyzing = ref(false)
  const currentIndex = ref(-1)
  const conclusion = ref('')
  const reasoning = ref('')
  const modelOptions = computed(() => models.value.map((item) => ({ label: item, value: item })))

  const streamEvent = useTauriEvent<AiAnalyzeStreamEvent>('ai-analyze-stream', (event) => {
    if (!analyzing.value) return
    if (typeof event.step === 'number' && event.step >= 1 && event.step <= stageCount) {
      currentIndex.value = event.step - 1
    }
    if (event.reasoning) reasoning.value += event.reasoning
    if (event.delta) conclusion.value += event.delta
    if (event.done) {
      currentIndex.value = stageCount - 1
      if (event.content) conclusion.value = event.content
      analyzing.value = false
    }
    if (event.error) {
      analyzing.value = false
      toastError(`AI 分析失败：${event.error}`)
    }
    if (event.cancelled) analyzing.value = false
  })

  async function loadModels() {
    streamEvent.start()
    try {
      const config = await aiLoadConfig()
      models.value = config.models ?? []
      model.value = config.defaultModel || config.models?.[0] || ''
    } catch {
      models.value = []
    }
  }

  function resetResult() {
    currentIndex.value = -1
    conclusion.value = ''
    reasoning.value = ''
  }

  async function runAnalyze(source: string) {
    if (!source.trim()) {
      toastError('未获取到日志内容')
      return
    }
    if (!model.value) {
      toastError('请先选择 AI 模型')
      return
    }
    analyzing.value = true
    resetResult()
    try {
      await experimentalAiAnalyzeLog({
        logText: source,
        model: model.value,
        reasoningEffort: null,
        localAnalyze: true,
      })
    } catch (error) {
      analyzing.value = false
      toastError(`AI 分析失败: ${error instanceof Error ? error.message : String(error)}`)
    }
  }

  function cancel() {
    if (!analyzing.value) return
    analyzing.value = false
    void experimentalManager<void>(EXPERIMENTAL_ACTIONS.CANCEL_LOG_ANALYZE).catch(() => {})
  }

  onMounted(loadModels)

  return {
    model,
    models,
    analyzing,
    currentIndex,
    conclusion,
    reasoning,
    modelOptions,
    runAnalyze,
    cancel,
  }
}
