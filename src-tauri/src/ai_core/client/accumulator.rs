//! 流式响应累积：处理文本、推理内容、usage 与工具调用增量。

use super::types::{
    ChatResult, StreamCallbacks, StreamToolDelta, StreamUsage, ToolCall, ToolCallFunction,
};

pub(crate) struct StreamAccumulator<'a> {
    pub(crate) content: String,
    pub(crate) reasoning_content: String,
    pub(crate) tool_calls: Vec<ToolCall>,
    pub(crate) usage: StreamUsage,
    callbacks: &'a StreamCallbacks,
}

impl<'a> StreamAccumulator<'a> {
    pub(crate) fn new(callbacks: &'a StreamCallbacks) -> Self {
        Self {
            content: String::new(),
            reasoning_content: String::new(),
            tool_calls: Vec::new(),
            usage: StreamUsage::default(),
            callbacks,
        }
    }

    pub(crate) fn apply(&mut self, parsed: &serde_json::Value) -> Option<ChatResult> {
        let choice = parsed
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|c| c.first())?;
        self.update_usage(parsed);
        let finish = choice.get("finish_reason").and_then(|v| v.as_str());
        let delta = choice.get("delta");
        self.append_text(delta);
        self.append_reasoning(delta);
        self.append_tool_calls(delta);
        if finish == Some("tool_calls") || finish == Some("stop") {
            super::chat::finalize_tool_calls(&mut self.tool_calls);
            (self.callbacks.on_done)(&self.usage);
            return Some(self.finish(true));
        }
        None
    }

    pub(crate) fn finish(&mut self, keep_tool_calls: bool) -> ChatResult {
        ChatResult {
            content: non_empty(&mut self.content),
            reasoning_content: non_empty(&mut self.reasoning_content),
            tool_calls: if keep_tool_calls {
                std::mem::take(&mut self.tool_calls)
            } else {
                Vec::new()
            },
        }
    }

    fn update_usage(&mut self, parsed: &serde_json::Value) {
        if let Some(u) = parsed.get("usage") {
            self.usage = StreamUsage {
                prompt_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                completion_tokens: u
                    .get("completion_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
            };
        }
    }

    fn append_text(&mut self, delta: Option<&serde_json::Value>) {
        if let Some(text) = delta
            .and_then(|d| d.get("content"))
            .and_then(|v| v.as_str())
        {
            if !text.is_empty() {
                self.content.push_str(text);
                (self.callbacks.on_delta)(text);
            }
        }
    }

    fn append_reasoning(&mut self, delta: Option<&serde_json::Value>) {
        if let Some(text) = delta
            .and_then(|d| d.get("reasoning_content"))
            .and_then(|v| v.as_str())
        {
            if !text.is_empty() {
                self.reasoning_content.push_str(text);
                (self.callbacks.on_reasoning_delta)(text);
            }
        }
    }

    fn append_tool_calls(&mut self, delta: Option<&serde_json::Value>) {
        let Some(calls) = delta
            .and_then(|d| d.get("tool_calls"))
            .and_then(|v| v.as_array())
        else {
            return;
        };
        for call in calls {
            let index = call.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            while self.tool_calls.len() <= index {
                self.tool_calls.push(ToolCall {
                    id: String::new(),
                    ty: "function".to_string(),
                    function: ToolCallFunction {
                        name: String::new(),
                        arguments: String::new(),
                    },
                });
            }
            let target = &mut self.tool_calls[index];
            if let Some(id) = call.get("id").and_then(|v| v.as_str()) {
                target.id = id.to_string();
            }
            if let Some(name) = call
                .get("function")
                .and_then(|f| f.get("name"))
                .and_then(|v| v.as_str())
            {
                target.function.name = name.to_string();
            }
            let arguments = call
                .get("function")
                .and_then(|f| f.get("arguments"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            target.function.arguments.push_str(&arguments);
            if !arguments.is_empty() {
                (self.callbacks.on_tool_delta)(&StreamToolDelta {
                    index,
                    id: None,
                    name: None,
                    arguments,
                });
            }
        }
    }
}

fn non_empty(value: &mut String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(std::mem::take(value))
    }
}
