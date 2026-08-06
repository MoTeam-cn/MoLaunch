/**
 * AI 模块 API（本地 OpenAI 兼容服务）
 *
 * 对应后端 `experimental_manager` IPC 命令（AI action 已并入实验性统一分发，
 * 不再使用独立的 ai_manager），通过 `action` 字段分发到不同子模块。
 * 服务为本地 OpenAI 兼容 API（如 Ollama / LM Studio），不依赖云端。
 *
 * 注意：需先开启「设置 → 进阶设置 → 实验性功能」，否则后端返回错误。
 */

import { invoke } from '@tauri-apps/api/core'

/** 调用 experimental_manager IPC（AI action） */
export async function aiManager<T = unknown>(action: string, params?: unknown): Promise<T> {
  return invoke<T>('experimental_manager', { req: { action, params: params ?? null } })
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

/** AI 服务配置（与后端 AiConfig 对应，字段为 camelCase） */
export interface AiConfig {
  /** 服务地址，如 http://127.0.0.1:11434/v1 */
  baseUrl: string
  /** API Key（明文，后端保存时经 SDK DES 加密写入 config.ini） */
  apiKey: string
  /** 超时秒数 */
  timeoutSecs: number
  /** 上下文窗口 token 上限（输入侧，超限自动压缩上下文） */
  maxInputTokens?: number
  /** 单次回复最大输出 token（作为 max_tokens 下发） */
  maxOutputTokens?: number
  /** 已启用（导入）的模型列表 */
  models: string[]
  /** 默认模型名 */
  defaultModel: string
  /** 模型图标样式：color（彩色）/ mono（黑白），见 utils/model-icon-mode.ts */
  iconColorMode?: string
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

/** AI 分析结果（与后端 AiAnalysisResult 对应，字段为 camelCase） */
export interface AiAnalysisResult {
  /** 模型回复（Markdown） */
  content: string
  /** 使用的模型名 */
  model: string
  /** 耗时毫秒 */
  elapsedMs: number
}

/** 连接状态（与后端 AiStatusResult 对应，字段为 camelCase） */
export interface AiStatusResult {
  /** 是否可用 */
  available: boolean
  /** 服务地址 */
  baseUrl: string
  /** 默认模型名 */
  model: string
}

/** 服务探测参数（前端表单当前值，避免未保存时探测旧配置；字段为 camelCase） */
export interface AiProbeParams {
  /** 服务地址 */
  baseUrl: string
  /** API Key */
  apiKey: string
  /** 超时秒数 */
  timeoutSecs: number
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
