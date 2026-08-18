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

/** class 常量池候选（运行时可见字符串，跨文件按文本聚合） */
export interface ModTranslationClassCandidate {
  id: string
  path: string
  paths: string[]
  occurrences: number
  text: string
}

/** token 报价预估（分析阶段展示，供用户评估成本） */
export interface ModTranslationQuote {
  estimatedInputTokens: number
  estimatedOutputTokens: number
  estimatedTokens: number
  estimatedCalls: number
  languageBatches: number
  classBatches: number
  points: number
  characters: number
  entries: number
}

/** 资源覆盖诊断（每个工作区文件的处置结论） */
export interface ModTranslationResourceCoverage {
  path: string
  mediaType: string
  disposition: string
  targetPath: string | null
  textCandidates: number
  reason: string
}

/** 模组中文名决策结果 */
export interface ModTranslationModName {
  name: string
  source: string
}

/** 任务完成报告（终态快照携带） */
export interface ModTranslationReport {
  taskId: string
  ok: boolean
  outputPath: string
  modName: ModTranslationModName | null
  languageAttempted: number
  languageAccepted: number
  classResolved: number
  classTotal: number
  warnings: string[]
}

/** 已存在的中文语言文件（预检：模组自带 zh_cn/zh_tw 时提示覆盖风险） */
export interface ModTranslationExistingChinese {
  path: string
  locale: string
  entries: number
}

export interface ModTranslationAnalyzeResult {
  filename: string
  loader: string
  modIds: string[]
  projectNames: string[]
  version: string | null
  signed: boolean
  sources: ModTranslationSourceSummary[]
  totalEntries: number
  classCandidates: ModTranslationClassCandidate[]
  quote: ModTranslationQuote
  coverage: ModTranslationResourceCoverage[]
  modName: ModTranslationModName | null
  existingChinese: ModTranslationExistingChinese[]
  warnings: string[]
}

export interface ModTranslationTaskSnapshot {
  taskId: string
  /** idle / running / completed / failed / cancelled */
  status: string
  /** analyze / language / repair / class / validation / package */
  stage: string
  /** 总进度（0-100，按阶段权重加权计算） */
  progress: number
  /** 当前阶段分进度（0-100） */
  stageProgress: number
  /** 重试信息（重试时携带） */
  retry: { attempt: number; total: number } | null
  /** 各阶段进度（分进度折叠区展示） */
  stages: { stage: string; weight: number; progress: number }[]
  message: string
  outputPath: string | null
  error: string | null
  modName: ModTranslationModName | null
  report: ModTranslationReport | null
}

export interface ModTranslationStartParams {
  jarPath: string
  model: string
  batchSize: number
  /** 是否生成模组中文名（默认开） */
  generateModName?: boolean
  /** 是否启用质量回修兜底（默认开） */
  repairEnabled?: boolean
  /** 是否翻译 class 常量池文本（默认开） */
  classTextEnabled?: boolean
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