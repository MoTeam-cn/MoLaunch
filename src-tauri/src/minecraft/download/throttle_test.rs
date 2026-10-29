//! 限流压力信号单测

use super::{note_throttle, snapshot};

#[test]
fn test_throttle_计数单调递增且只增不减() {
    let before = snapshot();
    note_throttle();
    let after_one = snapshot();
    assert!(after_one > before, "记一次限流后计数必须增长");
    note_throttle();
    assert!(snapshot() > after_one, "再次限流继续增长");
}

#[test]
fn test_throttle_快照不会被别的调用方取走() {
    note_throttle();
    let first = snapshot();
    let again = snapshot();
    assert_eq!(first, again, "快照是只读的，多个文件各取一次不该互相抢");
}
