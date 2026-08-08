//! PoW Challenge 客户端求解单元测试（同目录 pow_test.rs，随包编译）

use super::*;

#[test]
fn test_leading_zero_bits() {
    assert_eq!(leading_zero_bits(&[0x01]), 7);
    assert_eq!(leading_zero_bits(&[0x00, 0x80]), 8);
    assert_eq!(leading_zero_bits(&[0xff]), 0);
    assert_eq!(leading_zero_bits(&[0x00, 0x00, 0x01]), 23);
}

#[test]
fn test_parse_challenge() {
    let body = r#"{"code":1007,"msg":"pow_challenge_required","data":{"challenge_id":"abc","salt":"00ff","difficulty":20,"ttl":120,"path":"/v3/auth/register","header_name":"x-molaunch-pow"}}"#;
    let c = parse_challenge(body).expect("should parse");
    assert_eq!(c.challenge_id, "abc");
    assert_eq!(c.difficulty, 20);
    assert_eq!(c.path, "/v3/auth/register");
    assert_eq!(c.header_name, "x-molaunch-pow");
    assert_eq!(c.salt_bytes(), Some(vec![0x00, 0xff]));
}

#[test]
fn test_parse_challenge_defaults_header_name() {
    // 旧服务端未下发 header_name 时，回退到默认 x-molaunch-pow
    let body = r#"{"code":1007,"msg":"pow_challenge_required","data":{"challenge_id":"abc","salt":"00ff","difficulty":20,"ttl":120,"path":"/v3/auth/register"}}"#;
    let c = parse_challenge(body).expect("should parse");
    assert_eq!(c.header_name, POW_HEADER);
}

#[test]
fn test_parse_challenge_rejects_other_codes() {
    let body = r#"{"code":1001,"msg":"bad","data":null}"#;
    assert!(parse_challenge(body).is_none());
}

#[test]
fn test_solve_sync_matches_difficulty() {
    // 固定测试输入（PoW 求解需确定性结果，非真实密钥）
    let fixed_input = b"test-salt";
    let difficulty = 8; // 期望 ~256 次尝试，耗时可控
    let nonce = solve_sync(fixed_input, difficulty).expect("solved");
    let mut hasher = Sha256::new();
    hasher.update(fixed_input);
    hasher.update(nonce.to_le_bytes());
    let digest = hasher.finalize();
    assert!(leading_zero_bits(&digest) >= difficulty);
}
