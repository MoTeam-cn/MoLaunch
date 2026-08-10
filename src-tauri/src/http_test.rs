//! http 工具函数测试

use super::*;

#[derive(Debug)]
struct TestError {
    msg: String,
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for TestError {}

#[derive(Debug)]
struct WrapError {
    inner: TestError,
}

impl std::fmt::Display for WrapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error sending request for url (https://example.com)")
    }
}

impl std::error::Error for WrapError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}

#[test]
fn tls_cert_error_detected() {
    assert!(is_tls_cert_error(&TestError {
        msg: "invalid peer certificate: UnknownIssuer".into()
    }));
    assert!(is_tls_cert_error(&TestError {
        msg: "tls handshake failed".into()
    }));
    assert!(!is_tls_cert_error(&TestError {
        msg: "connection refused".into()
    }));
}

#[test]
fn tls_cert_error_walk_source_chain() {
    let top = WrapError {
        inner: TestError {
            msg: "invalid peer certificate: UnknownIssuer".into(),
        },
    };
    assert!(is_tls_cert_error(&top));
}
