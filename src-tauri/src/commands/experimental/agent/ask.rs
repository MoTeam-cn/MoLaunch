//! Agent ask_user 工具：向用户提问并等待回答

use once_cell::sync::Lazy;
use serde_json::json;
use std::collections::HashMap;
use tauri::Emitter;
use tokio::sync::{oneshot, Mutex};

use super::tools::AgentContext;

/// ask_user 等待超时（秒）
const ASK_USER_TIMEOUT_SECS: u64 = 120;

/// ask_user 等待队列：conversation_id → reply 发送端
pub static ASK_USER_QUEUE: Lazy<Mutex<HashMap<i64, oneshot::Sender<String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// ask_user：emit ai-ask-user 事件并等待用户回答
pub(super) async fn ask_user(
    args: &serde_json::Value,
    ctx: &AgentContext,
) -> Result<String, String> {
    let question = args
        .get("question")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "ask_user 缺少 question 参数".to_string())?;

    // options 兼容纯字符串 `"选项"` 与对象 `{"label": "...", "description": "备注"}`，
    // 统一归一化为 `{label, description?}` 后透传前端展示
    let options: Vec<serde_json::Value> = args
        .get("options")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    if let Some(s) = v.as_str() {
                        Some(json!({ "label": s }))
                    } else if v.get("label").and_then(|l| l.as_str()).is_some() {
                        Some(v.clone())
                    } else {
                        None
                    }
                })
                .take(6)
                .collect()
        })
        .unwrap_or_default();

    // 注册等待通道；同一会话已有提问未回答时先移除旧的（防堆积）
    let (tx, rx) = oneshot::channel::<String>();
    {
        let mut queue = ASK_USER_QUEUE.lock().await;
        let _ = queue.insert(ctx.conversation_id, tx);
    }

    // 通知前端弹窗询问
    let _ = ctx.app.emit(
        "ai-ask-user",
        json!({
            "conversationId": ctx.conversation_id,
            "question": question,
            "options": options
        }),
    );

    // 等待回答（超时保护，防止模型提问后无人回应挂死工具循环）
    let wait_outcome =
        tokio::time::timeout(std::time::Duration::from_secs(ASK_USER_TIMEOUT_SECS), rx).await;
    match wait_outcome {
        Ok(Ok(reply)) => {
            let reply = reply.trim().to_string();
            if reply.is_empty() {
                Ok("（用户未提供有效回答）".to_string())
            } else {
                Ok(format!("用户回答: {}", reply))
            }
        }
        Ok(Err(_)) => {
            // 发送端被丢弃（例如会话被切换），清理队列
            let mut queue = ASK_USER_QUEUE.lock().await;
            queue.remove(&ctx.conversation_id);
            Err("提问等待被中断".to_string())
        }
        Err(_) => {
            let mut queue = ASK_USER_QUEUE.lock().await;
            queue.remove(&ctx.conversation_id);
            Err("向用户提问超时（120s 内未收到回答）".to_string())
        }
    }
}

/// 回填 ask_user 回答（由 `reply_ask_user` action 调用）
pub async fn reply_ask_user(conversation_id: i64, reply: String) -> Result<(), String> {
    let mut queue = ASK_USER_QUEUE.lock().await;
    if let Some(sender) = queue.remove(&conversation_id) {
        let _ = sender.send(reply);
        Ok(())
    } else {
        Err("没有正在等待的回答".to_string())
    }
}
