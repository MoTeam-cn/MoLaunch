use super::*;

#[test]
fn test_fill_template_supports_tunnel_id() {
    // config 端点 query.tunnel = {id} / {tunnel} 应替换为当前隧道 ID
    let filled = fill_template("{tunnel}", "device-1", "lolia-frp", "16977", "");
    assert_eq!(filled, "16977");

    let filled = fill_template("{id}", "device-1", "lolia-frp", "4722", "");
    assert_eq!(filled, "4722");
}

#[test]
fn test_fill_template_supports_tunnel_name() {
    // Lolia 等厂商 config 端点 query.tunnel = {tunnelName}，填隧道 name（真实标识）
    let filled = fill_template(
        "{tunnelName}",
        "device-1",
        "lolia-frp",
        "16977",
        "my-tunnel",
    );
    assert_eq!(filled, "my-tunnel");
}

#[test]
fn test_fill_template_context_placeholders() {
    let filled = fill_template(
        "{device_id}-{provider_id}",
        "dev-abc",
        "my-provider",
        "tunnel-x",
        "name-y",
    );
    assert_eq!(filled, "dev-abc-my-provider");
}

#[test]
fn test_fill_template_no_placeholders() {
    let filled = fill_template("page=1&limit=100", "d", "p", "t", "n");
    assert_eq!(filled, "page=1&limit=100");
}