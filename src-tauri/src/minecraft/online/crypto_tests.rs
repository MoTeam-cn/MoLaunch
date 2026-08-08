//! crypto 单元测试

use super::*;
use ed25519_dalek::Verifier;

#[test]
fn test_ed25519_sign_verify() {
    let kp = Ed25519KeyPair::generate();
    let msg = b"hello world";
    let sig = kp.sign(msg);
    // 验证签名
    let verifying_key = kp.verifying_key;
    verifying_key
        .verify(msg, &Signature::from_bytes(&sig))
        .expect("signature should verify");
}

#[test]
fn test_x25519_ecdh_round_trip() {
    let alice = X25519StaticKeyPair::generate();
    let bob = X25519StaticKeyPair::generate();
    let shared_a = alice.diffie_hellman(&bob.public);
    let shared_b = bob.diffie_hellman(&alice.public);
    assert_eq!(shared_a, shared_b, "ECDH 共享密钥应一致");
}

#[test]
fn test_aes_gcm_round_trip() {
    let key = [42u8; 32];
    let plaintext = b"secret payload";
    let ciphertext = aes_gcm_encrypt(&key, plaintext).unwrap();
    let decrypted = aes_gcm_decrypt(&key, &ciphertext).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_hkdf_deterministic() {
    let ikm = [1u8; 32];
    // 固定测试盐（HKDF 需确定性输出，非真实密钥）
    let fixed_input = [2u8; 16];
    let info = b"mosign-v1-session-key";
    let k1 = hkdf_sha256(&ikm, &fixed_input, info, 32).unwrap();
    let k2 = hkdf_sha256(&ikm, &fixed_input, info, 32).unwrap();
    assert_eq!(k1, k2, "相同输入应派生相同密钥");
}
