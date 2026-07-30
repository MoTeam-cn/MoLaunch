//! bridge 单元测试

use super::*;

#[test]
fn test_bridge_state_default() {
    let state = BridgeState::Stopped;
    assert_eq!(state, BridgeState::Stopped);
}

#[test]
fn test_event_name_format() {
    assert!(EVENT_TUN_PACKET_OUT.starts_with("online://"));
    assert!(EVENT_TUN_PACKET_OUT.contains("tun-packet-out"));
}
