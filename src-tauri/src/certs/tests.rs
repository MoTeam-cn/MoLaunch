//! certs 单元测试

use super::*;

#[test]
fn test_validate_filename_valid() {
    assert!(validate_filename("my-cert.pem").is_ok());
    assert!(validate_filename("root_2024.PEM").is_ok());
    assert!(validate_filename("ca-1.crt").is_ok());
}

#[test]
fn test_validate_filename_invalid() {
    assert!(validate_filename("").is_err());
    assert!(validate_filename("../evil.pem").is_err());
    assert!(validate_filename("path\\to\\cert.pem").is_err());
    assert!(validate_filename("ca cert.pem").is_err());
    assert!(validate_filename("ca:cert.pem").is_err());
}