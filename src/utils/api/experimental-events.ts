export interface AiChatUsage {
  promptTokens: number
  completionTokens: number
  totalTokens: number
}

export interface AiChatStreamEvent {
  conversationId: number
  delta?: string
  reasoning?: string
  done?: boolean
  toolCall?: {
    name: string
    status: 'running' | 'done'
    index?: string
    arguments?: string
    preContent?: string
    output?: string
  }
  status?: string
  usage?: AiChatUsage
  durationMs?: number
}

export interface AskUserOption {
  label: string
  description?: string
}

export interface AiAskUserEvent {
  conversationId: number
  question: string
  options: AskUserOption[]
}

export interface AiAnalyzeStreamEvent {
  delta?: string
  reasoning?: string
  step?: number
  done?: boolean
  content?: string
  error?: string
  cancelled?: boolean
}
