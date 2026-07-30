//! ecies 单元测试

use super::*;
use crate::minecraft::online::crypto::X25519StaticKeyPair;

#[test]
fn test_ecies_round_trip() {
    // 模拟云端：生成静态密钥对，公钥交给客户端
    let server_kp = X25519StaticKeyPair::generate();
    let server_public_b64u = server_kp.public_b64u();

    // 客户端加密（用云端公钥）
    let plaintext = br#"{"room_code":"AB3K7Q","max_players":4}"#;
    let sealed = seal(plaintext, &server_public_b64u).unwrap();

    // 云端解密（用自己私钥）
    let server_secret = server_kp.secret_bytes();
    let decrypted = open(&sealed.envelope, &server_secret).unwrap();
    assert_eq!(decrypted, plaintext);

    // 反向：云端加密（用客户端公钥），客户端解密
    let client_kp = X25519StaticKeyPair::generate();
    let client_public_b64u = client_kp.public_b64u();

    let sealed_resp = seal(b"response data", &client_public_b64u).unwrap();
    let client_secret = client_kp.secret_bytes();
    let decrypted_resp = open(&sealed_resp.envelope, &client_secret).unwrap();
    assert_eq!(decrypted_resp, b"response data");
}

#[test]
fn test_is_envelope() {
    let envelope_json = serde_json::json!({
        "payload": "abc",
        "key": "def"
    });
    assert!(is_envelope(&envelope_json));

    let plain_json = serde_json::json!({
        "code": 1,
        "data": null,
        "msg": "ok"
    });
    assert!(!is_envelope(&plain_json));
}
