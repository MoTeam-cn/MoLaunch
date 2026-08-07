use serde_json::Value;

use super::super::super::agent;
use super::super::super::types::ReplyAskUserParams;

pub(crate) async fn reply_ask_user(params: ReplyAskUserParams) -> Result<Value, String> {
    agent::reply_ask_user(params.conversation_id, params.reply).await?;
    serde_json::to_value(()).map_err(|e| e.to_string())
}
