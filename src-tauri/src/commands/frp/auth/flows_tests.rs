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

#[test]
fn test_fill_body_removes_unresolved_secret_for_pkce() {
    // PKCE：无 client_secret 时 {clientSecret} 字段应被删除，而非发送字面量
    let ctx = FlowContext {
        client_id: "my-client".to_string(),
        code: Some("abc".to_string()),
        code_verifier: Some("verifier-123".to_string()),
        client_secret: None,
        ..Default::default()
    };
    let body = serde_json::json!({
        "grant_type": "authorization_code",
        "code": "{code}",
        "client_id": "{clientId}",
        "code_verifier": "{codeVerifier}",
        "client_secret": "{clientSecret}"
    });
    let filled = fill_body_template(&body, &ctx);
    let obj = filled.as_object().unwrap();
    assert_eq!(obj["code_verifier"], "verifier-123");
    // client_secret 字段被删除
    assert!(!obj.contains_key("client_secret"));
    assert_eq!(obj["grant_type"], "authorization_code");
}

#[test]
fn test_fill_body_keeps_secret_when_present() {
    // 非 PKCE：有 client_secret 时正常填充保留，不删除
    let ctx = FlowContext {
        client_id: "my-client".to_string(),
        code: Some("abc".to_string()),
        client_secret: Some("secret-456".to_string()),
        ..Default::default()
    };
    let body = serde_json::json!({
        "client_id": "{clientId}",
        "client_secret": "{clientSecret}"
    });
    let filled = fill_body_template(&body, &ctx);
    let obj = filled.as_object().unwrap();
    assert_eq!(obj["client_secret"], "secret-456");
    assert!(obj.contains_key("client_secret"));
}
