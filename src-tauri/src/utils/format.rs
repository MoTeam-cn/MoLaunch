//! 字节数与速度格式化工具
//!
//! 提供统一的字节数和速度格式化函数，避免在各业务模块中重复实现。

/// 字节数单位（1024 进制）
const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

/// 默认小数位数（1 位，用于下载日志场景）
const DEFAULT_DECIMALS: usize = 1;

/// 格式化字节数为人类可读大小（默认 1 位小数）
pub fn bytes(bytes: u64) -> String {
    bytes_with(bytes, DEFAULT_DECIMALS)
}

/// 格式化字节数为人类可读大小（指定小数位数）
pub fn bytes_with(bytes: u64, decimals: usize) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }

    let k = 1024u64;
    // 通过循环确定单位档位，避免使用 log 计算（f64::log 需手动实现）
    let mut i = 0usize;
    let mut value = bytes as f64;
    while value >= k as f64 && i < UNITS.len() - 1 {
        value /= k as f64;
        i += 1;
    }

    format!("{:.*} {}", decimals, value, UNITS[i])
}

/// 格式化速度为人类可读字符串（默认 1 位小数）
pub fn speed(bytes_per_sec: u64) -> String {
    format!("{}/s", bytes(bytes_per_sec))
}

/// 按字符数截断文本（安全处理中文等多字节字符）
///
/// 超出 `max` 字符时截断并追加省略号；否则原样返回。
pub fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max).collect();
        format!("{}…", cut)
    }
}

/// 截取文本的指定行范围（从 1 起，含首尾），超长时按字符数截断
///
/// 参数无效或起始行超出总行数时返回带行数的说明文本（供工具结果直接展示）。
pub fn read_line_range(
    content: &str,
    start: i64,
    end: i64,
    max_chars: usize,
) -> Result<String, String> {
    if start < 1 || end < start {
        return Err(format!(
            "行范围参数无效: startLine={} endLine={}（需 start>=1 且 end>=start）",
            start, end
        ));
    }
    let all: Vec<&str> = content.lines().collect();
    if start as usize > all.len() {
        return Ok(format!(
            "（日志共 {} 行，起始行 {} 超出范围）",
            all.len(),
            start
        ));
    }
    let end_u = (end as usize).min(all.len());
    let lines = &all[(start as usize - 1)..end_u];
    let joined = lines.join("\n");
    let trimmed = truncate_chars(&joined, max_chars);
    Ok(format!("（共 {} 行）\n{}", lines.len(), trimmed))
}
