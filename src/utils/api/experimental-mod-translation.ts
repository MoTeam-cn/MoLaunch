/**
 * 实验性 - 模组翻译 API 封装
 *
 * 后端 action：mod_translation_analyze / start / cancel / status，
 * 进度经 `mod-translation-event` 事件推送（见 useModTranslation）。
 */
import { experimentalManager } from './experimental'
import { EXPERIMENTAL_ACTIONS } from './experimental-actions'

export interface ModTranslationSourceSummary {
  kind: 'json' | 'key-value' | 'structured-json' | 'free-text'
  namespace: string
  sourcePath: string
  targetPath: string
  entries: number
}

export interface ModTranslationAnalyzeResult {
  filename: string
  loader: string
  modIds: string[]
  signed: boolean
  sources: ModTranslationSourceSummary[]
  totalEntries: number
  warnings: string[]
}

export interface ModTranslationTaskSnapshot {
  taskId: string
  /** idle / running / completed / failed / cancelled */
  status: string
  /** analyze / translate / package */
  stage: string
  progress: number
  message: string
  outputPath: string | null
  error: string | null
}

export interface ModTranslationStartParams {
  jarPath: string
  model: string
  batchSize: number
}

export function modTranslationAnalyze(jarPath: string): Promise<ModTranslationAnalyzeResult> {
  return experimentalManager<ModTranslationAnalyzeResult>(EXPERIMENTAL_ACTIONS.MOD_TRANSLATION_ANALYZE, {
    jarPath,
  })
}

export function modTranslationStart(params: ModTranslationStartParams): Promise<ModTranslationTaskSnapshot> {
  return experimentalManager<ModTranslationTaskSnapshot>(EXPERIMENTAL_ACTIONS.MOD_TRANSLATION_START, params)
}

export function modTranslationCancel(): Promise<void> {
  return experimentalManager<void>(EXPERIMENTAL_ACTIONS.MOD_TRANSLATION_CANCEL)
}

export function modTranslationStatus(): Promise<ModTranslationTaskSnapshot> {
  return experimentalManager<ModTranslationTaskSnapshot>(EXPERIMENTAL_ACTIONS.MOD_TRANSLATION_STATUS)
}