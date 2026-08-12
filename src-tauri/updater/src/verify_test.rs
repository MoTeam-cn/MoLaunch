//! verify 模块测试：签名格式兼容性（tauri base64 编码 / 标准 4 行文本）

use super::parse_signature;
use base64::engine::{general_purpose, Engine as _};

/// tauri 打包产物 `.sig` 内容（base64 编码），取自 v0.3.5-rc9 实际发布数据
const SIG_B64: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVRWElKOUZSeXBZRXZFaHcySTNWcDNCVm0wcUJ6d25vcHd2V1dPS0pXMHFQNnN3cnFnL3RCd05adUE5eEI1NlFrd3dBTk5hZDhTN1VIM0JtdGZlOHh5Zzk0MXU1UlhUUUFNPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzg2NTI2MjQ4CWZpbGU6TW9MYXVuY2hfMC4zLjUtcmM5X3g2NC5leGUKb2dYYlFCd2JMSHBJTFhDNyt4Zy9TR0VzT2pzOFF6RmVET01sL3dmMG5TWVRZcHFCcFhvZEJFYTRLbUpuajJoUE05OFhzem1RK3dTM25qNUlveGZIRHc9PQo=";

#[test]
fn parse_tauri_b64_sig() {
    assert!(parse_signature(SIG_B64).is_ok());
}

#[test]
fn parse_plain_text_sig() {
    let text = String::from_utf8(general_purpose::STANDARD.decode(SIG_B64).unwrap()).unwrap();
    assert!(text.starts_with("untrusted comment:"));
    assert!(parse_signature(&text).is_ok());
}
