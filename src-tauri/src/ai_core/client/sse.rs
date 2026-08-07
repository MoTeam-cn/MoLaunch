//! SSE 行解析：负责处理跨 chunk 的行缓冲与 `data:` 块提取。

#[derive(Debug)]
pub(crate) enum SseEvent {
    Done,
    Json(serde_json::Value),
}

pub(crate) struct SseLineBuffer {
    buf: String,
}

impl SseLineBuffer {
    pub(crate) fn new() -> Self {
        Self { buf: String::new() }
    }

    pub(crate) fn push(&mut self, chunk: &[u8]) -> Vec<SseEvent> {
        self.buf.push_str(&String::from_utf8_lossy(chunk));
        let mut events = Vec::new();
        while let Some(nl) = self.buf.find('\n') {
            let line = self.buf[..nl].to_string();
            self.buf.drain(..=nl);
            if let Some(event) = parse_line(&line) {
                events.push(event);
            }
        }
        events
    }

    pub(crate) fn finish(&mut self) -> Option<SseEvent> {
        if self.buf.trim().is_empty() {
            return None;
        }
        let line = std::mem::take(&mut self.buf);
        parse_line(&line)
    }
}

fn parse_line(line: &str) -> Option<SseEvent> {
    let line = line.trim();
    if !line.starts_with("data:") {
        return None;
    }
    let data = line[5..].trim();
    if data == "[DONE]" {
        return Some(SseEvent::Done);
    }
    serde_json::from_str(data).ok().map(SseEvent::Json)
}
