//! 失败码单测：0 与非 0 的语义边界不能乱

use super::{CANCELLED, DISK, INSTALL, NETWORK, NONE, VERIFY};

#[test]
fn test_error_code_零表示没有错误() {
    assert_eq!(NONE, 0);
}

#[test]
fn test_error_code_各码互不相同且非零() {
    let codes = [NETWORK, VERIFY, DISK, CANCELLED, INSTALL];
    for (index, code) in codes.iter().enumerate() {
        assert_ne!(*code, NONE, "失败码不能是 0");
        for other in codes.iter().skip(index + 1) {
            assert_ne!(code, other, "同一个码不能表示两件事");
        }
    }
}
