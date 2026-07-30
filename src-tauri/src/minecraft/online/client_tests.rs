//! client 单元测试

use super::*;

#[test]
fn test_client_url_trim() {
    let c1 = OnlineClient::new("https://api.example.com/");
    assert_eq!(c1.base_url(), "https://api.example.com");
    let c2 = OnlineClient::new("https://api.example.com");
    assert_eq!(c2.base_url(), "https://api.example.com");
}
