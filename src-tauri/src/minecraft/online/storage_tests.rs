//! storage 单元测试

use super::*;

#[test]
fn test_is_registered() {
    let empty = DeviceCredentials::default();
    assert!(!empty.is_registered());

    let creds = DeviceCredentials {
        device_pk: "uuid".to_string(),
        ed25519_seed_b64u: "seed".to_string(),
        x25519_secret_b64u: "sec".to_string(),
        device_public_key_b64u: "pub".to_string(),
        ..Default::default()
    };
    assert!(creds.is_registered());
}

#[test]
fn test_is_token_expired() {
    let mut creds = DeviceCredentials::default();
    // token_expires_at = 0 视为已过期
    assert!(creds.is_token_expired());

    // 设为未来 1 小时
    let future = (chrono::Utc::now().timestamp() + 3600) as u64;
    creds.token_expires_at = future;
    assert!(!creds.is_token_expired());

    // 设为过去
    let past = (chrono::Utc::now().timestamp() - 100) as u64;
    creds.token_expires_at = past;
    assert!(creds.is_token_expired());
}
