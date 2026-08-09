//! SSE 行解析：负责处理跨 chunk 的行缓冲与 `data:` 块提取。

#[derive(Debug)]
pub(crate) enum SseEvent {
    Done,
    Json(serde_json::Value),
}

/// 单行最大长度（1MB），超限行丢弃
const MAX_LINE_LEN: usize = 1024 * 1024;
/// 未终结行缓冲累计上限（4MB），超限丢弃
const MAX_BUF_LEN: usize = 4 * 1024 * 1024;

pub(crate) struct SseLineBuffer {
    buf: String,
    dropped_lines: u64,
}

impl SseLineBuffer {
    pub(crate) fn new() -> Self {
        Self {
            buf: String::new(),
            dropped_lines: 0,
        }
    }

    pub(crate) fn push(&mut self, chunk: &[u8]) -> Vec<SseEvent> {
        self.buf.push_str(&String::from_utf8_lossy(chunk));
        let mut events = Vec::new();
        while let Some(nl) = self.buf.find('\n') {
            let line = self.buf[..nl].to_string();
            self.buf.drain(..=nl);
            if line.len() > MAX_LINE_LEN {
                self.dropped_lines += 1;
                continue;
            }
            if let Some(event) = parse_line(&line) {
                events.push(event);
            }
        }
        // 未终结行缓冲超限：丢弃，防内存放大
        if self.buf.len() > MAX_BUF_LEN {
            self.dropped_lines += 1;
            self.buf.clear();
        }
        events
    }

    pub(crate) fn finish(&mut self) -> Option<SseEvent> {
        if self.buf.trim().is_empty() {
            return None;
        }
        let line = std::mem::take(&mut self.buf);
        if line.len() > MAX_LINE_LEN {
            self.dropped_lines += 1;
            return None;
        }
        parse_line(&line)
    }

    #[allow(dead_code)]
    pub(crate) fn dropped_lines(&self) -> u64 {
        self.dropped_lines
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
