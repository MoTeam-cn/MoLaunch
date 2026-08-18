/** experimental_manager action 名称，与后端 dispatcher 注册一致。 */
export const EXPERIMENTAL_ACTIONS = {
  CREATE_CONVERSATION: 'create_conversation',
  LIST_CONVERSATIONS: 'list_conversations',
  DELETE_CONVERSATION: 'delete_conversation',
  RENAME_CONVERSATION: 'rename_conversation',
  LIST_MESSAGES: 'list_messages',
  LIST_TOOL_CALLS: 'list_tool_calls',
  CLEAR_CONVERSATION: 'clear_conversation',
  CHAT_SEND: 'chat_send',
  COLLECT_CONTEXT: 'collect_context',
  DELETE_MESSAGE: 'delete_message',
  REGENERATE_REPLY: 'regenerate_reply',
  EDIT_MESSAGE: 'edit_message',
  REPLY_ASK_USER: 'reply_ask_user',
  CANCEL_CHAT: 'cancel_chat',
  CANCEL_LOG_ANALYZE: 'cancel_log_analyze',
  LIST_INSTALLED_VERSIONS: 'list_installed_versions',
  AI_ANALYZE_LOG: 'ai_analyze_log',
  MOD_TRANSLATION_ANALYZE: 'mod_translation_analyze',
  MOD_TRANSLATION_START: 'mod_translation_start',
  MOD_TRANSLATION_CANCEL: 'mod_translation_cancel',
  MOD_TRANSLATION_STATUS: 'mod_translation_status',
} as const

export type ExperimentalAction = typeof EXPERIMENTAL_ACTIONS[keyof typeof EXPERIMENTAL_ACTIONS]
