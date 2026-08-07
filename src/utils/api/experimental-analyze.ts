import { experimentalManager } from './experimental'
import { EXPERIMENTAL_ACTIONS } from './experimental-actions'

export interface AiAnalyzeLogParams {
  logText: string
  model: string
  reasoningEffort?: string | null
  localAnalyze?: boolean
}

export function experimentalAiAnalyzeLog(params: AiAnalyzeLogParams): Promise<void> {
  return experimentalManager<void>(EXPERIMENTAL_ACTIONS.AI_ANALYZE_LOG, params)
}
