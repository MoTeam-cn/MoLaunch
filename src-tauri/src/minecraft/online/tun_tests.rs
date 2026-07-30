//! tun 单元测试

use super::*;

/// 单元测试：VirtualNetInfo 构造
#[test]
fn test_virtual_net_info() {
    let info = VirtualNetInfo {
        name: "tun-test".to_string(),
        ipv4: "10.244.1.1".to_string(),
        prefix_len: 24,
        mtu: 1400,
    };
    assert_eq!(info.name, "tun-test");
    assert_eq!(info.ipv4, "10.244.1.1");
    assert_eq!(info.prefix_len, 24);
    assert_eq!(info.mtu, 1400);
}
