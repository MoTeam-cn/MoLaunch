//! auth 单元测试

use super::*;

#[test]
fn test_generate_device_id_format() {
    let id = generate_device_id();
    assert!(id.starts_with("mcsdk-"));
    // 4 段，每段 4 字符十六进制
    let parts: Vec<&str> = id.split('-').collect();
    assert_eq!(parts.len(), 5);
    assert_eq!(parts[0], "mcsdk");
    for part in &parts[1..] {
        assert_eq!(part.len(), 4);
        assert!(part.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

#[test]
fn test_build_login_request_round_trip() {
    // 模拟设备已注册的状态
    let kp = OnlineKeyPair::generate();
    let creds = DeviceCredentials {
        ed25519_seed_b64u: b64u_encode(&kp.ed25519.seed()),
        x25519_secret_b64u: b64u_encode(&kp.x25519.secret_bytes()),
        device_pk: "test-device-pk".to_string(),
        device_public_key_b64u: kp.x25519.public_b64u(), // 用自己公钥模拟云端公钥（仅测试流程）
        device_id: "mcsdk-test".to_string(),
        ..Default::default()
    };

    // 由于云端公钥 = 自己公钥，ECDH 会产生 shared（虽然不真实，但流程可走通）
    let req = build_login_request(&creds);
    // 由于 ECDH 需要真正的对方公钥，这里可能失败，但能验证流程
    assert!(req.is_ok(), "登录请求构造应成功");
    let req = req.unwrap();
    assert_eq!(req.device_pk, "test-device-pk");
    assert_eq!(req.v, PROTOCOL_VERSION);
    assert!(!req.signature.is_empty());
    assert!(!req.content.is_empty());
}
