/**
 * AI 分析模块 API（本地 OpenAI 兼容服务）
 *
 * 对应后端 `ai_manager` IPC 命令，通过 `action` 字段分发到不同子模块。
 * 服务为本地 OpenAI 兼容 API（如 Ollama / LM Studio），不依赖云端。
 */

import { invoke } from '@tauri-apps/api/core'

/** 调用 ai_manager IPC */
export async function aiManager<T = unknown>(action: string, params?: unknown): Promise<T> {
  return invoke<T>('ai_manager', { req: { action, params: params ?? null } })
}

/** 所有可用的 action 名称（与后端 commands::ai::manager 注册一致） */
export const AI_ACTIONS = {
  /** 崩溃日志 AI 分析 */
  ANALYZE_CRASH: 'analyze_crash',
  /** 检测本地 AI 服务是否可用 */
  CHECK_STATUS: 'check_status',
  /** 保存 AI 配置 */
  SAVE_CONFIG: 'save_config',
  /** 读取 AI 配置 */
  LOAD_CONFIG: 'load_config',
  /** 拉取服务端模型列表 */
  LIST_MODELS: 'list_models',
} as const

/** action 名称类型 */
export type AiAction = typeof AI_ACTIONS[keyof typeof AI_ACTIONS]

/** AI 服务配置（与后端 AiConfig 对应） */
export interface AiConfig {
  /** 服务地址，如 http://127.0.0.1:11434/v1 */
  base_url: string
  /** API Key（明文，后端保存时经 SDK DES 加密写入 config.ini） */
  api_key: string
  /** 超时秒数 */
  timeout_secs: number
  /** 已启用（导入）的模型列表 */
  models: string[]
  /** 默认模型名 */
  default_model: string
}

/** 崩溃日志 AI 分析参数（与后端 AnalyzeCrashParams 对应） */
export interface AnalyzeCrashParams {
  /** 运行时日志全文 */
  runtimeLog: string
  /** 错误级别日志行 */
  errorLines?: string[]
  /** 崩溃报告文本 */
  crashReport?: string
  /** hs_err 日志文本 */
  hsErr?: string
  /** 可选显式指定模型；为空时使用默认模型 */
  model?: string
}

/** AI 分析结果（与后端 AiAnalysisResult 对应） */
export interface AiAnalysisResult {
  /** 模型回复（Markdown） */
  content: string
  /** 使用的模型名 */
  model: string
  /** 耗时毫秒 */
  elapsed_ms: number
}

/** 连接状态（与后端 AiStatusResult 对应） */
export interface AiStatusResult {
  /** 是否可用 */
  available: boolean
  /** 服务地址 */
  base_url: string
  /** 默认模型名 */
  model: string
}

/** 服务探测参数（前端表单当前值，避免未保存时探测旧配置） */
export interface AiProbeParams {
  /** 服务地址 */
  base_url: string
  /** API Key */
  api_key: string
  /** 超时秒数 */
  timeout_secs: number
}

/** 崩溃日志 AI 分析 */
export function aiAnalyzeCrash(params: AnalyzeCrashParams): Promise<AiAnalysisResult> {
  return aiManager<AiAnalysisResult>(AI_ACTIONS.ANALYZE_CRASH, params)
}

/** 检测本地 AI 服务是否可用（可传当前表单值探测） */
export function aiCheckStatus(params?: AiProbeParams): Promise<AiStatusResult> {
  return aiManager<AiStatusResult>(AI_ACTIONS.CHECK_STATUS, params ?? null)
}

/** 保存 AI 配置 */
export function aiSaveConfig(config: AiConfig): Promise<void> {
  return aiManager<void>(AI_ACTIONS.SAVE_CONFIG, config)
}

/** 读取 AI 配置 */
export function aiLoadConfig(): Promise<AiConfig> {
  return aiManager<AiConfig>(AI_ACTIONS.LOAD_CONFIG)
}

/** 拉取服务端模型列表（OpenAI 兼容 GET /models，返回模型 id 数组） */
export function aiListModels(params: AiProbeParams): Promise<string[]> {
  return aiManager<string[]>(AI_ACTIONS.LIST_MODELS, params)
}
