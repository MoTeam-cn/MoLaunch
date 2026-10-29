//! 遥测持久化默认值单测

use super::PersistedState;

#[test]
fn test_persisted_state_默认未同意() {
    let state = PersistedState::default();
    assert!(
        !state.enabled,
        "缺省必须是「未同意」：读不到设置就默认开启，等于替用户做了决定"
    );
}
