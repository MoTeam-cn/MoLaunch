//! PEM 元信息解析（简易实现，避免引入 x509-parser 依赖）

/// 从 PEM 字节中解析 Subject CN 和 NOT AFTER 时间
///
/// 简易实现：base64 解码后查找 `CN=` 和 `NOT AFTER` 子串。
/// 解析失败时 `subject` 回退为 `fallback_filename`，`not_after` 为空字符串。
pub(super) fn parse_pem_meta(pem_bytes: &[u8], fallback_filename: &str) -> (String, String) {
    // PEM → base64 解码（提取 BEGIN/END 之间的内容）
    let pem_str = match std::str::from_utf8(pem_bytes) {
        Ok(s) => s,
        Err(_) => return (fallback_filename.to_string(), String::new()),
    };

    let mut b64_lines = Vec::new();
    let mut in_body = false;
    for line in pem_str.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("-----BEGIN") {
            in_body = true;
            continue;
        }
        if trimmed.starts_with("-----END") {
            break;
        }
        if in_body && !trimmed.is_empty() {
            b64_lines.push(trimmed);
        }
    }

    if b64_lines.is_empty() {
        return (fallback_filename.to_string(), String::new());
    }

    let b64_content: String = b64_lines.concat();
    use base64::Engine;
    let der_bytes = match base64::engine::general_purpose::STANDARD.decode(&b64_content) {
        Ok(b) => b,
        Err(_) => return (fallback_filename.to_string(), String::new()),
    };

    // 在 DER 字节中查找 ASCII 子串（CN= / NOT AFTER）
    // 这种方式对常见 X.509 证书有效，但不保证所有证书都能匹配
    let der_str: Vec<u8> = der_bytes
        .iter()
        .map(|&b| {
            if b.is_ascii_graphic() || b == b' ' {
                b
            } else {
                b' '
            }
        })
        .collect();
    let der_text = String::from_utf8_lossy(&der_str);

    let subject = extract_cn(&der_text, fallback_filename);
    let not_after = extract_not_after(&der_text);

    (subject, not_after)
}

/// 从 DER 文本中提取 CN= 后的内容
fn extract_cn(text: &str, fallback: &str) -> String {
    if let Some(pos) = text.find("CN=") {
        let start = pos + 3;
        let end = text[start..]
            .find([',', '/', '\n'])
            .map(|i| start + i)
            .unwrap_or(text.len());
        let cn = text[start..end].trim();
        if !cn.is_empty() {
            return cn.to_string();
        }
    }
    fallback.to_string()
}

/// 从 DER 文本中提取 NOT AFTER 后的时间字符串
fn extract_not_after(text: &str) -> String {
    // X.509 v3 证书中常见 "Not After" 标签（UTCTime 或 GeneralizedTime）
    let markers = ["Not After : ", "Not After ", "NOT AFTER:"];
    for marker in markers {
        if let Some(pos) = text.find(marker) {
            let start = pos + marker.len();
            let end = text[start..]
                .find(['\n', ','])
                .map(|i| start + i)
                .unwrap_or((start + 24).min(text.len()));
            let value = text[start..end].trim();
            if !value.is_empty() {
                return value.to_string();
            }
        }
    }
    String::new()
}
