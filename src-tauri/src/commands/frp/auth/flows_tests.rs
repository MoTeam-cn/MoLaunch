//! flows 单元测试

use super::*;

#[test]
fn test_fill_template() {
    let ctx = FlowContext {
        base_url: Some("https://api.example.com/v1".to_string()),
        client_id: "my-client".to_string(),
        code: Some("abc123".to_string()),
        ..Default::default()
    };
    assert_eq!(fill_template("{clientId}", &ctx), "my-client");
    assert_eq!(fill_template("{code}", &ctx), "abc123");
    assert_eq!(
        fill_template("{baseUrl}/oauth2/token", &ctx),
        "https://api.example.com/v1/oauth2/token"
    );
    assert_eq!(
        fill_template("https://example.com/token?code={code}", &ctx),
        "https://example.com/token?code=abc123"
    );
}

#[test]
fn test_fill_body_template() {
    let ctx = FlowContext {
        client_id: "my-client".to_string(),
        code: Some("abc".to_string()),
        ..Default::default()
    };
    let body = serde_json::json!({
        "grant_type": "authorization_code",
        "code": "{code}",
        "client_id": "{clientId}"
    });
    let filled = fill_body_template(&body, &ctx);
    assert_eq!(filled["code"], "abc");
    assert_eq!(filled["client_id"], "my-client");
    assert_eq!(filled["grant_type"], "authorization_code");
}
