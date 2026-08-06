//! Token 估算（本地估算，供上下文窗口用量展示）
//!
//! 与前端 `src/utils/tokens.ts` 保持一致：CJK 字符每个约 1 token，
//! 其余字符每 4 个约 1 token。服务端响应后可基于实际 usage 校准。

/// 估算文本 token 数
pub fn estimate_tokens(text: &str) -> u64 {
    let mut cjk = 0u64;
    let mut other = 0u64;
    for c in text.chars() {
        if is_cjk(c) {
            cjk += 1;
        } else {
            other += 1;
        }
    }
    cjk + other.div_ceil(4)
}

fn is_cjk(c: char) -> bool {
    matches!(c as u32,
        0x4E00..=0x9FFF   // CJK 统一表意文字
        | 0x3400..=0x4DBF // 扩展 A
        | 0x20000..=0x2A6DF // 扩展 B
        | 0x3000..=0x303F // 标点
        | 0xFF00..=0xFFEF // 全角
    )
}
