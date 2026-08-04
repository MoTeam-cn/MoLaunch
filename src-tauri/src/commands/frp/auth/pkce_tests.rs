use super::*;

#[test]
fn test_verifier_length_and_charset() {
    let verifier = generate_code_verifier();
    assert!(verifier.len() >= 43 && verifier.len() <= 128);
    assert!(verifier.chars().all(|c| c.is_ascii_alphanumeric()
        || c == '-'
        || c == '.'
        || c == '_'
        || c == '~'));
}

#[test]
fn test_verifier_unique() {
    let a = generate_code_verifier();
    let b = generate_code_verifier();
    assert_ne!(a, b);
}

#[test]
fn test_challenge_deterministic() {
    // RFC 7636 附录 B 示例
    let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    let expected = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
    assert_eq!(code_challenge_s256(verifier), expected);
}

#[test]
fn test_challenge_nopad_urlsafe() {
    let verifier = generate_code_verifier();
    let challenge = code_challenge_s256(&verifier);
    assert!(!challenge.contains('='));
    assert!(!challenge.contains('+'));
    assert!(!challenge.contains('/'));
}
