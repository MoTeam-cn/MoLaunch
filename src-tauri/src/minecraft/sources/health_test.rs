use super::*;

#[test]
fn test_still_slow_未到期返回真() {
    assert!(still_slow(100, 50));
}

#[test]
fn test_still_slow_已到期返回假() {
    assert!(!still_slow(100, 100));
    assert!(!still_slow(100, 200));
}

#[test]
fn test_still_slow_零表示正常() {
    assert!(!still_slow(0, 0));
    assert!(!still_slow(0, 9_999));
}

#[test]
fn test_mark_official_slow_之后处于观察期() {
    mark_official_slow();
    assert!(official_is_slow());
    assert!(official_slow_remaining_secs() > 0);
    mark_official_ok();
    assert!(!official_is_slow());
    assert_eq!(official_slow_remaining_secs(), 0);
}
